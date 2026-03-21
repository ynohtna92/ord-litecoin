use {
  super::*,
  base64::Engine,
  hyper::{client::HttpConnector, Body, Client, Method, Request, Uri},
  serde_json::{json, Value},
};

pub(crate) struct Fetcher {
  auth: String,
  client: Client<HttpConnector>,
  url: Uri,
}

#[derive(Deserialize, Debug)]
struct JsonResponse<T> {
  error: Option<JsonError>,
  id: usize,
  result: Option<T>,
}

#[derive(Deserialize, Debug)]
struct JsonError {
  code: i32,
  message: String,
}

impl Fetcher {
  pub(crate) fn new(settings: &Settings) -> Result<Self> {
    let client = Client::new();

    let url = if settings.bitcoin_rpc_url(None).starts_with("http://") {
      settings.bitcoin_rpc_url(None)
    } else {
      "http://".to_string() + &settings.bitcoin_rpc_url(None)
    };

    let url = Uri::try_from(&url).map_err(|e| anyhow!("Invalid rpc url {url}: {e}"))?;

    let (user, password) = settings.bitcoin_credentials()?.get_user_pass()?;
    let auth = format!("{}:{}", user.unwrap(), password.unwrap());
    let auth = format!(
      "Basic {}",
      &base64::engine::general_purpose::STANDARD.encode(auth)
    );
    Ok(Fetcher { client, url, auth })
  }

  pub(crate) async fn get_transactions(&self, txids: Vec<Txid>) -> Result<Vec<Transaction>> {
    if txids.is_empty() {
      return Ok(Vec::new());
    }

    let mut reqs = Vec::with_capacity(txids.len());
    for (i, txid) in txids.iter().enumerate() {
      let req = json!({
        "jsonrpc": "2.0",
        "id": i, // Use the index as id, so we can quickly sort the response
        "method": "getrawtransaction",
        "params": [ txid ]
      });
      reqs.push(req);
    }

    let body = Value::Array(reqs).to_string();

    let mut results: Vec<JsonResponse<String>>;
    let mut retries = 0;

    loop {
      results = match self.try_get_transactions(body.clone()).await {
        Ok(results) => results,
        Err(error) => {
          if retries >= 5 {
            return Err(anyhow!(
              "failed to fetch raw transactions after 5 retries: {}",
              error
            ));
          }

          log::info!("failed to fetch raw transactions, retrying: {}", error);

          tokio::time::sleep(Duration::from_millis(100 * u64::pow(2, retries))).await;
          retries += 1;
          continue;
        }
      };
      break;
    }

    if results.len() != txids.len() {
      return Err(anyhow!(
        "batched JSON-RPC returned {} results for {} txids",
        results.len(),
        txids.len()
      ));
    }

    results.sort_by(|a, b| a.id.cmp(&b.id));

    let mut txs = Vec::with_capacity(txids.len());

    for (expected_id, (txid, res)) in txids.iter().zip(results.into_iter()).enumerate() {
      if res.id != expected_id {
        return Err(anyhow!(
          "batched JSON-RPC response id mismatch for {txid}: expected {expected_id}, got {}",
          res.id
        ));
      }

      match self.parse_batched_response(txid, res) {
        Ok(tx) => txs.push(tx),
        Err(error) => {
          log::warn!(
            "batched getrawtransaction decode failed for {txid}: {error}; retrying individually"
          );

          let tx = self
            .get_transaction(*txid)
            .await
            .map_err(|single_error| {
              anyhow!(
                "failed to fetch transaction {txid} individually after batch failure: {single_error}"
              )
            })?;

          txs.push(tx);
        }
      }
    }

    Ok(txs)
  }

  fn parse_batched_response(&self, txid: &Txid, res: JsonResponse<String>) -> Result<Transaction> {
    if let Some(err) = res.error {
      return Err(anyhow!(
        "batched JSON-RPC error for {txid}: code {} message {}",
        err.code,
        err.message
      ));
    }

    let raw = res
      .result
      .ok_or_else(|| anyhow!("missing result for batched JSON-RPC response for {txid}"))?;

    self.deserialize_transaction(txid, &raw, "batched JSON-RPC response")
  }

  async fn get_transaction(&self, txid: Txid) -> Result<Transaction> {
    let body = Value::Array(vec![json!({
      "jsonrpc": "2.0",
      "id": 0,
      "method": "getrawtransaction",
      "params": [ txid ],
    })])
    .to_string();

    let mut results = self.try_get_transactions(body).await?;

    if results.len() != 1 {
      return Err(anyhow!(
        "single JSON-RPC response for {txid} returned {} results",
        results.len()
      ));
    }

    let res = results.pop().unwrap();

    if let Some(err) = res.error {
      return Err(anyhow!(
        "single JSON-RPC error for {txid}: code {} message {}",
        err.code,
        err.message
      ));
    }

    let raw = res
      .result
      .ok_or_else(|| anyhow!("missing result for single JSON-RPC response for {txid}"))?;

    self.deserialize_transaction(&txid, &raw, "single JSON-RPC response")
  }

  fn deserialize_transaction(
    &self,
    txid: &Txid,
    raw: &str,
    source: &str,
  ) -> Result<Transaction> {
    let hex =
      hex::decode(raw).map_err(|e| anyhow!("{source} for {txid} not valid hex: {e}"))?;

    match consensus::deserialize(&hex) {
      Ok(tx) => Ok(tx),
      Err(error) => {
        if let Some(tx) = self.try_deserialize_litecoin_mweb(txid, &hex)? {
          log::warn!("{source} for {txid} required Litecoin MWEB fallback deserialization");
          return Ok(tx);
        }

        Err(anyhow!("{source} for {txid} not valid bitcoin tx: {error}"))
      }
    }
  }

  fn try_deserialize_litecoin_mweb(
    &self,
    txid: &Txid,
    raw: &[u8],
  ) -> Result<Option<Transaction>> {
    if raw.len() < 10 || raw[4] != 0 {
      return Ok(None);
    }

    let flags = raw[5];
    if flags & 8 == 0 {
      return Ok(None);
    }

    let (_, consumed) = match consensus::encode::deserialize_partial::<Transaction>(raw) {
      Ok(result) => result,
      Err(_) => return Ok(None),
    };

    if consumed < 10 || consumed > raw.len() || raw.len() < 4 {
      return Ok(None);
    }

    let body_end = consumed
      .checked_sub(4)
      .ok_or_else(|| anyhow!("partial deserialize for {txid} consumed too few bytes"))?;

    if body_end < 6 {
      return Ok(None);
    }

    let mut canonical = Vec::with_capacity(raw.len());
    canonical.extend_from_slice(&raw[..4]);

    if flags & 1 != 0 {
      canonical.push(0);
      canonical.push(1);
    }

    canonical.extend_from_slice(&raw[6..body_end]);
    canonical.extend_from_slice(&raw[raw.len() - 4..]);

    let tx = consensus::deserialize(&canonical)
      .map_err(|error| anyhow!("Litecoin MWEB fallback deserialize failed for {txid}: {error}"))?;

    Ok(Some(tx))
  }

  async fn try_get_transactions(&self, body: String) -> Result<Vec<JsonResponse<String>>> {
    let req = Request::builder()
      .method(Method::POST)
      .uri(&self.url)
      .header(hyper::header::AUTHORIZATION, &self.auth)
      .header(hyper::header::CONTENT_TYPE, "application/json")
      .body(Body::from(body))?;

    let response = self.client.request(req).await?;

    let buf = hyper::body::to_bytes(response).await?;

    let results: Vec<JsonResponse<String>> = match serde_json::from_slice(&buf) {
      Ok(results) => results,
      Err(e) => {
        return Err(anyhow!(
          "failed to parse JSON-RPC response: {e}. response: {response}",
          e = e,
          response = String::from_utf8_lossy(&buf)
        ))
      }
    };

    Ok(results)
  }
}

use {super::*, bitcoin::BlockHash};

#[test]
fn get_sat_without_sat_index() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let response = TestServer::spawn_with_server_args(&rpc_server, &[], &["--enable-json-api"])
    .json_request("/sat/8399999990759999");

  assert_eq!(response.status(), StatusCode::OK);

  let mut sat_json: SatJson = serde_json::from_str(&response.text().unwrap()).unwrap();

  // this is a hack to ignore the timestamp, since it changes for every request
  sat_json.timestamp = 0;

  pretty_assert_eq!(
    sat_json,
    SatJson {
      number: 8399999990759999,
      decimal: "27719999.0".into(),
      degree: "10°839999′2015″0‴".into(),
      name: "a".into(),
      block: 27719999,
      cycle: 10,
      epoch: 32,
      period: 13749,
      offset: 0,
      rarity: Rarity::Uncommon,
      percentile: "100%".into(),
      satpoint: None,
      timestamp: 0,
      inscriptions: vec![],
    }
  )
}

#[test]
fn get_sat_with_inscription_and_sat_index() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);

  let (inscription_id, reveal) = inscribe(&rpc_server);

  let response =
    TestServer::spawn_with_server_args(&rpc_server, &["--index-sats"], &["--enable-json-api"])
      .json_request(format!("/sat/{}", 50 * COIN_VALUE));

  assert_eq!(response.status(), StatusCode::OK);

  let sat_json: SatJson = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    sat_json,
    SatJson {
      number: 50 * COIN_VALUE,
      decimal: "1.0".into(),
      degree: "0°1′1″0‴".into(),
      name: "bgmbpulndxfd".into(),
      block: 1,
      cycle: 0,
      epoch: 0,
      period: 0,
      offset: 0,
      rarity: Rarity::Uncommon,
      percentile: "0.00005952380958928572%".into(),
      satpoint: Some(SatPoint::from_str(&format!("{}:{}:{}", reveal, 0, 0)).unwrap()),
      timestamp: 1,
      inscriptions: vec![inscription_id],
    }
  )
}

#[test]
fn get_sat_with_inscription_on_common_sat_and_more_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);

  inscribe(&rpc_server);

  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let Inscribe { reveal, .. } = CommandBuilder::new(format!(
    "wallet inscribe --satpoint {}:0:1 --fee-rate 1 --file foo.txt",
    txid
  ))
  .write("foo.txt", "FOO")
  .rpc_server(&rpc_server)
  .run_and_deserialize_output();

  rpc_server.mine_blocks(1);
  let inscription_id = InscriptionId {
    txid: reveal,
    index: 0,
  };

  let response =
    TestServer::spawn_with_server_args(&rpc_server, &["--index-sats"], &["--enable-json-api"])
      .json_request(format!("/sat/{}", 3 * 50 * COIN_VALUE + 1));

  assert_eq!(response.status(), StatusCode::OK);

  let sat_json: SatJson = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    sat_json,
    SatJson {
      number: 3 * 50 * COIN_VALUE + 1,
      decimal: "3.1".into(),
      degree: "0°3′3″1‴".into(),
      name: "bgmboobwefum".into(),
      block: 3,
      cycle: 0,
      epoch: 0,
      period: 0,
      offset: 1,
      rarity: Rarity::Common,
      percentile: "0.00017857142877976192%".into(),
      satpoint: Some(SatPoint::from_str(&format!("{}:{}:{}", reveal, 0, 0)).unwrap()),
      timestamp: 3,
      inscriptions: vec![inscription_id],
    }
  )
}

#[test]
fn get_inscription() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);

  let (inscription_id, reveal) = inscribe(&rpc_server);

  let response =
    TestServer::spawn_with_server_args(&rpc_server, &["--index-sats"], &["--enable-json-api"])
      .json_request(format!("/inscription/{}", inscription_id));

  assert_eq!(response.status(), StatusCode::OK);

  let mut inscription_json: InscriptionJson =
    serde_json::from_str(&response.text().unwrap()).unwrap();
  assert_regex_match!(inscription_json.address.unwrap(), r"ltc1p.*");
  inscription_json.address = None;
  assert_regex_match!(inscription_json.genesis_address.unwrap(), r"ltc1p.*");
  inscription_json.genesis_address = None;
  inscription_json.script_pubkey = None;

  pretty_assert_eq!(
    inscription_json,
    InscriptionJson {
      address: None,
      charms: vec!["coin".into(), "uncommon".into()],
      children: Vec::new(),
      content_length: Some(3),
      content_type: Some("text/plain;charset=utf-8".to_string()),
      genesis_fee: 138,
      genesis_height: 2,
      genesis_address: None,
      inscription_id,
      inscription_number: 0,
      next: None,
      output_value: Some(10000),
      parent: None,
      previous: None,
      rune: None,
      script_pubkey: None,
      sat: Some(ord::Sat(50 * COIN_VALUE)),
      satpoint: SatPoint::from_str(&format!("{}:{}:{}", reveal, 0, 0)).unwrap(),
      timestamp: 2,
    }
  )
}

#[test]
fn get_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);

  let witness = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

  let mut inscriptions = Vec::new();

  // Create 150 inscriptions
  for i in 0..50 {
    rpc_server.mine_blocks(1);
    rpc_server.mine_blocks(1);
    rpc_server.mine_blocks(1);

    let txid = rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[
        (i * 3 + 1, 0, 0, witness.clone()),
        (i * 3 + 2, 0, 0, witness.clone()),
        (i * 3 + 3, 0, 0, witness.clone()),
      ],
      ..Default::default()
    });

    inscriptions.push(InscriptionId { txid, index: 0 });
    inscriptions.push(InscriptionId { txid, index: 1 });
    inscriptions.push(InscriptionId { txid, index: 2 });
  }

  rpc_server.mine_blocks(1);

  let server =
    TestServer::spawn_with_server_args(&rpc_server, &["--index-sats"], &["--enable-json-api"]);

  let response = server.json_request("/inscriptions");
  assert_eq!(response.status(), StatusCode::OK);
  let inscriptions_json: InscriptionsJson =
    serde_json::from_str(&response.text().unwrap()).unwrap();

  assert_eq!(inscriptions_json.inscriptions.len(), 100);
  assert!(inscriptions_json.more);
  assert_eq!(inscriptions_json.page_index, 0);

  let response = server.json_request("/inscriptions/1");
  assert_eq!(response.status(), StatusCode::OK);
  let inscriptions_json: InscriptionsJson =
    serde_json::from_str(&response.text().unwrap()).unwrap();

  assert_eq!(inscriptions_json.inscriptions.len(), 50);
  assert!(!inscriptions_json.more);
  assert_eq!(inscriptions_json.page_index, 1);
}

#[test]
fn get_inscriptions_in_block() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);
  rpc_server.mine_blocks(10);

  let envelope = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

  let txid = rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[
      (1, 0, 0, envelope.clone()),
      (2, 0, 0, envelope.clone()),
      (3, 0, 0, envelope.clone()),
    ],
    ..Default::default()
  });

  rpc_server.mine_blocks(1);

  let _ = rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[(4, 0, 0, envelope.clone()), (5, 0, 0, envelope.clone())],
    ..Default::default()
  });

  rpc_server.mine_blocks(1);

  let _ = rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[(6, 0, 0, envelope.clone())],
    ..Default::default()
  });

  rpc_server.mine_blocks(1);

  let server = TestServer::spawn_with_server_args(
    &rpc_server,
    &["--index-sats", "--first-inscription-height", "0"],
    &["--enable-json-api"],
  );

  // get all inscriptions from block 11
  let response = server.json_request(format!("/inscriptions/block/{}", 11));
  assert_eq!(response.status(), StatusCode::OK);

  let inscriptions_json: InscriptionsJson =
    serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    inscriptions_json.inscriptions,
    vec![
      InscriptionId { txid, index: 0 },
      InscriptionId { txid, index: 1 },
      InscriptionId { txid, index: 2 },
    ]
  );
}

#[test]
fn get_output() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);
  rpc_server.mine_blocks(3);

  let envelope = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

  let txid = rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[
      (1, 0, 0, envelope.clone()),
      (2, 0, 0, envelope.clone()),
      (3, 0, 0, envelope.clone()),
    ],
    ..Default::default()
  });
  rpc_server.mine_blocks(1);

  let server =
    TestServer::spawn_with_server_args(&rpc_server, &["--index-sats"], &["--enable-json-api"]);

  let response = server.json_request(format!("/output/{}:0", txid));
  assert_eq!(response.status(), StatusCode::OK);

  let output_json: OutputJson = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    output_json,
    OutputJson {
      value: 3 * 50 * COIN_VALUE,
      script_pubkey: "".to_string(),
      address: None,
      transaction: txid.to_string(),
      sat_ranges: Some(vec![
        (5000000000, 10000000000,),
        (10000000000, 15000000000,),
        (15000000000, 20000000000,),
      ],),
      inscriptions: vec![
        InscriptionId { txid, index: 0 },
        InscriptionId { txid, index: 1 },
        InscriptionId { txid, index: 2 },
      ],
      runes: BTreeMap::new(),
    }
  );
}

#[test]
fn json_request_fails_when_not_enabled() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let response =
    TestServer::spawn_with_args(&rpc_server, &[]).json_request("/sat/2099999997689999");

  assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
}

#[test]
fn get_block() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  rpc_server.mine_blocks(1);

  let response = TestServer::spawn_with_server_args(&rpc_server, &[], &["--enable-json-api"])
    .json_request("/block/0");

  assert_eq!(response.status(), StatusCode::OK);

  let block_json: BlockJson = serde_json::from_str(&response.text().unwrap()).unwrap();

  assert_eq!(
    block_json,
    BlockJson {
      hash: "12a765e31ffd4059bada1e25190f6e98c99d9714d334efa41a195a7e7e04bfe2"
        .parse::<BlockHash>()
        .unwrap(),
      target: "00000ffff0000000000000000000000000000000000000000000000000000000"
        .parse::<BlockHash>()
        .unwrap(),
      best_height: 1,
      height: 0,
      inscriptions: vec![],
    }
  );
}

#[test]
fn get_status() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  inscribe(&rpc_server);

  let response = TestServer::spawn_with_server_args(
    &rpc_server,
    &["--regtest", "--index-sats", "--index-runes"],
    &["--enable-json-api"],
  )
  .json_request("/status");

  assert_eq!(response.status(), StatusCode::OK);

  let mut status_json: StatusHtml = serde_json::from_str(&response.text().unwrap()).unwrap();

  let dummy_started = "2012-12-12 12:12:12+00:00"
    .parse::<DateTime<Utc>>()
    .unwrap();

  let dummy_uptime = Duration::from_secs(1);

  status_json.started = dummy_started;
  status_json.uptime = dummy_uptime;

  pretty_assert_eq!(
    status_json,
    StatusHtml {
      blessed_inscriptions: 1,
      cursed_inscriptions: 0,
      chain: Chain::Regtest,
      height: Some(3),
      inscriptions: 1,
      lost_sats: 0,
      minimum_rune_for_next_block: Rune(99239298574102199),
      rune_index: true,
      runes: 0,
      sat_index: true,
      started: dummy_started,
      transaction_index: false,
      unrecoverably_reorged: false,
      uptime: dummy_uptime,
    }
  );
}

#[test]
fn get_runes() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  create_wallet(&rpc_server);
  rpc_server.mine_blocks(3);

  let a = etch(&rpc_server, Rune(RUNE));
  let b = etch(&rpc_server, Rune(RUNE + 1));
  let c = etch(&rpc_server, Rune(RUNE + 2));

  rpc_server.mine_blocks(1);

  let server = TestServer::spawn_with_server_args(
    &rpc_server,
    &["--index-runes", "--regtest"],
    &["--enable-json-api"],
  );

  let response = server.json_request(format!("/rune/{}", a.rune));
  assert_eq!(response.status(), StatusCode::OK);

  let rune_json: RuneJson = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    rune_json,
    RuneJson {
      entry: RuneEntry {
        burned: 0,
        deadline: None,
        divisibility: 0,
        end: None,
        etching: a.transaction,
        limit: None,
        mints: 0,
        number: 0,
        rune: Rune(RUNE),
        spacers: 0,
        supply: 1000,
        symbol: Some('¢'),
        timestamp: 5,
      },
      id: RuneId {
        height: 5,
        index: 1
      },
      parent: None,
    }
  );

  let response = server.json_request("/runes");

  assert_eq!(response.status(), StatusCode::OK);

  let runes_json: RunesJson = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    runes_json,
    RunesJson {
      entries: vec![
        (
          RuneId {
            height: 5,
            index: 1
          },
          RuneEntry {
            burned: 0,
            deadline: None,
            divisibility: 0,
            end: None,
            etching: a.transaction,
            limit: None,
            mints: 0,
            number: 0,
            rune: Rune(RUNE),
            spacers: 0,
            supply: 1000,
            symbol: Some('¢'),
            timestamp: 5,
          }
        ),
        (
          RuneId {
            height: 7,
            index: 1
          },
          RuneEntry {
            burned: 0,
            deadline: None,
            divisibility: 0,
            end: None,
            etching: b.transaction,
            limit: None,
            mints: 0,
            number: 1,
            rune: Rune(RUNE + 1),
            spacers: 0,
            supply: 1000,
            symbol: Some('¢'),
            timestamp: 7,
          }
        ),
        (
          RuneId {
            height: 9,
            index: 1
          },
          RuneEntry {
            burned: 0,
            deadline: None,
            divisibility: 0,
            end: None,
            etching: c.transaction,
            limit: None,
            mints: 0,
            number: 2,
            rune: Rune(RUNE + 2),
            spacers: 0,
            supply: 1000,
            symbol: Some('¢'),
            timestamp: 9,
          }
        )
      ]
    }
  );
}

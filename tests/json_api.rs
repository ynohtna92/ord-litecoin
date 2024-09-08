use {
  super::*,
  bitcoin::BlockHash,
  ord::{Envelope, Inscription},
};

#[test]
fn get_sat_without_sat_index() {
  let core = mockcore::spawn();

  let response =
    TestServer::spawn_with_server_args(&core, &[], &[]).json_request("/sat/8399999990759999");

  assert_eq!(response.status(), StatusCode::OK);

  let mut sat_json: api::Sat = serde_json::from_str(&response.text().unwrap()).unwrap();

  // this is a hack to ignore the timestamp, since it changes for every request
  sat_json.timestamp = 0;

  pretty_assert_eq!(
    sat_json,
    api::Sat {
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
      inscriptions: Vec::new(),
      charms: vec![Charm::Uncommon],
    }
  )
}

#[test]
fn get_sat_with_inscription_and_sat_index() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-sats"], &[]);

  create_wallet(&core, &ord);

  let (inscription_id, reveal) = inscribe(&core, &ord);

  let response = ord.json_request(format!("/sat/{}", 50 * COIN_VALUE));

  assert_eq!(response.status(), StatusCode::OK);

  let sat_json: api::Sat = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    sat_json,
    api::Sat {
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
      charms: vec![Charm::Coin, Charm::Uncommon],
    }
  )
}

#[test]
fn get_sat_with_inscription_on_common_sat_and_more_inscriptions() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-sats"], &[]);

  create_wallet(&core, &ord);

  inscribe(&core, &ord);

  let txid = core.mine_blocks(1)[0].txdata[0].txid();

  let Batch { reveal, .. } = CommandBuilder::new(format!(
    "wallet inscribe --satpoint {}:0:1 --fee-rate 1 --file foo.txt",
    txid
  ))
  .write("foo.txt", "FOO")
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output();

  core.mine_blocks(1);

  let inscription_id = InscriptionId {
    txid: reveal,
    index: 0,
  };

  let response = ord.json_request(format!("/sat/{}", 3 * 50 * COIN_VALUE + 1));

  assert_eq!(response.status(), StatusCode::OK);

  let sat_json: api::Sat = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    sat_json,
    api::Sat {
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
      charms: Vec::new(),
    }
  )
}

#[test]
fn get_inscription() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-sats"], &[]);

  create_wallet(&core, &ord);

  let (inscription_id, reveal) = inscribe(&core, &ord);

  let response = ord.json_request(format!("/inscription/{}", inscription_id));

  assert_eq!(response.status(), StatusCode::OK);

  let mut inscription_json: api::Inscription =
    serde_json::from_str(&response.text().unwrap()).unwrap();
  assert_regex_match!(inscription_json.address.unwrap(), r"ltc1p.*");
  inscription_json.address = None;

  pretty_assert_eq!(
    inscription_json,
    api::Inscription {
      address: None,
      charms: vec![Charm::Coin, Charm::Uncommon],
      children: Vec::new(),
      content_length: Some(3),
      content_type: Some("text/plain;charset=utf-8".to_string()),
      effective_content_type: Some("text/plain;charset=utf-8".to_string()),
      fee: 138,
      height: 2,
      id: inscription_id,
      number: 0,
      next: None,
      value: Some(10000),
      parents: Vec::new(),
      previous: None,
      rune: None,
      sat: Some(Sat(50 * COIN_VALUE)),
      satpoint: SatPoint::from_str(&format!("{}:{}:{}", reveal, 0, 0)).unwrap(),
      timestamp: 2,
    }
  )
}

#[test]
fn get_inscriptions() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-sats"], &[]);

  create_wallet(&core, &ord);

  let witness = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

  let mut inscriptions = Vec::new();

  // Create 150 inscriptions
  for i in 0..50 {
    core.mine_blocks(1);
    core.mine_blocks(1);
    core.mine_blocks(1);

    let txid = core.broadcast_tx(TransactionTemplate {
      inputs: &[
        (i * 3 + 1, 0, 0, witness.clone()),
        (i * 3 + 2, 0, 0, witness.clone()),
        (i * 3 + 3, 0, 0, witness.clone()),
      ],
      ..default()
    });

    inscriptions.push(InscriptionId { txid, index: 0 });
    inscriptions.push(InscriptionId { txid, index: 1 });
    inscriptions.push(InscriptionId { txid, index: 2 });
  }

  core.mine_blocks(1);

  let response = ord.json_request("/inscriptions");
  assert_eq!(response.status(), StatusCode::OK);
  let inscriptions_json: api::Inscriptions =
    serde_json::from_str(&response.text().unwrap()).unwrap();

  assert_eq!(inscriptions_json.ids.len(), 100);
  assert!(inscriptions_json.more);
  assert_eq!(inscriptions_json.page_index, 0);

  let response = ord.json_request("/inscriptions/1");
  assert_eq!(response.status(), StatusCode::OK);
  let inscriptions_json: api::Inscriptions =
    serde_json::from_str(&response.text().unwrap()).unwrap();

  assert_eq!(inscriptions_json.ids.len(), 50);
  assert!(!inscriptions_json.more);
  assert_eq!(inscriptions_json.page_index, 1);
}

#[test]
fn get_inscriptions_in_block() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(
    &core,
    &["--index-sats", "--first-inscription-height", "0"],
    &[],
  );

  create_wallet(&core, &ord);

  core.mine_blocks(10);

  let envelope = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

  let txid = core.broadcast_tx(TransactionTemplate {
    inputs: &[
      (1, 0, 0, envelope.clone()),
      (2, 0, 0, envelope.clone()),
      (3, 0, 0, envelope.clone()),
    ],
    ..default()
  });

  core.mine_blocks(1);

  let _ = core.broadcast_tx(TransactionTemplate {
    inputs: &[(4, 0, 0, envelope.clone()), (5, 0, 0, envelope.clone())],
    ..default()
  });

  core.mine_blocks(1);

  let _ = core.broadcast_tx(TransactionTemplate {
    inputs: &[(6, 0, 0, envelope.clone())],
    ..default()
  });

  core.mine_blocks(1);

  // get all inscriptions from block 11
  let response = ord.json_request(format!("/inscriptions/block/{}", 11));
  assert_eq!(response.status(), StatusCode::OK);

  let inscriptions_json: api::Inscriptions =
    serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    inscriptions_json.ids,
    vec![
      InscriptionId { txid, index: 0 },
      InscriptionId { txid, index: 1 },
      InscriptionId { txid, index: 2 },
    ]
  );
}

#[test]
fn get_output() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn(&core);

  create_wallet(&core, &ord);
  core.mine_blocks(3);

  let envelope = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

  let txid = core.broadcast_tx(TransactionTemplate {
    inputs: &[
      (1, 0, 0, envelope.clone()),
      (2, 0, 0, envelope.clone()),
      (3, 0, 0, envelope.clone()),
    ],
    ..default()
  });

  core.mine_blocks(1);

  let server = TestServer::spawn_with_server_args(&core, &["--index-sats"], &["--no-sync"]);

  let response = reqwest::blocking::Client::new()
    .get(server.url().join(&format!("/output/{}:0", txid)).unwrap())
    .header(reqwest::header::ACCEPT, "application/json")
    .send()
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  assert!(
    !serde_json::from_str::<api::Output>(&response.text().unwrap())
      .unwrap()
      .indexed
  );

  let server = TestServer::spawn_with_server_args(&core, &["--index-sats"], &[]);

  let response = server.json_request(format!("/output/{}:0", txid));
  assert_eq!(response.status(), StatusCode::OK);

  let output_json: api::Output = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    output_json,
    api::Output {
      address: Some(
        "bc1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq9e75rs"
          .parse()
          .unwrap()
      ),
      inscriptions: vec![
        InscriptionId { txid, index: 0 },
        InscriptionId { txid, index: 1 },
        InscriptionId { txid, index: 2 },
      ],
      indexed: true,
      runes: BTreeMap::new(),
      sat_ranges: Some(vec![
        (5000000000, 10000000000,),
        (10000000000, 15000000000,),
        (15000000000, 20000000000,),
      ],),
      script_pubkey: "OP_0 OP_PUSHBYTES_20 0000000000000000000000000000000000000000".into(),
      spent: false,
      transaction: txid.to_string(),
      value: 3 * 50 * COIN_VALUE,
    }
  );
}

#[test]
fn json_request_fails_when_disabled() {
  let core = mockcore::spawn();

  let response = TestServer::spawn_with_server_args(&core, &[], &["--disable-json-api"])
    .json_request("/sat/2099999997689999");

  assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
}

#[test]
fn get_block() {
  let core = mockcore::spawn();

  core.mine_blocks(1);

  let response = TestServer::spawn_with_server_args(&core, &[], &[]).json_request("/block/0");

  assert_eq!(response.status(), StatusCode::OK);

  let block_json: api::Block = serde_json::from_str(&response.text().unwrap()).unwrap();

  assert_eq!(
    block_json,
    api::Block {
      hash: "12a765e31ffd4059bada1e25190f6e98c99d9714d334efa41a195a7e7e04bfe2"
        .parse::<BlockHash>()
        .unwrap(),
      target: "00000ffff0000000000000000000000000000000000000000000000000000000"
        .parse::<BlockHash>()
        .unwrap(),
      best_height: 1,
      height: 0,
      inscriptions: Vec::new(),
      runes: Vec::new(),
      transactions: block_json.transactions.clone(),
    }
  );
}

#[test]
fn get_blocks() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn(&core);

  let blocks: Vec<BlockHash> = core
    .mine_blocks(101)
    .iter()
    .rev()
    .take(100)
    .map(|block| block.block_hash())
    .collect();

  ord.sync_server();

  let response = ord.json_request("/blocks");

  assert_eq!(response.status(), StatusCode::OK);

  let blocks_json: api::Blocks = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    blocks_json,
    api::Blocks {
      last: 101,
      blocks: blocks.clone(),
      featured_blocks: blocks
        .into_iter()
        .take(5)
        .map(|block_hash| (block_hash, Vec::new()))
        .collect(),
    }
  );
}

#[test]
fn get_transaction() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn(&core);

  let transaction = core.mine_blocks(1)[0].txdata[0].clone();

  let txid = transaction.txid();

  let response = ord.json_request(format!("/tx/{txid}"));

  assert_eq!(response.status(), StatusCode::OK);

  assert_eq!(
    serde_json::from_str::<api::Transaction>(&response.text().unwrap()).unwrap(),
    api::Transaction {
      chain: Chain::Mainnet,
      etching: None,
      inscription_count: 0,
      transaction,
      txid,
    }
  );
}

#[test]
fn get_status() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord =
    TestServer::spawn_with_server_args(&core, &["--regtest", "--index-sats", "--index-runes"], &[]);

  create_wallet(&core, &ord);
  core.mine_blocks(1);

  inscribe(&core, &ord);

  let response = ord.json_request("/status");

  assert_eq!(response.status(), StatusCode::OK);

  let mut status_json: api::Status = serde_json::from_str(&response.text().unwrap()).unwrap();

  let dummy_started = "2012-12-12 12:12:12+00:00"
    .parse::<DateTime<Utc>>()
    .unwrap();

  let dummy_duration = Duration::from_secs(1);

  status_json.initial_sync_time = dummy_duration;
  status_json.started = dummy_started;
  status_json.uptime = dummy_duration;

  pretty_assert_eq!(
    status_json,
    api::Status {
      address_index: false,
      blessed_inscriptions: 1,
      chain: Chain::Regtest,
      cursed_inscriptions: 0,
      height: Some(3),
      initial_sync_time: dummy_duration,
      inscriptions: 1,
      lost_sats: 0,
      minimum_rune_for_next_block: Rune(99239298574102199),
      rune_index: true,
      runes: 0,
      sat_index: true,
      started: dummy_started,
      transaction_index: false,
      unrecoverably_reorged: false,
      uptime: dummy_duration,
    }
  );
}

#[test]
fn get_runes() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(3);

  let a = etch(&core, &ord, Rune(RUNE));
  let b = etch(&core, &ord, Rune(RUNE + 1));
  let c = etch(&core, &ord, Rune(RUNE + 2));

  core.mine_blocks(1);

  let response = ord.json_request(format!("/rune/{}", a.output.rune.unwrap().rune));
  assert_eq!(response.status(), StatusCode::OK);

  let rune_json: api::Rune = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    rune_json,
    api::Rune {
      entry: RuneEntry {
        block: a.id.block,
        burned: 0,
        terms: None,
        divisibility: 0,
        etching: a.output.reveal,
        mints: 0,
        number: 0,
        premine: 1000,
        spaced_rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0
        },
        symbol: Some('¢'),
        timestamp: 10,
        turbo: false,
      },
      id: RuneId { block: 10, tx: 1 },
      mintable: false,
      parent: Some(InscriptionId {
        txid: a.output.reveal,
        index: 0,
      }),
    }
  );

  let response = ord.json_request("/runes");

  assert_eq!(response.status(), StatusCode::OK);

  let runes_json: api::Runes = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    runes_json,
    api::Runes {
      entries: vec![
        (
          RuneId { block: 24, tx: 1 },
          RuneEntry {
            block: c.id.block,
            burned: 0,
            terms: None,
            divisibility: 0,
            etching: c.output.reveal,
            mints: 0,
            number: 2,
            premine: 1000,
            spaced_rune: SpacedRune {
              rune: Rune(RUNE + 2),
              spacers: 0
            },
            symbol: Some('¢'),
            timestamp: 24,
            turbo: false,
          }
        ),
        (
          RuneId { block: 17, tx: 1 },
          RuneEntry {
            block: b.id.block,
            burned: 0,
            terms: None,
            divisibility: 0,
            etching: b.output.reveal,
            mints: 0,
            number: 1,
            premine: 1000,
            spaced_rune: SpacedRune {
              rune: Rune(RUNE + 1),
              spacers: 0
            },
            symbol: Some('¢'),
            timestamp: 17,
            turbo: false,
          }
        ),
        (
          RuneId { block: 10, tx: 1 },
          RuneEntry {
            block: a.id.block,
            burned: 0,
            terms: None,
            divisibility: 0,
            etching: a.output.reveal,
            mints: 0,
            number: 0,
            premine: 1000,
            spaced_rune: SpacedRune {
              rune: Rune(RUNE),
              spacers: 0
            },
            symbol: Some('¢'),
            timestamp: 10,
            turbo: false,
          }
        )
      ],
      more: false,
      next: None,
      prev: None,
    }
  );
}

#[test]
fn get_runes_balances() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(3);

  let rune0 = Rune(RUNE);
  let rune1 = Rune(RUNE + 1);
  let rune2 = Rune(RUNE + 2);

  let e0 = etch(&core, &ord, rune0);
  let e1 = etch(&core, &ord, rune1);
  let e2 = etch(&core, &ord, rune2);

  core.mine_blocks(1);

  let rune_balances: BTreeMap<Rune, BTreeMap<OutPoint, u128>> = vec![
    (
      rune0,
      vec![(
        OutPoint {
          txid: e0.output.reveal,
          vout: 1,
        },
        1000,
      )]
      .into_iter()
      .collect(),
    ),
    (
      rune1,
      vec![(
        OutPoint {
          txid: e1.output.reveal,
          vout: 1,
        },
        1000,
      )]
      .into_iter()
      .collect(),
    ),
    (
      rune2,
      vec![(
        OutPoint {
          txid: e2.output.reveal,
          vout: 1,
        },
        1000,
      )]
      .into_iter()
      .collect(),
    ),
  ]
  .into_iter()
  .collect();

  let response = ord.json_request("/runes/balances");
  assert_eq!(response.status(), StatusCode::OK);

  let runes_balance_json: BTreeMap<Rune, BTreeMap<OutPoint, u128>> =
    serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(runes_balance_json, rune_balances);
}

#[test]
fn get_decode_tx() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);
  core.mine_blocks(3);

  let envelope = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

  let txid = core.broadcast_tx(TransactionTemplate {
    inputs: &[(1, 0, 0, envelope.clone())],
    ..default()
  });

  let transaction = core.mine_blocks(1)[0].txdata[0].clone();

  let inscriptions = vec![Envelope {
    payload: Inscription {
      body: Some(vec![98, 97, 114]),
      content_type: Some(b"text/plain;charset=utf-8".into()),
      ..default()
    },
    input: 0,
    offset: 0,
    pushnum: false,
    stutter: false,
  }];
  let runestone = Runestone::decipher(&transaction);
  let response = ord.json_request(format!("/decode/{txid}"));

  assert_eq!(response.status(), StatusCode::OK);

  assert_eq!(
    serde_json::from_str::<api::Decode>(&response.text().unwrap()).unwrap(),
    api::Decode {
      inscriptions,
      runestone,
    }
  );
}

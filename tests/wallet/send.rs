use {super::*, ord::subcommand::wallet::send::Output};

#[test]
fn inscriptions_can_be_sent() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let Inscribe { inscription, .. } = inscribe(&rpc_server);

  rpc_server.mine_blocks(1);

  let stdout = CommandBuilder::new(format!(
    "wallet send --fee-rate 1 ltc1qfmvk898k6jgfgp98dhsc5gvr9hpxl2ggd25ygk {inscription}",
  ))
  .rpc_server(&rpc_server)
  .stdout_regex(r".*")
  .run_and_extract_stdout();

  let txid = rpc_server.mempool()[0].txid();
  assert_eq!(format!("{txid}\n"), stdout);

  rpc_server.mine_blocks(1);

  let send_txid = stdout.trim();

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);
  ord_server.assert_response_regex(
    format!("/inscription/{inscription}"),
    format!(
      ".*<h1>Inscription 0</h1>.*<dl>.*
  <dt>content length</dt>
  <dd>3 bytes</dd>
  <dt>content type</dt>
  <dd>text/plain;charset=utf-8</dd>
  .*
  <dt>location</dt>
  <dd class=monospace>{send_txid}:0:0</dd>
  .*
</dl>
.*",
    ),
  );
}

#[test]
fn send_unknown_inscription() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 ltc1qfmvk898k6jgfgp98dhsc5gvr9hpxl2ggd25ygk {txid}i0"
  ))
  .rpc_server(&rpc_server)
  .expected_stderr(format!("error: Inscription {txid}i0 not found\n"))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn send_inscribed_sat() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let Inscribe { inscription, .. } = inscribe(&rpc_server);

  rpc_server.mine_blocks(1);

  let stdout = CommandBuilder::new(format!(
    "wallet send --fee-rate 1 ltc1qfmvk898k6jgfgp98dhsc5gvr9hpxl2ggd25ygk {inscription}",
  ))
  .rpc_server(&rpc_server)
  .stdout_regex("[[:xdigit:]]{64}\n")
  .run_and_extract_stdout();

  rpc_server.mine_blocks(1);

  let send_txid = stdout.trim();

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);
  ord_server.assert_response_regex(
    format!("/inscription/{inscription}"),
    format!(
      ".*<h1>Inscription 0</h1>.*<dt>location</dt>.*<dd class=monospace>{send_txid}:0:0</dd>.*",
    ),
  );
}

#[test]
fn send_on_mainnnet_works_with_wallet_named_foo() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  CommandBuilder::new("--wallet foo wallet create")
    .rpc_server(&rpc_server)
    .run_and_check_output::<Create>();

  CommandBuilder::new(format!(
    "--wallet foo wallet send --fee-rate 1 ltc1qfmvk898k6jgfgp98dhsc5gvr9hpxl2ggd25ygk {txid}:0:0"
  ))
  .rpc_server(&rpc_server)
  .stdout_regex(r"[[:xdigit:]]{64}\n")
  .run_and_extract_stdout();
}

#[test]
fn send_addresses_must_be_valid_for_network() {
  let rpc_server = test_bitcoincore_rpc::builder().build();
  let txid = rpc_server.mine_blocks_with_subsidy(1, 1_000)[0].txdata[0].txid();
  create_wallet(&rpc_server);

  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 tltc1qfk58sxvnsy27ww6408qr3h7294anh7kqn8rn2r {txid}:0:0"
  ))
  .rpc_server(&rpc_server)
  .expected_stderr(
    "error: address tltc1qfk58sxvnsy27ww6408qr3h7294anh7kqn8rn2r belongs to network testnet which is different from required bitcoin\n",
  )
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn send_on_mainnnet_works_with_wallet_named_ord() {
  let rpc_server = test_bitcoincore_rpc::builder().build();
  let txid = rpc_server.mine_blocks_with_subsidy(1, 1_000_000)[0].txdata[0].txid();
  create_wallet(&rpc_server);

  let stdout = CommandBuilder::new(format!(
    "wallet send --fee-rate 1 ltc1qfmvk898k6jgfgp98dhsc5gvr9hpxl2ggd25ygk {txid}:0:0"
  ))
  .rpc_server(&rpc_server)
  .stdout_regex(r".*")
  .run_and_extract_stdout();

  let txid = rpc_server.mempool()[0].txid();
  assert_eq!(format!("{txid}\n"), stdout);
}

#[test]
fn send_does_not_use_inscribed_sats_as_cardinal_utxos() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let txid = rpc_server.mine_blocks_with_subsidy(1, 10_000)[0].txdata[0].txid();
  CommandBuilder::new(format!(
    "wallet inscribe --satpoint {txid}:0:0 degenerate.png --fee-rate 0"
  ))
  .write("degenerate.png", [1; 100])
  .rpc_server(&rpc_server)
  .run_and_check_output::<Inscribe>();

  let txid = rpc_server.mine_blocks_with_subsidy(1, 100)[0].txdata[0].txid();
  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 ltc1qfmvk898k6jgfgp98dhsc5gvr9hpxl2ggd25ygk {txid}:0:0"
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr("error: wallet does not contain enough cardinal UTXOs, please add additional funds to wallet.\n")
  .run_and_extract_stdout();
}

#[test]
fn do_not_accidentally_send_an_inscription() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let Inscribe {
    reveal,
    inscription,
    ..
  } = inscribe(&rpc_server);

  rpc_server.mine_blocks(1);

  let output = OutPoint {
    txid: reveal,
    vout: 0,
  };

  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {output}:55"
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: cannot send {output}:55 without also sending inscription {inscription} at {output}:0\n"
  ))
  .run_and_extract_stdout();
}

#[test]
fn inscriptions_cannot_be_sent_by_satpoint() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  let Inscribe { reveal, .. } = inscribe(&rpc_server);

  rpc_server.mine_blocks(1);

  CommandBuilder::new(format!(
    "wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {reveal}:0:0"
  ))
  .rpc_server(&rpc_server)
  .expected_stderr("error: inscriptions must be sent by inscription ID\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn send_btc() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  let output =
    CommandBuilder::new("wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 1btc")
      .rpc_server(&rpc_server)
      .run_and_check_output::<Output>();

  assert_eq!(
    output.transaction,
    "0000000000000000000000000000000000000000000000000000000000000000"
      .parse()
      .unwrap()
  );

  assert_eq!(
    rpc_server.sent(),
    &[Sent {
      amount: 1.0,
      address: "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
        .parse::<Address<NetworkUnchecked>>()
        .unwrap()
        .assume_checked(),
      locked: Vec::new(),
    }]
  )
}

#[test]
fn send_btc_locks_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  let Inscribe { reveal, .. } = inscribe(&rpc_server);

  let output =
    CommandBuilder::new("wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 1btc")
      .rpc_server(&rpc_server)
      .run_and_check_output::<Output>();

  assert_eq!(
    output.transaction,
    "0000000000000000000000000000000000000000000000000000000000000000"
      .parse()
      .unwrap()
  );

  assert_eq!(
    rpc_server.sent(),
    &[Sent {
      amount: 1.0,
      address: "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
        .parse::<Address<NetworkUnchecked>>()
        .unwrap()
        .assume_checked(),
      locked: vec![OutPoint {
        txid: reveal,
        vout: 0,
      }]
    }]
  )
}

#[test]
fn send_btc_fails_if_lock_unspent_fails() {
  let rpc_server = test_bitcoincore_rpc::builder()
    .fail_lock_unspent(true)
    .build();
  create_wallet(&rpc_server);

  rpc_server.mine_blocks(1);

  CommandBuilder::new("wallet send --fee-rate 1 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 1btc")
    .rpc_server(&rpc_server)
    .expected_stderr("error: failed to lock ordinal UTXOs\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn wallet_send_with_fee_rate() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let Inscribe { inscription, .. } = inscribe(&rpc_server);

  CommandBuilder::new(format!(
    "wallet send bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {inscription} --fee-rate 2.0"
  ))
  .rpc_server(&rpc_server)
  .stdout_regex("[[:xdigit:]]{64}\n")
  .run_and_extract_stdout();

  let tx = &rpc_server.mempool()[0];
  let mut fee = 0;
  for input in &tx.input {
    fee += rpc_server
      .get_utxo_amount(&input.previous_output)
      .unwrap()
      .to_sat();
  }
  for output in &tx.output {
    fee -= output.value;
  }

  let fee_rate = fee as f64 / tx.vsize() as f64;

  pretty_assert_eq!(fee_rate, 2.0);
}

#[test]
fn user_must_provide_fee_rate_to_send() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  create_wallet(&rpc_server);
  rpc_server.mine_blocks(1);

  let Inscribe { inscription, .. } = inscribe(&rpc_server);

  CommandBuilder::new(format!(
    "wallet send bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {inscription}"
  ))
  .rpc_server(&rpc_server)
  .expected_exit_code(2)
  .stderr_regex(
    ".*error: The following required arguments were not provided:
.*--fee-rate <FEE_RATE>.*",
  )
  .run_and_extract_stdout();
}

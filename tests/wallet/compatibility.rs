use super::*;

#[test]
fn core_21_wallet_creation_does_not_return_an_unused_mnemonic() {
  let core = mockcore::builder().version(210508).build();
  let output = CommandBuilder::new("wallet create")
    .core(&core)
    .run_and_deserialize_output::<serde_json::Value>();
  assert!(output.get("mnemonic").is_none());
  assert!(output["backup"].as_str().unwrap().contains("wallet.dat"));
  assert!(core.descriptors().is_empty());
}

#[test]
fn core_21_mnemonic_passphrase_is_rejected_before_wallet_creation() {
  let core = mockcore::builder().version(210508).build();
  CommandBuilder::new("wallet create --passphrase example")
    .core(&core)
    .expected_exit_code(1)
    .expected_stderr("error: mnemonic passphrases require Litecoin Core 24.0.0 or newer\n")
    .run_and_extract_stdout();
  assert!(core.state().wallets.is_empty());
}

#[test]
fn core_21_restore_is_rejected_before_reading_mnemonic() {
  let core = mockcore::builder().version(210508).build();
  CommandBuilder::new("wallet restore --from mnemonic")
    .core(&core)
    .expected_exit_code(1)
    .expected_stderr("error: descriptor and mnemonic restore requires Litecoin Core 24.0.0 or newer, current version is 21.5.8\n")
    .run_and_extract_stdout();
  assert!(core.state().wallets.is_empty());
}

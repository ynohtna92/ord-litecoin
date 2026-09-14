use super::*;

#[test]
fn verify() {
  assert_eq!(
    CommandBuilder::new([
      "verify",
      "--address", "ltc1q9vza2e8x573nczrlzms0wvx3gsqjx7vag5vzh0",
      "--message", "Hello World",
      "--witness", "AkcwRAIgZRfIY3p7/DoVTty6YZbWS71bc5Vct9p9Fia83eRmw2QCICK/ENGfwLtptFluMGs2KsqoNSk89pO7F29zJLUx9a/sASECx/EgAxlkQpQ9hYjgGu6EBCPMVPwVIVJqO4XCsMvViHI="
    ])
    .run_and_extract_stdout(),
    ""
  );
}

#[test]
fn verify_fails() {
  CommandBuilder::new([
      "verify",
      "--address", "ltc1q9vza2e8x573nczrlzms0wvx3gsqjx7vag5vzh0",
      "--message", "Hello World - this should fail",
      "--witness", "AkcwRAIgZRfIY3p7/DoVTty6YZbWS71bc5Vct9p9Fia83eRmw2QCICK/ENGfwLtptFluMGs2KsqoNSk89pO7F29zJLUx9a/sASECx/EgAxlkQpQ9hYjgGu6EBCPMVPwVIVJqO4XCsMvViHI="
  ])
  .expected_exit_code(1)
  .stderr_regex("error: Invalid signature.*")
  .run_and_extract_stdout();
}

#[test]
fn witness_and_transaction_conflict() {
  CommandBuilder::new([
      "verify",
      "--address", "ltc1q9vza2e8x573nczrlzms0wvx3gsqjx7vag5vzh0",
      "--message", "Hello World",
      "--transaction", "asdf",
      "--witness", "AkcwRAIgZRfIY3p7/DoVTty6YZbWS71bc5Vct9p9Fia83eRmw2QCICK/ENGfwLtptFluMGs2KsqoNSk89pO7F29zJLUx9a/sASECx/EgAxlkQpQ9hYjgGu6EBCPMVPwVIVJqO4XCsMvViHI="
  ])
  .expected_exit_code(2)
  .stderr_regex(".*error.*")
  .run_and_extract_stdout();
}

#[test]
fn verify_with_transaction() {
  let tx = bip322::sign_full_encoded(
    "ltc1q9vza2e8x573nczrlzms0wvx3gsqjx7vag5vzh0",
    "Hello World",
    "T9KX5zCugGdXizZsfv9st3xy98jRF53XdQt2WDrZ1mZUcQxUSprs",
  )
  .unwrap();

  assert_eq!(
    CommandBuilder::new([
      "verify",
      "--address",
      "ltc1q9vza2e8x573nczrlzms0wvx3gsqjx7vag5vzh0",
      "--message",
      "Hello World",
      "--transaction",
      &tx,
    ])
    .run_and_extract_stdout(),
    ""
  );
}

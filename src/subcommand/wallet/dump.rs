use super::*;

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  wallet.require_descriptors("descriptor dump")?;
  eprintln!(
    "==========================================
= THIS STRING CONTAINS YOUR PRIVATE KEYS =
=        DO NOT SHARE WITH ANYONE        =
=========================================="
  );

  Ok(Some(Box::new(
    wallet
      .bitcoin_client()
      .call::<ListDescriptorsResult>("listdescriptors", &[serde_json::to_value(true)?])?,
  )))
}

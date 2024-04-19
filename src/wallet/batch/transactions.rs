use super::*;

#[derive(Debug)]
pub(crate) struct Transactions {
  pub(crate) rune: Option<RuneInfo>,
  pub(crate) commit_tx: Transaction,
  pub(crate) commit_vout: usize,
  #[allow(dead_code)]
  pub(crate) recovery_key_pair: TweakedKeyPair,
  pub(crate) reveal_tx: Transaction,
  pub(crate) total_fees: u64,
}

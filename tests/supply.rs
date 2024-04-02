use super::*;

#[test]
fn genesis() {
  assert_eq!(
    CommandBuilder::new("supply").run_and_deserialize_output::<Supply>(),
    Supply {
      supply: 8399999990760000,
      first: 0,
      last: 8399999990759999,
      last_mined_in_block: 27719999
    }
  );
}

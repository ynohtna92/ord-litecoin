use {super::*, ord::subcommand::parse::Output, ord::Object};

#[test]
fn name() {
  assert_eq!(
    CommandBuilder::new("parse a").run_and_deserialize_output::<Output>(),
    Output {
      object: Object::Integer(8399999990759999),
    }
  );
}

#[test]
fn hash() {
  assert_eq!(
    CommandBuilder::new("parse 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
      .run_and_deserialize_output::<Output>(),
    Output {
      object: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        .parse::<Object>()
        .unwrap(),
    }
  );
}

#[test]
fn unrecognized_object() {
  CommandBuilder::new("parse Az")
    .stderr_regex(r"error: .*: Unrecognized representation.*")
    .expected_exit_code(2)
    .run_and_extract_stdout();
}

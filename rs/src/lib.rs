extern crate avm1_tree;
#[macro_use]
extern crate nom;

pub use self::avm1::{parse_action};

mod basic_data_types;
mod avm1;


#[cfg(test)]
mod parser_tests {
  use ::std::io::Read;

  use ::avm1_tree::Action;

  use ::test_generator::test_expand_paths;

  use super::*;

  test_expand_paths! { test_parse_action; "../tests/actions/*.avm1" }
  fn test_parse_action(path: &str) {
    let json_path: String = path.replace(".avm1", ".json");
    let mut input_file = ::std::fs::File::open(path).unwrap();
    let mut input: Vec<u8> = Vec::new();
    input_file.read_to_end(&mut input).expect("Unable to read file");

    let (remaining_input, actual_action) = parse_action(&input).unwrap();

    assert_eq!(remaining_input, &[] as &[u8]);

    let json_file = ::std::fs::File::open(json_path).unwrap();
    let reader = ::std::io::BufReader::new(json_file);
    let expected_action: Action = serde_json::from_reader(reader).unwrap();


    assert_eq!(actual_action, expected_action);
  }
}

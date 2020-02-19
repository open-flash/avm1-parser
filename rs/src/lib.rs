pub use crate::avm1::parse_action;
pub use crate::cfg::parse_cfg;

mod avm1;
mod basic_data_types;
mod cfg;

#[cfg(test)]
mod parser_tests {
  use super::*;
  use ::avm1_types as avm1;
  use ::test_generator::test_resources;
  use avm1_types::cfg::Cfg;
  use std::io::{Read, Write};
  use std::path::Path;

  #[test_resources("../tests/actions/*.avm1")]
  fn test_parse_action(path: &str) {
    let json_path: String = path.replace(".avm1", ".json");
    let mut input_file = ::std::fs::File::open(path).unwrap();
    let mut input: Vec<u8> = Vec::new();
    input_file.read_to_end(&mut input).expect("Failed to read AVM1 file");

    let (remaining_input, actual_action) = parse_action(&input).unwrap();

    assert_eq!(remaining_input, &[] as &[u8]);

    let json_file = ::std::fs::File::open(json_path).unwrap();
    let reader = ::std::io::BufReader::new(json_file);
    let expected_action: avm1::raw::Action = serde_json_v8::from_reader(reader).unwrap();

    assert_eq!(actual_action, expected_action);
  }

  #[test_resources("../tests/avm1/[!.]*/*/")]
  fn test_parse_cfg(path: &str) {
    use serde::Serialize;

    let path: &Path = Path::new(path);
    let _name = path
      .components()
      .last()
      .unwrap()
      .as_os_str()
      .to_str()
      .expect("Failed to retrieve sample name");

    //    if name == "hello-world" || name == "homestuck-beta2" {
    //      return;
    //    }

    let avm1_path = path.join("main.avm1");
    let avm1_bytes: Vec<u8> = ::std::fs::read(avm1_path).expect("Failed to read input");

    let actual_cfg = parse_cfg(&avm1_bytes);

    let actual_cfg_path = path.join("local-cfg.rs.json");
    let actual_cfg_file = ::std::fs::File::create(actual_cfg_path).expect("Failed to create actual CFG file");
    let actual_cfg_writer = ::std::io::BufWriter::new(actual_cfg_file);

    let mut ser = serde_json_v8::Serializer::pretty(actual_cfg_writer);
    actual_cfg.serialize(&mut ser).expect("Failed to write actual CFG");
    ser.into_inner().write_all(b"\n").unwrap();

    // assert_eq!(remaining_input, &[] as &[u8]);

    let expected_cfg_path = path.join("cfg.json");
    let expected_cfg_file = ::std::fs::File::open(expected_cfg_path).expect("Failed to open CFG");
    let expected_cfg_reader = ::std::io::BufReader::new(expected_cfg_file);
    let expected_cfg = serde_json_v8::from_reader::<_, Cfg>(expected_cfg_reader).expect("Failed to read CFG");

    assert_eq!(actual_cfg, expected_cfg);
  }
}

//struct Node<'a> {
//  pub parent: Option<&'a Node<'a>>,
//  pub value: u32,
//}
//
////fn root_node(value: u32) -> Node<'static> {
////  Node { parent: None, value }
////}
////
//fn child_node<'inner, 'outer: 'inner>(parent: &'inner mut Node<'outer>, value: u32) -> Node<'inner> {
//  Node { parent: Some(parent), value }
//}
//
//impl Node<'_> {
//  fn root(value: u32) -> Node<'static> {
//    Node { parent: None, value }
//  }
//}
//
//impl<'inner, 'outer: 'inner> Node<'outer> {
//  fn child(&'inner mut self, value: u32) -> Node<'inner> {
//    Node { parent: Some(self), value }
//  }
//}
//
//pub fn main() {
//  let mut root = Node::root(0);
//  let mut c1 = root.child(1);
//  let mut c2 = child_node(&mut c1, 2);
//  {
//    let mut c3 = child_node(&mut c2, 3);
//    let c4 = child_node(&mut c3, 4);
//    let mut cur = Some(&c4);
//    while let Some(n) = cur {
//      println!("{}", n.value);
//      cur = n.parent;
//    }
//  }
//  println!("{}", c2.value);
//}

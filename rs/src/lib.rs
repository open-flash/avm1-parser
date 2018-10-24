extern crate avm1_tree;
#[macro_use]
extern crate nom;
#[macro_use]
extern crate serde_derive;

pub mod basic_data_types;
mod avm1;


//#[cfg(test)]
//mod tests {
//  use nom::{IResult, Needed};
//  use super::*;
//  use super::basic_data_types::{parse_bool_bits, skip_bits};
//
//  #[test]
//  fn test_parse_encoded_le_u32() {
//    fn tb(input: (&[u8], usize), n: usize) -> IResult<(&[u8], usize), ()> {
//      skip_bits(input, n)
//    }
//    fn t4b(input: (&[u8], usize)) -> IResult<(&[u8], usize), ()> {
//      tb(input, 4)
//    }
//
////    fn take_4_bits(input: &[u8]) -> IResult<&[u8], (bool, bool, bool)> {
////      bits!(
////        input,
////        do_parse!(
////          catch_in_register: parse_bool_bits >>
////          finally_block: parse_bool_bits >>
////          catch_block: parse_bool_bits >>
////          ((catch_in_register, catch_block, finally_block))
////        )
////      )
////    }
//
////    named!( take_4_bits<u8>, bits!( take_bits!(u8, 4) ) );
//    named!( take_4_bits<()>, bits!( t4b ) );
////    named!( take_4_bits<u8>, bits!( apply!(tb, 4) ) );
////    named!( take_4_bits<u8>, bits!( do_parse! ( n: apply!(tb, 4) >> (n+1) ) ) );
//
//    let input = vec![0xAB, 0xCD, 0xEF, 0x12];
//    let sl = &input[..];
//
//    assert_eq!(take_4_bits(sl), Ok((&sl[1..], ())));
//  }
//}

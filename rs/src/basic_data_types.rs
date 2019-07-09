use nom::{IResult as NomResult, Needed};

/// Parse the bit-encoded representation of a bool (1 bit)
pub fn parse_bool_bits((input_slice, bit_pos): (&[u8], usize)) -> NomResult<(&[u8], usize), bool> {
  if input_slice.len() < 1 {
    Err(::nom::Err::Incomplete(Needed::Size(1)))
  } else {
    let res: bool = input_slice[0] & (1 << (7 - bit_pos)) > 0;
    if bit_pos == 7 {
      Ok(((&input_slice[1..], 0), res))
    } else {
      Ok(((input_slice, bit_pos + 1), res))
    }
  }
}

/// Parse a null-terminated sequence of bytes. The null byte is consumed but not included in the
/// result.
pub fn parse_c_string(input: &[u8]) -> NomResult<&[u8], String> {
  map!(input, take_until_and_consume!("\x00"), |str: &[u8]| String::from_utf8(
    str.to_vec()
  )
  .unwrap())
}

/// Skip `n` bits
pub fn skip_bits((input_slice, bit_pos): (&[u8], usize), n: usize) -> NomResult<(&[u8], usize), ()> {
  let slice_len: usize = input_slice.len();
  let available_bits: usize = 8 * slice_len - bit_pos;
  let skipped_full_bytes = (bit_pos + n) / 8;
  let final_bit_pos = (bit_pos + n) % 8;
  if available_bits < n {
    let needed_bytes = skipped_full_bytes + if final_bit_pos > 0 { 1 } else { 0 };
    Err(::nom::Err::Incomplete(Needed::Size(needed_bytes)))
  } else {
    Ok(((&input_slice[skipped_full_bytes..], final_bit_pos), ()))
  }
}

use nom::number::complete::le_u64 as parse_le_u64;
use nom::IResult as NomResult;

/// Parse a null-terminated sequence of bytes. The nul-byte is consumed but not included in the
/// result.
pub(crate) fn parse_c_string(input: &[u8]) -> NomResult<&[u8], String> {
  const NUL_BYTE: &[u8] = b"\x00";

  let (input, raw) = nom::bytes::streaming::take_until(NUL_BYTE)(input)?;
  let (input, _) = nom::bytes::streaming::take(NUL_BYTE.len())(input)?;

  match std::str::from_utf8(raw) {
    Ok(checked) => Ok((input, checked.to_string())),
    Err(_) => Err(nom::Err::Error(nom::error::Error::new(
      input,
      nom::error::ErrorKind::Verify,
    ))),
  }
}

pub(crate) fn parse_le32_f64(input: &[u8]) -> NomResult<&[u8], f64> {
  let (input, bits) = parse_le_u64(input)?;
  let bits = (bits >> 32) | (bits << 32);
  let bytes = bits.to_le_bytes();
  Ok((input, f64::from_le_bytes(bytes)))
}

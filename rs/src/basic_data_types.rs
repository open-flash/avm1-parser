use nom::IResult as NomResult;

/// Parse a null-terminated sequence of bytes. The nul-byte is consumed but not included in the
/// result.
pub(crate) fn parse_c_string(input: &[u8]) -> NomResult<&[u8], String> {
  const NUL_BYTE: &[u8] = b"\x00";

  let (input, raw) = nom::bytes::streaming::take_until(NUL_BYTE)(input)?;
  let (input, _) = nom::bytes::streaming::take(NUL_BYTE.len())(input)?;

  match std::str::from_utf8(raw) {
    Ok(checked) => Ok((input, checked.to_string())),
    Err(_) => Err(nom::Err::Error((input, nom::error::ErrorKind::Verify))),
  }
}

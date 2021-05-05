use crate::basic_data_types::{parse_c_string, parse_le32_f64};
use avm1_types as avm1;
use avm1_types::raw;
use nom::number::complete::{
  le_f32 as parse_le_f32, le_i16 as parse_le_i16, le_i32 as parse_le_i32, le_u16 as parse_le_u16, le_u8 as parse_u8,
};
use nom::{IResult as NomResult, Needed};
use std::convert::TryFrom;

#[derive(Debug, PartialEq, Eq)]
pub struct ActionHeader {
  pub code: u8,
  pub length: usize,
}

// TODO: Use nom::cond
pub fn parse_action_header(input: &[u8]) -> NomResult<&[u8], ActionHeader> {
  match parse_u8(input) {
    Ok((remaining_input, action_code)) => {
      if action_code < 0x80 {
        Ok((
          remaining_input,
          ActionHeader {
            code: action_code,
            length: 0,
          },
        ))
      } else {
        parse_le_u16(remaining_input).map(|(i, length)| {
          (
            i,
            ActionHeader {
              code: action_code,
              length: length as usize,
            },
          )
        })
      }
    }
    Err(e) => Err(e),
  }
}

pub fn parse_goto_frame_action(input: &[u8]) -> NomResult<&[u8], raw::GotoFrame> {
  let (input, frame) = parse_le_u16(input)?;
  Ok((input, raw::GotoFrame { frame }))
}

pub fn parse_get_url_action(input: &[u8]) -> NomResult<&[u8], raw::GetUrl> {
  let (input, url) = parse_c_string(input)?;
  let (input, target) = parse_c_string(input)?;
  Ok((input, raw::GetUrl { url, target }))
}

pub fn parse_store_register_action(input: &[u8]) -> NomResult<&[u8], raw::StoreRegister> {
  let (input, register) = parse_u8(input)?;
  Ok((input, raw::StoreRegister { register }))
}

pub fn parse_strict_mode_action(input: &[u8]) -> NomResult<&[u8], raw::StrictMode> {
  let (input, is_strict) = parse_u8(input)?;
  Ok((
    input,
    raw::StrictMode {
      is_strict: is_strict != 0,
    },
  ))
}

pub fn parse_constant_pool_action(input: &[u8]) -> NomResult<&[u8], raw::ConstantPool> {
  use nom::multi::count;
  let (input, const_count) = parse_le_u16(input)?;
  let (input, pool) = count(parse_c_string, usize::from(const_count))(input)?;
  Ok((input, raw::ConstantPool { pool }))
}

pub fn parse_wait_for_frame_action(input: &[u8]) -> NomResult<&[u8], raw::WaitForFrame> {
  let (input, frame) = parse_le_u16(input)?;
  let (input, skip) = parse_u8(input)?;
  Ok((input, raw::WaitForFrame { frame, skip }))
}

pub fn parse_set_target_action(input: &[u8]) -> NomResult<&[u8], raw::SetTarget> {
  let (input, target_name) = parse_c_string(input)?;
  Ok((input, raw::SetTarget { target_name }))
}

pub fn parse_goto_label_action(input: &[u8]) -> NomResult<&[u8], raw::GoToLabel> {
  let (input, label) = parse_c_string(input)?;
  Ok((input, raw::GoToLabel { label }))
}

pub fn parse_wait_for_frame2_action(input: &[u8]) -> NomResult<&[u8], raw::WaitForFrame2> {
  let (input, skip) = parse_u8(input)?;
  Ok((input, raw::WaitForFrame2 { skip }))
}

#[derive(Debug, PartialEq, Eq)]
struct DefineFunction2Flags {
  pub preload_parent: bool,
  pub preload_root: bool,
  pub suppress_super: bool,
  pub preload_super: bool,
  pub suppress_arguments: bool,
  pub preload_arguments: bool,
  pub suppress_this: bool,
  pub preload_this: bool,
  pub preload_global: bool,
}

// TODO(demurgos): registerCount

pub fn parse_define_function2_action(input: &[u8]) -> NomResult<&[u8], raw::DefineFunction2> {
  use nom::multi::count;

  let (input, name) = parse_c_string(input)?;
  let (input, parameter_count) = parse_le_u16(input)?;
  let (input, register_count) = parse_u8(input)?;

  let (input, flags) = parse_le_u16(input)?;
  let preload_this = (flags & (1 << 0)) != 0;
  let suppress_this = (flags & (1 << 1)) != 0;
  let preload_arguments = (flags & (1 << 2)) != 0;
  let suppress_arguments = (flags & (1 << 3)) != 0;
  let preload_super = (flags & (1 << 4)) != 0;
  let suppress_super = (flags & (1 << 5)) != 0;
  let preload_root = (flags & (1 << 6)) != 0;
  let preload_parent = (flags & (1 << 7)) != 0;
  let preload_global = (flags & (1 << 8)) != 0;
  // Skip bits [9, 15]

  let (input, parameters) = count(parse_parameter, usize::from(parameter_count))(input)?;
  fn parse_parameter(input: &[u8]) -> NomResult<&[u8], avm1::Parameter> {
    let (input, register) = parse_u8(input)?;
    let (input, name) = parse_c_string(input)?;
    Ok((input, avm1::Parameter { register, name }))
  }

  let (input, body_size) = parse_le_u16(input)?;

  Ok((
    input,
    raw::DefineFunction2 {
      name,
      preload_this,
      suppress_this,
      preload_arguments,
      suppress_arguments,
      preload_super,
      suppress_super,
      preload_root,
      preload_parent,
      preload_global,
      register_count,
      parameters,
      body_size,
    },
  ))
}

pub fn parse_try_action(input: &[u8]) -> NomResult<&[u8], raw::Try> {
  let (input, flags) = parse_u8(input)?;
  let has_catch_block = (flags & (1 << 0)) != 0;
  let has_finally_block = (flags & (1 << 1)) != 0;
  let catch_in_register = (flags & (1 << 2)) != 0;
  // Skip bits [3, 7]

  let (input, r#try) = parse_le_u16(input)?;
  let (input, catch_size) = parse_le_u16(input)?;
  let (input, finally_size) = parse_le_u16(input)?;

  let (input, catch_target) = parse_catch_target(input, catch_in_register)?;
  fn parse_catch_target(input: &[u8], catch_in_register: bool) -> NomResult<&[u8], avm1::CatchTarget> {
    use nom::combinator::map;
    if catch_in_register {
      map(parse_u8, avm1::CatchTarget::Register)(input)
    } else {
      map(parse_c_string, avm1::CatchTarget::Variable)(input)
    }
  }

  let catch: Option<raw::CatchBlock> = if has_catch_block {
    Some(raw::CatchBlock {
      target: catch_target,
      size: catch_size,
    })
  } else {
    None
  };

  let finally = if has_finally_block { Some(finally_size) } else { None };

  Ok((input, raw::Try { r#try, catch, finally }))
}

pub fn parse_with_action(input: &[u8]) -> NomResult<&[u8], raw::With> {
  let (input, size) = parse_le_u16(input)?;

  Ok((input, raw::With { size }))
}

pub fn parse_push_action(mut input: &[u8]) -> NomResult<&[u8], raw::Push> {
  let mut values: Vec<avm1::PushValue> = Vec::new();
  while !input.is_empty() {
    let (next_input, value) = parse_push_value(input)?;
    values.push(value);
    input = next_input;
  }
  Ok((input, raw::Push { values }))
}

fn parse_push_value(input: &[u8]) -> NomResult<&[u8], avm1::PushValue> {
  use nom::combinator::map;
  let (input, code) = parse_u8(input)?;
  match code {
    0 => map(parse_c_string, avm1::PushValue::String)(input),
    1 => map(parse_le_f32, avm1::PushValue::Float32)(input),
    2 => Ok((input, avm1::PushValue::Null)),
    3 => Ok((input, avm1::PushValue::Undefined)),
    4 => map(parse_u8, avm1::PushValue::Register)(input),
    5 => map(parse_u8, |v| avm1::PushValue::Boolean(v != 0))(input),
    6 => map(parse_le32_f64, avm1::PushValue::Float64)(input),
    7 => map(parse_le_i32, avm1::PushValue::Sint32)(input),
    8 => map(parse_u8, |v| avm1::PushValue::Constant(u16::from(v)))(input),
    9 => map(parse_le_u16, avm1::PushValue::Constant)(input),
    _ => Err(nom::Err::Error((input, nom::error::ErrorKind::Switch))),
  }
}

pub fn parse_jump_action(input: &[u8]) -> NomResult<&[u8], raw::Jump> {
  let (input, offset) = parse_le_i16(input)?;
  Ok((input, raw::Jump { offset }))
}

pub fn parse_get_url2_action(input: &[u8]) -> NomResult<&[u8], raw::GetUrl2> {
  let (input, flags) = parse_u8(input)?;
  let load_variables = (flags & (1 << 0)) != 0;
  let load_target = (flags & (1 << 1)) != 0;
  // Skip bits [2, 5]
  let method_code = flags >> 6;

  let method = match method_code {
    0 => avm1::GetUrl2Method::None,
    1 => avm1::GetUrl2Method::Get,
    2 => avm1::GetUrl2Method::Post,
    _ => return Err(nom::Err::Error((input, nom::error::ErrorKind::Switch))),
  };

  Ok((
    input,
    raw::GetUrl2 {
      method,
      load_target,
      load_variables,
    },
  ))
}

pub fn parse_define_function_action(input: &[u8]) -> NomResult<&[u8], raw::DefineFunction> {
  use nom::multi::count;
  let (input, name) = parse_c_string(input)?;
  let (input, param_count) = parse_le_u16(input)?;
  let (input, parameters) = count(parse_c_string, param_count.into())(input)?;
  let (input, body_size) = parse_le_u16(input)?;

  Ok((
    input,
    raw::DefineFunction {
      name,
      parameters,
      body_size,
    },
  ))
}

pub fn parse_if_action(input: &[u8]) -> NomResult<&[u8], raw::If> {
  let (input, offset) = parse_le_i16(input)?;
  Ok((input, raw::If { offset }))
}

pub fn parse_goto_frame2_action(input: &[u8]) -> NomResult<&[u8], raw::GotoFrame2> {
  use nom::combinator::cond;
  let (input, flags) = parse_u8(input)?;
  let play = (flags & (1 << 0)) != 0;
  let has_scene_bias = (flags & (1 << 1)) != 0;
  // Skip bits [2, 7]
  let (input, scene_bias) = cond(has_scene_bias, parse_le_u16)(input)?;

  Ok((
    input,
    raw::GotoFrame2 {
      play,
      scene_bias: scene_bias.unwrap_or_default(),
    },
  ))
}

// TODO: Return `(&[u8], ast::Action)` (the function should never fail)
pub fn parse_action(input: &[u8]) -> NomResult<&[u8], raw::Action> {
  let base_input = input; // Keep original input to compute lengths.

  let (input, header) = parse_action_header(input)?;

  let body_len = usize::try_from(header.length).unwrap();
  if input.len() < body_len {
    let header_len = base_input.len() - input.len();
    let action_len = header_len + body_len;
    return Err(nom::Err::Incomplete(Needed::Size(action_len)));
  }
  let (action_body, input) = input.split_at(body_len);
  let action = parse_action_body(action_body, header.code);
  Ok((input, action))
}

fn parse_action_body(input: &[u8], code: u8) -> raw::Action {
  use nom::combinator::map;
  let result = match code {
    0x00 => Ok((input, raw::Action::End)),
    0x04 => Ok((input, raw::Action::NextFrame)),
    0x05 => Ok((input, raw::Action::PrevFrame)),
    0x06 => Ok((input, raw::Action::Play)),
    0x07 => Ok((input, raw::Action::Stop)),
    0x08 => Ok((input, raw::Action::ToggleQuality)),
    0x09 => Ok((input, raw::Action::StopSounds)),
    0x0a => Ok((input, raw::Action::Add)),
    0x0b => Ok((input, raw::Action::Subtract)),
    0x0c => Ok((input, raw::Action::Multiply)),
    0x0d => Ok((input, raw::Action::Divide)),
    0x0e => Ok((input, raw::Action::Equals)),
    0x0f => Ok((input, raw::Action::Less)),
    0x10 => Ok((input, raw::Action::And)),
    0x11 => Ok((input, raw::Action::Or)),
    0x12 => Ok((input, raw::Action::Not)),
    0x13 => Ok((input, raw::Action::StringEquals)),
    0x14 => Ok((input, raw::Action::StringLength)),
    0x15 => Ok((input, raw::Action::StringExtract)),
    0x17 => Ok((input, raw::Action::Pop)),
    0x18 => Ok((input, raw::Action::ToInteger)),
    0x1c => Ok((input, raw::Action::GetVariable)),
    0x1d => Ok((input, raw::Action::SetVariable)),
    0x20 => Ok((input, raw::Action::SetTarget2)),
    0x21 => Ok((input, raw::Action::StringAdd)),
    0x22 => Ok((input, raw::Action::GetProperty)),
    0x23 => Ok((input, raw::Action::SetProperty)),
    0x24 => Ok((input, raw::Action::CloneSprite)),
    0x25 => Ok((input, raw::Action::RemoveSprite)),
    0x26 => Ok((input, raw::Action::Trace)),
    0x27 => Ok((input, raw::Action::StartDrag)),
    0x28 => Ok((input, raw::Action::EndDrag)),
    0x29 => Ok((input, raw::Action::StringLess)),
    0x2a => Ok((input, raw::Action::Throw)),
    0x2b => Ok((input, raw::Action::CastOp)),
    0x2c => Ok((input, raw::Action::ImplementsOp)),
    0x2d => Ok((input, raw::Action::FsCommand2)),
    0x30 => Ok((input, raw::Action::RandomNumber)),
    0x31 => Ok((input, raw::Action::MbStringLength)),
    0x32 => Ok((input, raw::Action::CharToAscii)),
    0x33 => Ok((input, raw::Action::AsciiToChar)),
    0x34 => Ok((input, raw::Action::GetTime)),
    0x35 => Ok((input, raw::Action::MbStringExtract)),
    0x36 => Ok((input, raw::Action::MbCharToAscii)),
    0x37 => Ok((input, raw::Action::MbAsciiToChar)),
    0x3a => Ok((input, raw::Action::Delete)),
    0x3b => Ok((input, raw::Action::Delete2)),
    0x3c => Ok((input, raw::Action::DefineLocal)),
    0x3d => Ok((input, raw::Action::CallFunction)),
    0x3e => Ok((input, raw::Action::Return)),
    0x3f => Ok((input, raw::Action::Modulo)),
    0x40 => Ok((input, raw::Action::NewObject)),
    0x41 => Ok((input, raw::Action::DefineLocal2)),
    0x42 => Ok((input, raw::Action::InitArray)),
    0x43 => Ok((input, raw::Action::InitObject)),
    0x44 => Ok((input, raw::Action::TypeOf)),
    0x45 => Ok((input, raw::Action::TargetPath)),
    0x46 => Ok((input, raw::Action::Enumerate)),
    0x47 => Ok((input, raw::Action::Add2)),
    0x48 => Ok((input, raw::Action::Less2)),
    0x49 => Ok((input, raw::Action::Equals2)),
    0x4a => Ok((input, raw::Action::ToNumber)),
    0x4b => Ok((input, raw::Action::ToString)),
    0x4c => Ok((input, raw::Action::PushDuplicate)),
    0x4d => Ok((input, raw::Action::StackSwap)),
    0x4e => Ok((input, raw::Action::GetMember)),
    0x4f => Ok((input, raw::Action::SetMember)),
    0x50 => Ok((input, raw::Action::Increment)),
    0x51 => Ok((input, raw::Action::Decrement)),
    0x52 => Ok((input, raw::Action::CallMethod)),
    0x53 => Ok((input, raw::Action::NewMethod)),
    0x54 => Ok((input, raw::Action::InstanceOf)),
    0x55 => Ok((input, raw::Action::Enumerate2)),
    0x60 => Ok((input, raw::Action::BitAnd)),
    0x61 => Ok((input, raw::Action::BitOr)),
    0x62 => Ok((input, raw::Action::BitXor)),
    0x63 => Ok((input, raw::Action::BitLShift)),
    0x64 => Ok((input, raw::Action::BitRShift)),
    0x65 => Ok((input, raw::Action::BitURShift)),
    0x66 => Ok((input, raw::Action::StrictEquals)),
    0x67 => Ok((input, raw::Action::Greater)),
    0x68 => Ok((input, raw::Action::StringGreater)),
    0x69 => Ok((input, raw::Action::Extends)),
    0x81 => map(parse_goto_frame_action, raw::Action::GotoFrame)(input),
    0x83 => map(parse_get_url_action, raw::Action::GetUrl)(input),
    0x87 => map(parse_store_register_action, raw::Action::StoreRegister)(input),
    0x88 => map(parse_constant_pool_action, raw::Action::ConstantPool)(input),
    0x89 => map(parse_strict_mode_action, raw::Action::StrictMode)(input),
    0x8a => map(parse_wait_for_frame_action, raw::Action::WaitForFrame)(input),
    0x8b => map(parse_set_target_action, raw::Action::SetTarget)(input),
    0x8c => map(parse_goto_label_action, raw::Action::GotoLabel)(input),
    0x8d => map(parse_wait_for_frame2_action, raw::Action::WaitForFrame2)(input),
    0x8e => map(parse_define_function2_action, raw::Action::DefineFunction2)(input),
    0x8f => map(parse_try_action, raw::Action::Try)(input),
    0x94 => map(parse_with_action, raw::Action::With)(input),
    0x96 => map(parse_push_action, raw::Action::Push)(input),
    0x99 => map(parse_jump_action, raw::Action::Jump)(input),
    0x9a => map(parse_get_url2_action, raw::Action::GetUrl2)(input),
    0x9b => map(parse_define_function_action, raw::Action::DefineFunction)(input),
    0x9d => map(parse_if_action, raw::Action::If)(input),
    0x9e => Ok((input, raw::Action::Call)),
    0x9f => map(parse_goto_frame2_action, raw::Action::GotoFrame2)(input),
    _ => Ok((
      &[][..],
      raw::Action::Raw(raw::Raw {
        code,
        data: input.to_vec(),
      }),
    )),
  };
  match result {
    Ok((_, action)) => action,
    Err(_) => raw::Action::Error(raw::Error { error: None }),
  }
}

#[cfg(test)]
mod tests {
  use nom;

  use super::*;
  use avm1_types::PushValue;

  #[test]
  fn test_parse_push_action() {
    {
      let input = vec![0x04, 0x00, 0x07, 0x01, 0x00, 0x00, 0x00, 0x08, 0x02];
      let actual = parse_push_action(&input[..]);
      let expected = Ok((
        &[][..],
        raw::Push {
          values: vec![PushValue::Register(0), PushValue::Sint32(1), PushValue::Constant(2)],
        },
      ));
      assert_eq!(actual, expected);
    }
    {
      let input = vec![0x00, 0x00];
      let actual = parse_push_action(&input[..]);
      let expected = Ok((
        &[][..],
        raw::Push {
          values: vec![PushValue::String(String::from(""))],
        },
      ));
      assert_eq!(actual, expected);
    }
    {
      let input = vec![0x00, 0x01, 0x00];
      let actual = parse_push_action(&input[..]);
      let expected = Ok((
        &[][..],
        raw::Push {
          values: vec![PushValue::String(String::from("\x01"))],
        },
      ));
      assert_eq!(actual, expected);
    }
  }

  #[test]
  fn test_parse_action_header() {
    {
      let input = vec![0b00000000, 0b00000000, 0b00000000, 0b00000000];
      assert_eq!(
        parse_action_header(&input[..]),
        Ok((&input[1..], ActionHeader { code: 0x00, length: 0 }))
      );
    }
    {
      let input = vec![0b00000001, 0b00000000, 0b00000000, 0b00000000];
      assert_eq!(
        parse_action_header(&input[..]),
        Ok((&input[1..], ActionHeader { code: 0x01, length: 0 }))
      );
    }
    {
      let input = vec![0b00010000, 0b00000000, 0b00000000, 0b00000000];
      assert_eq!(
        parse_action_header(&input[..]),
        Ok((&input[1..], ActionHeader { code: 0x10, length: 0 }))
      );
    }
    {
      let input = vec![0b10000000, 0b00000000, 0b00000000, 0b00000000];
      assert_eq!(
        parse_action_header(&input[..]),
        Ok((&input[3..], ActionHeader { code: 0x80, length: 0 }))
      );
    }
    {
      let input = vec![0b10000000, 0b00000001, 0b00000000, 0b00000000];
      assert_eq!(
        parse_action_header(&input[..]),
        Ok((&input[3..], ActionHeader { code: 0x80, length: 1 }))
      );
    }
    {
      let input = vec![0b10000000, 0b00000000, 0b00000001, 0b00000000];
      assert_eq!(
        parse_action_header(&input[..]),
        Ok((
          &input[3..],
          ActionHeader {
            code: 0x80,
            length: 256,
          }
        ))
      );
    }
  }

  #[test]
  fn test_parse_action() {
    {
      let input = vec![0b00000001, 0b00000000, 0b00000000, 0b00000000];
      assert_eq!(
        parse_action(&input),
        Ok((
          &input[1..],
          raw::Action::Raw(raw::Raw {
            code: 0x01,
            data: Vec::new(),
          })
        ))
      );
    }
    {
      let input = vec![0b10000000, 0b00000001, 0b00000000, 0b00000011];
      assert_eq!(
        parse_action(&input[..]),
        Ok((
          &input[4..],
          raw::Action::Raw(raw::Raw {
            code: 0x80,
            data: vec![0x03],
          })
        ))
      );
    }
    {
      let input = vec![0b10000000, 0b00000010, 0b00000000, 0b00000011];
      assert_eq!(
        parse_action(&input[..]),
        Err(::nom::Err::Incomplete(nom::Needed::Size(5)))
      );
    }
  }
}

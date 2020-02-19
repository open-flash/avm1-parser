use crate::basic_data_types::parse_c_string;
use avm1_types as ast;
use nom::number::complete::{
  le_f32 as parse_le_f32, le_f64 as parse_le_f64, le_i16 as parse_le_i16, le_i32 as parse_le_i32,
  le_u16 as parse_le_u16, le_u8 as parse_u8,
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

pub fn parse_goto_frame_action(input: &[u8]) -> NomResult<&[u8], ast::actions::GotoFrame> {
  let (input, frame) = parse_le_u16(input)?;
  Ok((
    input,
    ast::actions::GotoFrame {
      frame: usize::from(frame),
    },
  ))
}

pub fn parse_get_url_action(input: &[u8]) -> NomResult<&[u8], ast::actions::GetUrl> {
  let (input, url) = parse_c_string(input)?;
  let (input, target) = parse_c_string(input)?;
  Ok((input, ast::actions::GetUrl { url, target }))
}

pub fn parse_store_register_action(input: &[u8]) -> NomResult<&[u8], ast::actions::StoreRegister> {
  let (input, register) = parse_u8(input)?;
  Ok((input, ast::actions::StoreRegister { register }))
}

pub fn parse_constant_pool_action(input: &[u8]) -> NomResult<&[u8], ast::actions::ConstantPool> {
  use nom::multi::count;
  let (input, const_count) = parse_le_u16(input)?;
  let (input, constant_pool) = count(parse_c_string, usize::from(const_count))(input)?;
  Ok((input, ast::actions::ConstantPool { constant_pool }))
}

pub fn parse_wait_for_frame_action(input: &[u8]) -> NomResult<&[u8], ast::actions::WaitForFrame> {
  let (input, frame) = parse_le_u16(input)?;
  let (input, skip_count) = parse_u8(input)?;
  Ok((
    input,
    ast::actions::WaitForFrame {
      frame: usize::from(frame),
      skip_count: usize::from(skip_count),
    },
  ))
}

pub fn parse_set_target_action(input: &[u8]) -> NomResult<&[u8], ast::actions::SetTarget> {
  let (input, target_name) = parse_c_string(input)?;
  Ok((input, ast::actions::SetTarget { target_name }))
}

pub fn parse_goto_label_action(input: &[u8]) -> NomResult<&[u8], ast::actions::GoToLabel> {
  let (input, label) = parse_c_string(input)?;
  Ok((input, ast::actions::GoToLabel { label }))
}

pub fn parse_wait_for_frame2_action(input: &[u8]) -> NomResult<&[u8], ast::actions::WaitForFrame2> {
  let (input, skip_count) = parse_u8(input)?;
  Ok((
    input,
    ast::actions::WaitForFrame2 {
      skip_count: usize::from(skip_count),
    },
  ))
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

pub fn parse_define_function2_action(input: &[u8]) -> NomResult<&[u8], ast::actions::DefineFunction2> {
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
  fn parse_parameter(input: &[u8]) -> NomResult<&[u8], ast::actions::define_function2::Parameter> {
    let (input, register) = parse_u8(input)?;
    let (input, name) = parse_c_string(input)?;
    Ok((input, ast::actions::define_function2::Parameter { register, name }))
  }

  let (input, body_size) = parse_le_u16(input)?;

  Ok((
    input,
    ast::actions::DefineFunction2 {
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

pub fn parse_try_action(input: &[u8]) -> NomResult<&[u8], ast::actions::Try> {
  let (input, flags) = parse_u8(input)?;
  let has_catch_block = (flags & (1 << 0)) != 0;
  let has_finally_block = (flags & (1 << 1)) != 0;
  let catch_in_register = (flags & (1 << 2)) != 0;
  // Skip bits [3, 7]

  let (input, try_size) = parse_le_u16(input)?;
  let (input, catch_size) = parse_le_u16(input)?;
  let (input, finally_size) = parse_le_u16(input)?;

  let (input, catch_target) = parse_catch_target(input, catch_in_register)?;
  fn parse_catch_target(input: &[u8], catch_in_register: bool) -> NomResult<&[u8], ast::actions::r#try::CatchTarget> {
    use nom::combinator::map;
    if catch_in_register {
      map(parse_u8, ast::actions::r#try::CatchTarget::Register)(input)
    } else {
      map(parse_c_string, ast::actions::r#try::CatchTarget::Variable)(input)
    }
  }

  Ok((
    input,
    ast::actions::Try {
      try_size,
      catch_target,
      catch_size: if has_catch_block { Some(catch_size) } else { None },
      finally_size: if has_finally_block { Some(finally_size) } else { None },
    },
  ))
}

pub fn parse_with_action(input: &[u8]) -> NomResult<&[u8], ast::actions::With> {
  let (input, with_size) = parse_le_u16(input)?;

  Ok((input, ast::actions::With { with_size }))
}

pub fn parse_push_action(mut input: &[u8]) -> NomResult<&[u8], ast::actions::Push> {
  let mut values: Vec<ast::Value> = Vec::new();
  while !input.is_empty() {
    let (next_input, value) = parse_action_value(input)?;
    values.push(value);
    input = next_input;
  }
  Ok((input, ast::actions::Push { values }))
}

fn parse_action_value(input: &[u8]) -> NomResult<&[u8], ast::Value> {
  use nom::combinator::map;
  let (input, code) = parse_u8(input)?;
  match code {
    0 => map(parse_c_string, ast::Value::String)(input),
    1 => map(parse_le_f32, ast::Value::Float32)(input),
    2 => Ok((input, ast::Value::Null)),
    3 => Ok((input, ast::Value::Undefined)),
    4 => map(parse_u8, ast::Value::Register)(input),
    5 => map(parse_u8, |v| ast::Value::Boolean(v != 0))(input),
    6 => map(parse_le_f64, ast::Value::Float64)(input),
    7 => map(parse_le_i32, ast::Value::Sint32)(input),
    8 => map(parse_u8, |v| ast::Value::Constant(u16::from(v)))(input),
    9 => map(parse_le_u16, ast::Value::Constant)(input),
    _ => Err(nom::Err::Error((input, nom::error::ErrorKind::Switch))),
  }
}

pub fn parse_jump_action(input: &[u8]) -> NomResult<&[u8], ast::actions::Jump> {
  let (input, offset) = parse_le_i16(input)?;
  Ok((input, ast::actions::Jump { offset }))
}

pub fn parse_get_url2_action(input: &[u8]) -> NomResult<&[u8], ast::actions::GetUrl2> {
  let (input, flags) = parse_u8(input)?;
  let load_variables = (flags & (1 << 0)) != 0;
  let load_target = (flags & (1 << 1)) != 0;
  // Skip bits [2, 5]
  let method_code = flags >> 6;

  let method = match method_code {
    0 => ast::actions::get_url2::Method::None,
    1 => ast::actions::get_url2::Method::Get,
    2 => ast::actions::get_url2::Method::Post,
    _ => return Err(nom::Err::Error((input, nom::error::ErrorKind::Switch))),
  };

  Ok((
    input,
    ast::actions::GetUrl2 {
      method,
      load_target,
      load_variables,
    },
  ))
}

pub fn parse_define_function_action(input: &[u8]) -> NomResult<&[u8], ast::actions::DefineFunction> {
  use nom::multi::count;
  let (input, name) = parse_c_string(input)?;
  let (input, param_count) = parse_le_u16(input)?;
  let (input, parameters) = count(parse_c_string, param_count.into())(input)?;
  let (input, body_size) = parse_le_u16(input)?;

  Ok((
    input,
    ast::actions::DefineFunction {
      name,
      parameters,
      body_size,
    },
  ))
}

pub fn parse_if_action(input: &[u8]) -> NomResult<&[u8], ast::actions::If> {
  let (input, offset) = parse_le_i16(input)?;
  Ok((input, ast::actions::If { offset }))
}

pub fn parse_goto_frame2_action(input: &[u8]) -> NomResult<&[u8], ast::actions::GotoFrame2> {
  use nom::combinator::cond;
  let (input, flags) = parse_u8(input)?;
  let play = (flags & (1 << 0)) != 0;
  let has_scene_bias = (flags & (1 << 1)) != 0;
  // Skip bits [2, 7]
  let (input, scene_bias) = cond(has_scene_bias, parse_le_u16)(input)?;

  Ok((
    input,
    ast::actions::GotoFrame2 {
      play,
      scene_bias: scene_bias.map(usize::from).unwrap_or_default(),
    },
  ))
}

// TODO: Return `(&[u8], ast::Action)` (the function should never fail)
pub fn parse_action(input: &[u8]) -> NomResult<&[u8], ast::Action> {
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

fn parse_action_body(input: &[u8], code: u8) -> ast::Action {
  use nom::combinator::map;
  let result = match code {
    0x00 => Ok((input, ast::Action::End)),
    0x04 => Ok((input, ast::Action::NextFrame)),
    0x05 => Ok((input, ast::Action::PrevFrame)),
    0x06 => Ok((input, ast::Action::Play)),
    0x07 => Ok((input, ast::Action::Stop)),
    0x08 => Ok((input, ast::Action::ToggleQuality)),
    0x09 => Ok((input, ast::Action::StopSounds)),
    0x0a => Ok((input, ast::Action::Add)),
    0x0b => Ok((input, ast::Action::Subtract)),
    0x0c => Ok((input, ast::Action::Multiply)),
    0x0d => Ok((input, ast::Action::Divide)),
    0x0e => Ok((input, ast::Action::Equals)),
    0x0f => Ok((input, ast::Action::Less)),
    0x10 => Ok((input, ast::Action::And)),
    0x11 => Ok((input, ast::Action::Or)),
    0x12 => Ok((input, ast::Action::Not)),
    0x13 => Ok((input, ast::Action::StringEquals)),
    0x14 => Ok((input, ast::Action::StringLength)),
    0x15 => Ok((input, ast::Action::StringExtract)),
    0x17 => Ok((input, ast::Action::Pop)),
    0x18 => Ok((input, ast::Action::ToInteger)),
    0x1c => Ok((input, ast::Action::GetVariable)),
    0x1d => Ok((input, ast::Action::SetVariable)),
    0x20 => Ok((input, ast::Action::SetTarget2)),
    0x21 => Ok((input, ast::Action::StringAdd)),
    0x22 => Ok((input, ast::Action::GetProperty)),
    0x23 => Ok((input, ast::Action::SetProperty)),
    0x24 => Ok((input, ast::Action::CloneSprite)),
    0x25 => Ok((input, ast::Action::RemoveSprite)),
    0x26 => Ok((input, ast::Action::Trace)),
    0x27 => Ok((input, ast::Action::StartDrag)),
    0x28 => Ok((input, ast::Action::EndDrag)),
    0x29 => Ok((input, ast::Action::StringLess)),
    0x2a => Ok((input, ast::Action::Throw)),
    0x2b => Ok((input, ast::Action::CastOp)),
    0x2c => Ok((input, ast::Action::ImplementsOp)),
    0x2d => Ok((input, ast::Action::FsCommand2)),
    0x30 => Ok((input, ast::Action::RandomNumber)),
    0x31 => Ok((input, ast::Action::MbStringLength)),
    0x32 => Ok((input, ast::Action::CharToAscii)),
    0x33 => Ok((input, ast::Action::AsciiToChar)),
    0x34 => Ok((input, ast::Action::GetTime)),
    0x35 => Ok((input, ast::Action::MbStringExtract)),
    0x36 => Ok((input, ast::Action::MbCharToAscii)),
    0x37 => Ok((input, ast::Action::MbAsciiToChar)),
    0x3a => Ok((input, ast::Action::Delete)),
    0x3b => Ok((input, ast::Action::Delete2)),
    0x3c => Ok((input, ast::Action::DefineLocal)),
    0x3d => Ok((input, ast::Action::CallFunction)),
    0x3e => Ok((input, ast::Action::Return)),
    0x3f => Ok((input, ast::Action::Modulo)),
    0x40 => Ok((input, ast::Action::NewObject)),
    0x41 => Ok((input, ast::Action::DefineLocal2)),
    0x42 => Ok((input, ast::Action::InitArray)),
    0x43 => Ok((input, ast::Action::InitObject)),
    0x44 => Ok((input, ast::Action::TypeOf)),
    0x45 => Ok((input, ast::Action::TargetPath)),
    0x46 => Ok((input, ast::Action::Enumerate)),
    0x47 => Ok((input, ast::Action::Add2)),
    0x48 => Ok((input, ast::Action::Less2)),
    0x49 => Ok((input, ast::Action::Equals2)),
    0x4a => Ok((input, ast::Action::ToNumber)),
    0x4b => Ok((input, ast::Action::ToString)),
    0x4c => Ok((input, ast::Action::PushDuplicate)),
    0x4d => Ok((input, ast::Action::StackSwap)),
    0x4e => Ok((input, ast::Action::GetMember)),
    0x4f => Ok((input, ast::Action::SetMember)),
    0x50 => Ok((input, ast::Action::Increment)),
    0x51 => Ok((input, ast::Action::Decrement)),
    0x52 => Ok((input, ast::Action::CallMethod)),
    0x53 => Ok((input, ast::Action::NewMethod)),
    0x54 => Ok((input, ast::Action::InstanceOf)),
    0x55 => Ok((input, ast::Action::Enumerate2)),
    0x60 => Ok((input, ast::Action::BitAnd)),
    0x61 => Ok((input, ast::Action::BitOr)),
    0x62 => Ok((input, ast::Action::BitXor)),
    0x63 => Ok((input, ast::Action::BitLShift)),
    0x64 => Ok((input, ast::Action::BitRShift)),
    0x65 => Ok((input, ast::Action::BitURShift)),
    0x66 => Ok((input, ast::Action::StrictEquals)),
    0x67 => Ok((input, ast::Action::Greater)),
    0x68 => Ok((input, ast::Action::StringGreater)),
    0x69 => Ok((input, ast::Action::Extends)),
    0x81 => map(parse_goto_frame_action, ast::Action::GotoFrame)(input),
    0x83 => map(parse_get_url_action, ast::Action::GetUrl)(input),
    0x87 => map(parse_store_register_action, ast::Action::StoreRegister)(input),
    0x88 => map(parse_constant_pool_action, ast::Action::ConstantPool)(input),
    0x8a => map(parse_wait_for_frame_action, ast::Action::WaitForFrame)(input),
    0x8b => map(parse_set_target_action, ast::Action::SetTarget)(input),
    0x8c => map(parse_goto_label_action, ast::Action::GotoLabel)(input),
    0x8d => map(parse_wait_for_frame2_action, ast::Action::WaitForFrame2)(input),
    0x8e => map(parse_define_function2_action, ast::Action::DefineFunction2)(input),
    0x8f => map(parse_try_action, ast::Action::Try)(input),
    0x94 => map(parse_with_action, ast::Action::With)(input),
    0x96 => map(parse_push_action, ast::Action::Push)(input),
    0x99 => map(parse_jump_action, ast::Action::Jump)(input),
    0x9a => map(parse_get_url2_action, ast::Action::GetUrl2)(input),
    0x9b => map(parse_define_function_action, ast::Action::DefineFunction)(input),
    0x9d => map(parse_if_action, ast::Action::If)(input),
    0x9e => Ok((input, ast::Action::Call)),
    0x9f => map(parse_goto_frame2_action, ast::Action::GotoFrame2)(input),
    _ => Ok((
      &[][..],
      ast::Action::Unknown(ast::actions::UnknownAction {
        code,
        data: input.to_vec(),
      }),
    )),
  };
  match result {
    Ok((_, action)) => action,
    Err(_) => ast::Action::Error(ast::actions::Error { error: None }),
  }
}

#[cfg(test)]
mod tests {
  use nom;

  use super::*;

  #[test]
  fn test_parse_push_action() {
    {
      let input = vec![0x04, 0x00, 0x07, 0x01, 0x00, 0x00, 0x00, 0x08, 0x02];
      let actual = parse_push_action(&input[..]);
      let expected = Ok((
        &[][..],
        ast::actions::Push {
          values: vec![ast::Value::Register(0), ast::Value::Sint32(1), ast::Value::Constant(2)],
        },
      ));
      assert_eq!(actual, expected);
    }
    {
      let input = vec![0x00, 0x00];
      let actual = parse_push_action(&input[..]);
      let expected = Ok((
        &[][..],
        ast::actions::Push {
          values: vec![ast::Value::String(String::from(""))],
        },
      ));
      assert_eq!(actual, expected);
    }
    {
      let input = vec![0x00, 0x01, 0x00];
      let actual = parse_push_action(&input[..]);
      let expected = Ok((
        &[][..],
        ast::actions::Push {
          values: vec![ast::Value::String(String::from("\x01"))],
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
          ast::Action::Unknown(ast::actions::UnknownAction {
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
          ast::Action::Unknown(ast::actions::UnknownAction {
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

use crate::avm1::parse_action_header;
use crate::parse_action;
use avm1_types::actions as raw_actions;
use avm1_types::actions::r#try::CatchTarget;
use avm1_types::cfg_actions::{CfgDefineFunction, CfgDefineFunction2};
use avm1_types::cfg_blocks::{
  CfgErrorBlock, CfgIfBlock, CfgReturnBlock, CfgSimpleBlock, CfgThrowBlock, CfgTryBlock, CfgWaitForFrame2Block,
  CfgWaitForFrameBlock, CfgWithBlock,
};
use avm1_types::Action as RawAction;
use avm1_types::{Cfg, CfgAction as SimpleAction, CfgBlock, CfgLabel};
use core::convert::TryFrom;
use core::iter::Iterator;
use core::ops::Range;
use nom::lib::std::collections::{BTreeMap, HashMap};

type Avm1Index = usize;
type Avm1Range = Range<usize>;

pub fn parse_cfg(avm1: &[u8]) -> Cfg {
  let mut idg = IdGen::new();
  let parser = Avm1Parser::new(avm1);
  let range: Avm1Range = 0..avm1.len();
  let mut parse_cx = ParseContext::new(&mut idg, range);
  parse_into_cfg(&parser, &mut parse_cx)
}

/// Block identifier generator
#[derive(Clone, Debug, Eq, PartialEq)]
struct IdGen(u64);

impl IdGen {
  fn new() -> Self {
    IdGen(0)
  }

  fn next(&mut self) -> u64 {
    let result: u64 = self.0;
    self.0 += 1;
    result
  }
}

fn try_add_offset(left: usize, right: i16) -> Option<usize> {
  if right >= 0 {
    let right_u16: u16 = u16::try_from(right).unwrap();
    let right_usize = usize::from(right_u16);
    left.checked_add(right_usize)
  } else {
    let right_u16: u16 = u16::try_from(-right).unwrap();
    let right_usize = usize::from(right_u16);
    left.checked_sub(right_usize)
  }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
struct Avm1Parser<'a> {
  bytes: &'a [u8],
}

impl<'a> Avm1Parser<'a> {
  fn new(bytes: &'a [u8]) -> Self {
    Avm1Parser { bytes }
  }

  fn get(&self, offset: usize) -> (usize, RawAction) {
    if offset >= self.bytes.len() {
      return (offset, RawAction::End);
    }
    let input: &[u8] = &self.bytes[offset..];
    match parse_action(input) {
      Ok((next_input, action)) => (offset + (input.len() - next_input.len()), action),
      Err(_) => (offset, RawAction::Error(raw_actions::Error { error: None })),
    }
  }

  fn skip(&self, offset: usize, action_count: usize) -> usize {
    let mut input: &[u8] = &self.bytes[offset..];
    for _ in 0..action_count {
      let (next_input, header) = parse_action_header(input).unwrap();
      input = &next_input[header.length..];
    }
    self.bytes.len() - input.len()
  }
}

#[derive(Debug, Eq, PartialEq)]
struct ParseContext<'a> {
  idg: &'a mut IdGen,
  layers: Vec<LayerContext>,
}

/// Enum representing the reachability status of an action
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
enum Reachability {
  /// The action is only reached through simple linear control flow following a simple action
  /// TODO: Rename to `advance`
  Linear,
  /// The action is reached through a jump:
  /// - Entry point
  /// - Jump action
  /// - Linear flow following two or more simple actions
  Jump,
}

impl Reachability {
  fn set_jump(&mut self) -> () {
    *self = Reachability::Jump
  }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LayerContext {
  /// Id for this layer
  id: u64,
  /// Range for this layer
  range: Avm1Range,
  /// Indexes of discovered actions and their reachability status
  actions: BTreeMap<Avm1Index, Reachability>,
  /// Stack of discovered actions that were not consumed yet.
  /// Actions on the stack are consume with `pop`.
  new_actions: Vec<Avm1Index>,
}

impl<'a> ParseContext<'a> {
  fn new(idg: &'a mut IdGen, range: Avm1Range) -> Self {
    let id: u64 = idg.next();
    let mut layer = LayerContext {
      id,
      range,
      actions: BTreeMap::new(),
      new_actions: Vec::new(),
    };
    layer.actions.insert(layer.range.start, Reachability::Jump);
    layer.new_actions.push(layer.range.start);
    Self {
      idg,
      layers: vec![layer],
    }
  }

  fn push_layer(&mut self, range: Avm1Range) -> () {
    let id: u64 = self.idg.next();
    let mut layer = LayerContext {
      id,
      range,
      actions: BTreeMap::new(),
      new_actions: Vec::new(),
    };
    layer.actions.insert(layer.range.start, Reachability::Jump);
    layer.new_actions.push(layer.range.start);
    self.layers.push(layer);
  }

  fn pop_layer(&mut self) -> () {
    let top_layer = self.layers.pop();
    debug_assert!(top_layer.is_some());
  }

  /// Adds a path reaching `index` through linear flow
  fn linear(&mut self, index: Avm1Index) -> () {
    let top_layer = self.layers.last_mut().unwrap();
    let actions = &mut top_layer.actions;
    let new_actions = &mut top_layer.new_actions;
    actions
      .entry(index)
      .and_modify(Reachability::set_jump)
      .or_insert_with(|| {
        new_actions.push(index);
        Reachability::Linear
      });
  }

  /// Marks `index` as a jump target and returns the target label
  fn jump(&mut self, index: Avm1Index) -> Option<CfgLabel> {
    let mut is_top = true;
    for layer in self.layers.iter_mut().rev() {
      if layer.range.contains(&index) || (!is_top && index == layer.range.start) {
        let actions = &mut layer.actions;
        let new_actions = &mut layer.new_actions;

        actions
          .entry(index)
          .and_modify(Reachability::set_jump)
          .or_insert_with(|| {
            new_actions.push(index);
            Reachability::Jump
          });
        return Some(CfgLabel(format!("l{}_{}", layer.id, index)));
      };
      is_top = false;
    }
    None
  }

  fn pop_action(&mut self) -> Option<Avm1Index> {
    let top_layer = self.layers.last_mut().unwrap();
    top_layer.new_actions.pop()
  }

  fn iter_labels(&self) -> impl Iterator<Item = Avm1Index> + '_ {
    let top_layer = self.layers.last().unwrap();
    top_layer.actions.iter().filter_map(|(i, r)| match r {
      Reachability::Jump => Some(*i),
      _ => None,
    })
  }

  fn top_layer(&self) -> &LayerContext {
    self.layers.last().unwrap()
  }

  fn get_target_label(&self, target: Avm1Index) -> Option<CfgLabel> {
    for layer in self.layers.iter().rev() {
      if layer.range.contains(&target) {
        return Some(CfgLabel(format!("l{}_{}", layer.id, target)));
      }
    }
    None
  }
}

#[derive(Debug, Eq, PartialEq)]
enum CfgFlow {
  Simple(usize, SimpleAction),
  If(Option<CfgLabel>, Option<CfgLabel>),
  Jump(Option<CfgLabel>),
  Error,
  Throw,
  Return,
  With(Cfg),
  Try(Cfg, CatchTarget, Option<Cfg>, Option<Cfg>),
  WaitForFrame(u16, Option<CfgLabel>, Option<CfgLabel>),
  WaitForFrame2(Option<CfgLabel>, Option<CfgLabel>),
}

fn parse_into_cfg(parser: &Avm1Parser, traversal: &mut ParseContext) -> Cfg {
  let mut parsed: HashMap<usize, CfgFlow> = HashMap::new();

  while let Some(cur_offset) = traversal.pop_action() {
    if !traversal.top_layer().range.contains(&cur_offset) {
      parsed.insert(cur_offset, CfgFlow::Jump(traversal.jump(cur_offset)));
      continue;
    }

    let (end_offset, raw) = parser.get(cur_offset);

    let action: CfgFlow = match raw {
      RawAction::Add => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Add)
      }
      RawAction::Add2 => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Add2)
      }
      RawAction::And => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::And)
      }
      RawAction::AsciiToChar => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::AsciiToChar)
      }
      RawAction::BitAnd => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::BitAnd)
      }
      RawAction::BitOr => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::BitOr)
      }
      RawAction::BitLShift => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::BitLShift)
      }
      RawAction::BitRShift => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::BitRShift)
      }
      RawAction::BitURShift => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::BitURShift)
      }
      RawAction::BitXor => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::BitXor)
      }
      RawAction::Call => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Call)
      }
      RawAction::CallFunction => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::CallFunction)
      }
      RawAction::CallMethod => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::CallMethod)
      }
      RawAction::CharToAscii => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::CharToAscii)
      }
      RawAction::CastOp => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::CastOp)
      }
      RawAction::CloneSprite => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::CloneSprite)
      }
      RawAction::ConstantPool(action) => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::ConstantPool(action))
      }
      RawAction::Decrement => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Decrement)
      }
      RawAction::DefineFunction(action) => {
        let fn_range: Avm1Range = end_offset..(end_offset + usize::from(action.body_size));
        let mut fn_child_traversal = ParseContext::new(traversal.idg, fn_range.clone());
        let cfg: Cfg = parse_into_cfg(parser, &mut fn_child_traversal);
        traversal.linear(fn_range.end);
        CfgFlow::Simple(
          fn_range.end,
          SimpleAction::DefineFunction(CfgDefineFunction {
            name: action.name,
            parameters: action.parameters,
            body: cfg,
          }),
        )
      }
      RawAction::DefineFunction2(action) => {
        let fn_range: Avm1Range = end_offset..(end_offset + usize::from(action.body_size));
        let mut fn_child_traversal = ParseContext::new(traversal.idg, fn_range.clone());
        let cfg: Cfg = parse_into_cfg(parser, &mut fn_child_traversal);
        traversal.linear(fn_range.end);
        CfgFlow::Simple(
          fn_range.end,
          SimpleAction::DefineFunction2(CfgDefineFunction2 {
            name: action.name,
            register_count: action.register_count,
            preload_this: action.preload_this,
            suppress_this: action.suppress_this,
            preload_arguments: action.preload_arguments,
            suppress_arguments: action.suppress_arguments,
            preload_super: action.preload_super,
            suppress_super: action.suppress_super,
            preload_root: action.preload_root,
            preload_parent: action.preload_parent,
            preload_global: action.preload_global,
            parameters: action.parameters,
            body: cfg,
          }),
        )
      }
      RawAction::DefineLocal => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::DefineLocal)
      }
      RawAction::DefineLocal2 => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::DefineLocal2)
      }
      RawAction::Delete => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Delete)
      }
      RawAction::Delete2 => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Delete2)
      }
      RawAction::Divide => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Divide)
      }
      RawAction::End => CfgFlow::Jump(None),
      RawAction::EndDrag => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::EndDrag)
      }
      RawAction::Enumerate => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Enumerate)
      }
      RawAction::Enumerate2 => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Enumerate2)
      }
      RawAction::Equals => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Equals)
      }
      RawAction::Equals2 => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Equals2)
      }
      RawAction::Error(_) => CfgFlow::Error,
      RawAction::Extends => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Extends)
      }
      RawAction::FsCommand2 => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::FsCommand2)
      }
      RawAction::GetMember => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::GetMember)
      }
      RawAction::GetProperty => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::GetProperty)
      }
      RawAction::GetTime => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::GetTime)
      }
      RawAction::GetUrl(action) => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::GetUrl(action))
      }
      RawAction::GetUrl2(action) => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::GetUrl2(action))
      }
      RawAction::GetVariable => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::GetVariable)
      }
      RawAction::GotoFrame(action) => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::GotoFrame(action))
      }
      RawAction::GotoFrame2(action) => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::GotoFrame2(action))
      }
      RawAction::GotoLabel(action) => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::GotoLabel(action))
      }
      RawAction::Greater => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Greater)
      }
      RawAction::If(action) => {
        let branch_true = if let Some(jump_offset) = try_add_offset(end_offset, action.offset) {
          traversal.jump(jump_offset)
        } else {
          None
        };
        let branch_false = traversal.jump(end_offset);
        CfgFlow::If(branch_true, branch_false)
      }
      RawAction::ImplementsOp => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::ImplementsOp)
      }
      RawAction::Increment => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Increment)
      }
      RawAction::InitArray => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::InitArray)
      }
      RawAction::InitObject => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::InitObject)
      }
      RawAction::InstanceOf => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::InstanceOf)
      }
      RawAction::Jump(action) => {
        let target = if let Some(jump_offset) = try_add_offset(end_offset, action.offset) {
          traversal.jump(jump_offset)
        } else {
          None
        };
        CfgFlow::Jump(target)
      }
      RawAction::Less => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Less)
      }
      RawAction::Less2 => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Less2)
      }
      RawAction::MbAsciiToChar => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::MbAsciiToChar)
      }
      RawAction::MbCharToAscii => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::MbCharToAscii)
      }
      RawAction::MbStringExtract => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::MbStringExtract)
      }
      RawAction::MbStringLength => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::MbStringLength)
      }
      RawAction::Modulo => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Modulo)
      }
      RawAction::Multiply => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Multiply)
      }
      RawAction::NewMethod => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::NewMethod)
      }
      RawAction::NewObject => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::NewObject)
      }
      RawAction::NextFrame => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::NextFrame)
      }
      RawAction::Not => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Not)
      }
      RawAction::Or => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Or)
      }
      RawAction::Play => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Play)
      }
      RawAction::Pop => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Pop)
      }
      RawAction::PrevFrame => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::PrevFrame)
      }
      RawAction::Push(action) => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Push(action))
      }
      RawAction::PushDuplicate => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::PushDuplicate)
      }
      RawAction::RandomNumber => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::RandomNumber)
      }
      RawAction::RemoveSprite => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::RemoveSprite)
      }
      RawAction::Return => CfgFlow::Return,
      RawAction::SetMember => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::SetMember)
      }
      RawAction::SetProperty => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::SetProperty)
      }
      RawAction::SetTarget(action) => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::SetTarget(action))
      }
      RawAction::SetTarget2 => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::SetTarget2)
      }
      RawAction::SetVariable => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::SetVariable)
      }
      RawAction::StackSwap => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::StackSwap)
      }
      RawAction::StartDrag => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::StartDrag)
      }
      RawAction::Stop => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Stop)
      }
      RawAction::StopSounds => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::StopSounds)
      }
      RawAction::StoreRegister(action) => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::StoreRegister(action))
      }
      RawAction::StrictEquals => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::StrictEquals)
      }
      // RawAction::StrictMode(action) => { traversal.linear(end_offset); CfgFlow::Simple(end_offset, SimpleAction::StrictMode(action)) },
      RawAction::StringAdd => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::StringAdd)
      }
      RawAction::StringEquals => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::StringEquals)
      }
      RawAction::StringExtract => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::StringExtract)
      }
      RawAction::StringGreater => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::StringGreater)
      }
      RawAction::StringLength => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::StringLength)
      }
      RawAction::StringLess => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::StringLess)
      }
      RawAction::Subtract => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Subtract)
      }
      RawAction::TargetPath => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::TargetPath)
      }
      RawAction::Throw => CfgFlow::Throw,
      RawAction::ToInteger => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::ToInteger)
      }
      RawAction::ToNumber => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::ToNumber)
      }
      RawAction::ToString => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::ToString)
      }
      RawAction::ToggleQuality => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::ToggleQuality)
      }
      RawAction::Trace => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Trace)
      }
      RawAction::Try(action) => {
        let try_range: Avm1Range = end_offset..(end_offset + usize::from(action.try_size));
        let mut next_offset = try_range.end;
        let catch_range: Option<Avm1Range> = if let Some(catch_size) = action.catch_size {
          let catch_range = next_offset..(next_offset + usize::from(catch_size));
          next_offset = catch_range.end;
          Some(catch_range)
        } else {
          None
        };
        let finally_range: Option<Avm1Range> = if let Some(finally_size) = action.finally_size {
          let finally_range = next_offset..(next_offset + usize::from(finally_size));
          Some(finally_range)
        } else {
          None
        };

        let finally_body: Option<Cfg> = if let Some(finally_range) = &finally_range {
          traversal.push_layer(finally_range.clone());
          Some(parse_into_cfg(parser, traversal))
        } else {
          None
        };

        traversal.push_layer(try_range.clone());
        let try_body: Cfg = parse_into_cfg(parser, traversal);
        traversal.pop_layer();

        let catch_body = catch_range.map(|catch_range| {
          traversal.push_layer(catch_range.clone());
          let catch_body: Cfg = parse_into_cfg(parser, traversal);
          traversal.pop_layer();
          catch_body
        });

        if finally_range.is_some() {
          traversal.pop_layer();
        }

        CfgFlow::Try(try_body, action.catch_target, catch_body, finally_body)
      }
      RawAction::TypeOf => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::TypeOf)
      }
      RawAction::Unknown(action) => {
        traversal.linear(end_offset);
        CfgFlow::Simple(end_offset, SimpleAction::Unknown(action))
      }
      RawAction::WaitForFrame(action) => {
        let not_loaded_offset = parser.skip(end_offset, action.skip_count);
        let not_loaded_target = traversal.jump(not_loaded_offset);
        let loaded_target = traversal.jump(end_offset);
        CfgFlow::WaitForFrame(u16::try_from(action.frame).unwrap(), not_loaded_target, loaded_target)
      }
      RawAction::WaitForFrame2(action) => {
        let not_loaded_offset = parser.skip(end_offset, action.skip_count);
        let not_loaded_target = traversal.jump(not_loaded_offset);
        let loaded_target = traversal.jump(end_offset);
        CfgFlow::WaitForFrame2(not_loaded_target, loaded_target)
      }
      RawAction::With(action) => {
        let range: Avm1Range = end_offset..(end_offset + usize::from(action.with_size));
        traversal.push_layer(range.clone());
        let cfg: Cfg = parse_into_cfg(parser, traversal);
        traversal.pop_layer();
        CfgFlow::With(cfg)
      }
    };

    {
      let old: Option<CfgFlow> = parsed.insert(cur_offset, action);
      debug_assert!(old.is_none());
    }
  }
  let mut head: Option<CfgBlock> = None;
  let mut tail: Vec<CfgBlock> = Vec::new();

  for start_index in traversal.iter_labels() {
    let label: CfgLabel = CfgLabel(format!("l{}_{}", traversal.top_layer().id, start_index));
    let mut builder: CfgBlockBuilder = CfgBlockBuilder::new(label);
    let mut index: Avm1Index = start_index;
    let block: CfgBlock = loop {
      let action = parsed
        .remove(&index)
        .expect("`parsed` to have actions found during traversal");
      match action {
        CfgFlow::Simple(next, simple) => {
          builder.action(simple);
          index = next
        }
        CfgFlow::Jump(target) => break builder.simple(target),
        CfgFlow::If(true_label, false_label) => break builder.cond(true_label, false_label),
        CfgFlow::Return => break builder.r#return(),
        CfgFlow::Throw => break builder.throw(),
        CfgFlow::With(body) => break builder.with(body),
        CfgFlow::Try(r#try, catch_target, catch, finally) => break builder.r#try(r#try, catch_target, catch, finally),
        CfgFlow::Error => break builder.error(),
        CfgFlow::WaitForFrame(frame, loading_label, ready_label) => {
          break builder.wff(frame, loading_label, ready_label)
        }
        CfgFlow::WaitForFrame2(loading_label, ready_label) => break builder.wff2(loading_label, ready_label),
      };
      if traversal.top_layer().actions.get(&index) == Some(&Reachability::Jump) {
        break builder.simple(traversal.get_target_label(index));
      }
    };
    if head.is_none() {
      head = Some(block);
    } else {
      tail.push(block);
    }
  }
  let cfg: Cfg = Cfg {
    head: Box::new(head.expect("Expected head to be defined")),
    tail,
  };
  cfg
}

struct CfgBlockBuilder {
  label: CfgLabel,
  actions: Vec<SimpleAction>,
}

impl CfgBlockBuilder {
  fn new(label: CfgLabel) -> Self {
    Self {
      label,
      actions: Vec::new(),
    }
  }

  fn action(&mut self, action: SimpleAction) -> &mut Self {
    self.actions.push(action);
    self
  }

  fn simple(self, next: Option<CfgLabel>) -> CfgBlock {
    CfgBlock::Simple(CfgSimpleBlock {
      label: self.label,
      actions: self.actions,
      next,
    })
  }

  fn cond(self, if_true: Option<CfgLabel>, if_false: Option<CfgLabel>) -> CfgBlock {
    CfgBlock::If(CfgIfBlock {
      label: self.label,
      actions: self.actions,
      if_true,
      if_false,
    })
  }

  fn r#return(self) -> CfgBlock {
    CfgBlock::Return(CfgReturnBlock {
      label: self.label,
      actions: self.actions,
    })
  }

  fn throw(self) -> CfgBlock {
    CfgBlock::Throw(CfgThrowBlock {
      label: self.label,
      actions: self.actions,
    })
  }

  fn error(self) -> CfgBlock {
    CfgBlock::Error(CfgErrorBlock {
      label: self.label,
      actions: self.actions,
      error: None,
    })
  }

  fn with(self, body: Cfg) -> CfgBlock {
    CfgBlock::With(CfgWithBlock {
      label: self.label,
      actions: self.actions,
      with: body,
    })
  }

  fn wff(self, frame: u16, if_not_loaded: Option<CfgLabel>, if_loaded: Option<CfgLabel>) -> CfgBlock {
    CfgBlock::WaitForFrame(CfgWaitForFrameBlock {
      label: self.label,
      actions: self.actions,
      frame,
      if_not_loaded,
      if_loaded,
    })
  }

  fn wff2(self, if_not_loaded: Option<CfgLabel>, if_loaded: Option<CfgLabel>) -> CfgBlock {
    CfgBlock::WaitForFrame2(CfgWaitForFrame2Block {
      label: self.label,
      actions: self.actions,
      if_not_loaded,
      if_loaded,
    })
  }

  fn r#try(self, r#try: Cfg, catch_target: CatchTarget, catch: Option<Cfg>, finally: Option<Cfg>) -> CfgBlock {
    CfgBlock::Try(CfgTryBlock {
      label: self.label,
      actions: self.actions,
      r#try,
      catch_target,
      catch,
      finally,
    })
  }
}

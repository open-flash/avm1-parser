use crate::avm1::parse_action_header;
use crate::parse_action;
use avm1_types::cfg;
use avm1_types::cfg::{Cfg, CfgBlock, CfgFlow, CfgLabel};
use avm1_types::raw;
use core::convert::TryFrom;
use core::iter::Iterator;
use core::ops::Range;
use nom::lib::std::collections::{BTreeMap, HashMap};
use vec1::Vec1;

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

  fn get(&self, offset: usize) -> (usize, raw::Action) {
    if offset >= self.bytes.len() {
      return (offset, raw::Action::End);
    }
    let input: &[u8] = &self.bytes[offset..];
    match parse_action(input) {
      Ok((next_input, action)) => (offset + (input.len() - next_input.len()), action),
      Err(_) => (offset, raw::Action::Error(raw::Error { error: None })),
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
enum Parsed {
  Action(usize, cfg::Action),
  Flow(CfgFlow),
}

fn parse_into_cfg(parser: &Avm1Parser, traversal: &mut ParseContext) -> Cfg {
  let mut parsed: HashMap<usize, Parsed> = HashMap::new();

  while let Some(cur_offset) = traversal.pop_action() {
    if !traversal.top_layer().range.contains(&cur_offset) {
      let jump = cfg::Simple {
        next: traversal.jump(cur_offset),
      };
      parsed.insert(cur_offset, Parsed::Flow(CfgFlow::Simple(jump)));
      continue;
    }

    let (end_offset, raw) = parser.get(cur_offset);

    let cur_parsed: Parsed = match raw {
      raw::Action::Add => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Add)
      }
      raw::Action::Add2 => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Add2)
      }
      raw::Action::And => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::And)
      }
      raw::Action::AsciiToChar => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::AsciiToChar)
      }
      raw::Action::BitAnd => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::BitAnd)
      }
      raw::Action::BitOr => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::BitOr)
      }
      raw::Action::BitLShift => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::BitLShift)
      }
      raw::Action::BitRShift => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::BitRShift)
      }
      raw::Action::BitURShift => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::BitURShift)
      }
      raw::Action::BitXor => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::BitXor)
      }
      raw::Action::Call => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Call)
      }
      raw::Action::CallFunction => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::CallFunction)
      }
      raw::Action::CallMethod => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::CallMethod)
      }
      raw::Action::CharToAscii => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::CharToAscii)
      }
      raw::Action::CastOp => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::CastOp)
      }
      raw::Action::CloneSprite => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::CloneSprite)
      }
      raw::Action::ConstantPool(action) => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::ConstantPool(action))
      }
      raw::Action::Decrement => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Decrement)
      }
      raw::Action::DefineFunction(action) => {
        let fn_range: Avm1Range = end_offset..(end_offset + usize::from(action.body_size));
        let mut fn_child_traversal = ParseContext::new(traversal.idg, fn_range.clone());
        let cfg: Cfg = parse_into_cfg(parser, &mut fn_child_traversal);
        traversal.linear(fn_range.end);
        Parsed::Action(
          fn_range.end,
          cfg::Action::DefineFunction(cfg::DefineFunction {
            name: action.name,
            parameters: action.parameters,
            body: cfg,
          }),
        )
      }
      raw::Action::DefineFunction2(action) => {
        let fn_range: Avm1Range = end_offset..(end_offset + usize::from(action.body_size));
        let mut fn_child_traversal = ParseContext::new(traversal.idg, fn_range.clone());
        let cfg: Cfg = parse_into_cfg(parser, &mut fn_child_traversal);
        traversal.linear(fn_range.end);
        Parsed::Action(
          fn_range.end,
          cfg::Action::DefineFunction2(cfg::DefineFunction2 {
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
      raw::Action::DefineLocal => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::DefineLocal)
      }
      raw::Action::DefineLocal2 => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::DefineLocal2)
      }
      raw::Action::Delete => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Delete)
      }
      raw::Action::Delete2 => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Delete2)
      }
      raw::Action::Divide => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Divide)
      }
      raw::Action::End => Parsed::Flow(CfgFlow::Simple(cfg::Simple { next: None })),
      raw::Action::EndDrag => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::EndDrag)
      }
      raw::Action::Enumerate => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Enumerate)
      }
      raw::Action::Enumerate2 => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Enumerate2)
      }
      raw::Action::Equals => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Equals)
      }
      raw::Action::Equals2 => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Equals2)
      }
      raw::Action::Error(action) => Parsed::Flow(CfgFlow::Error(cfg::Error { error: action.error })),
      raw::Action::Extends => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Extends)
      }
      raw::Action::FsCommand2 => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::FsCommand2)
      }
      raw::Action::GetMember => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::GetMember)
      }
      raw::Action::GetProperty => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::GetProperty)
      }
      raw::Action::GetTime => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::GetTime)
      }
      raw::Action::GetUrl(action) => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::GetUrl(action))
      }
      raw::Action::GetUrl2(action) => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::GetUrl2(action))
      }
      raw::Action::GetVariable => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::GetVariable)
      }
      raw::Action::GotoFrame(action) => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::GotoFrame(action))
      }
      raw::Action::GotoFrame2(action) => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::GotoFrame2(action))
      }
      raw::Action::GotoLabel(action) => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::GotoLabel(action))
      }
      raw::Action::Greater => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Greater)
      }
      raw::Action::If(action) => {
        let true_target = if let Some(jump_offset) = try_add_offset(end_offset, action.offset) {
          traversal.jump(jump_offset)
        } else {
          None
        };
        let false_target = traversal.jump(end_offset);
        Parsed::Flow(CfgFlow::If(cfg::If {
          true_target,
          false_target,
        }))
      }
      raw::Action::ImplementsOp => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::ImplementsOp)
      }
      raw::Action::Increment => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Increment)
      }
      raw::Action::InitArray => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::InitArray)
      }
      raw::Action::InitObject => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::InitObject)
      }
      raw::Action::InstanceOf => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::InstanceOf)
      }
      raw::Action::Jump(action) => {
        let next = if let Some(jump_offset) = try_add_offset(end_offset, action.offset) {
          traversal.jump(jump_offset)
        } else {
          None
        };
        Parsed::Flow(CfgFlow::Simple(cfg::Simple { next }))
      }
      raw::Action::Less => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Less)
      }
      raw::Action::Less2 => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Less2)
      }
      raw::Action::MbAsciiToChar => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::MbAsciiToChar)
      }
      raw::Action::MbCharToAscii => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::MbCharToAscii)
      }
      raw::Action::MbStringExtract => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::MbStringExtract)
      }
      raw::Action::MbStringLength => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::MbStringLength)
      }
      raw::Action::Modulo => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Modulo)
      }
      raw::Action::Multiply => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Multiply)
      }
      raw::Action::NewMethod => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::NewMethod)
      }
      raw::Action::NewObject => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::NewObject)
      }
      raw::Action::NextFrame => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::NextFrame)
      }
      raw::Action::Not => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Not)
      }
      raw::Action::Or => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Or)
      }
      raw::Action::Play => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Play)
      }
      raw::Action::Pop => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Pop)
      }
      raw::Action::PrevFrame => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::PrevFrame)
      }
      raw::Action::Push(action) => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Push(action))
      }
      raw::Action::PushDuplicate => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::PushDuplicate)
      }
      raw::Action::RandomNumber => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::RandomNumber)
      }
      raw::Action::RemoveSprite => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::RemoveSprite)
      }
      raw::Action::Return => Parsed::Flow(CfgFlow::Return),
      raw::Action::SetMember => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::SetMember)
      }
      raw::Action::SetProperty => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::SetProperty)
      }
      raw::Action::SetTarget(action) => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::SetTarget(action))
      }
      raw::Action::SetTarget2 => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::SetTarget2)
      }
      raw::Action::SetVariable => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::SetVariable)
      }
      raw::Action::StackSwap => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::StackSwap)
      }
      raw::Action::StartDrag => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::StartDrag)
      }
      raw::Action::Stop => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Stop)
      }
      raw::Action::StopSounds => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::StopSounds)
      }
      raw::Action::StoreRegister(action) => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::StoreRegister(action))
      }
      raw::Action::StrictEquals => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::StrictEquals)
      }
      raw::Action::StrictMode(action) => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::StrictMode(action))
      }
      raw::Action::StringAdd => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::StringAdd)
      }
      raw::Action::StringEquals => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::StringEquals)
      }
      raw::Action::StringExtract => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::StringExtract)
      }
      raw::Action::StringGreater => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::StringGreater)
      }
      raw::Action::StringLength => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::StringLength)
      }
      raw::Action::StringLess => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::StringLess)
      }
      raw::Action::Subtract => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Subtract)
      }
      raw::Action::TargetPath => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::TargetPath)
      }
      raw::Action::Throw => Parsed::Flow(CfgFlow::Throw),
      raw::Action::ToInteger => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::ToInteger)
      }
      raw::Action::ToNumber => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::ToNumber)
      }
      raw::Action::ToString => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::ToString)
      }
      raw::Action::ToggleQuality => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::ToggleQuality)
      }
      raw::Action::Trace => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Trace)
      }
      raw::Action::Try(action) => {
        let try_start: Avm1Index = end_offset;
        let catch_start: Avm1Index = try_start + usize::from(action.r#try);
        let finally_start: Avm1Index = catch_start + action.catch.as_ref().map_or(0, |c| usize::from(c.size));

        let finally: Option<Cfg> = if let Some(finally_size) = action.finally {
          traversal.push_layer(finally_start..(finally_start + usize::from(finally_size)));
          Some(parse_into_cfg(parser, traversal))
        } else {
          None
        };

        let r#try = {
          traversal.push_layer(try_start..(try_start + usize::from(action.r#try)));
          let r#try: Cfg = parse_into_cfg(parser, traversal);
          traversal.pop_layer();
          r#try
        };

        let catch = action.catch.map(|raw_catch| {
          traversal.push_layer(catch_start..(catch_start + usize::from(raw_catch.size)));
          let body: Cfg = parse_into_cfg(parser, traversal);
          traversal.pop_layer();
          cfg::CatchBlock {
            target: raw_catch.target,
            body,
          }
        });

        if finally.is_some() {
          traversal.pop_layer();
        }

        Parsed::Flow(CfgFlow::Try(cfg::Try { r#try, catch, finally }))
      }
      raw::Action::TypeOf => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::TypeOf)
      }
      raw::Action::Raw(action) => {
        traversal.linear(end_offset);
        Parsed::Action(end_offset, cfg::Action::Raw(action))
      }
      raw::Action::WaitForFrame(action) => {
        let loading_offset = parser.skip(end_offset, usize::from(action.skip));
        let loading_target = traversal.jump(loading_offset);
        let ready_target = traversal.jump(end_offset);
        let wff = cfg::WaitForFrame {
          frame: action.frame,
          loading_target,
          ready_target,
        };
        Parsed::Flow(CfgFlow::WaitForFrame(wff))
      }
      raw::Action::WaitForFrame2(action) => {
        let loading_offset = parser.skip(end_offset, usize::from(action.skip));
        let loading_target = traversal.jump(loading_offset);
        let ready_target = traversal.jump(end_offset);
        let wff = cfg::WaitForFrame2 {
          loading_target,
          ready_target,
        };
        Parsed::Flow(CfgFlow::WaitForFrame2(wff))
      }
      raw::Action::With(action) => {
        let range: Avm1Range = end_offset..(end_offset + usize::from(action.size));
        traversal.push_layer(range);
        let body: Cfg = parse_into_cfg(parser, traversal);
        traversal.pop_layer();
        Parsed::Flow(CfgFlow::With(cfg::With { body }))
      }
    };

    {
      let old: Option<Parsed> = parsed.insert(cur_offset, cur_parsed);
      debug_assert!(old.is_none());
    }
  }

  let mut blocks: Vec<CfgBlock> = Vec::new();

  for start_index in traversal.iter_labels() {
    let label: CfgLabel = CfgLabel(format!("l{}_{}", traversal.top_layer().id, start_index));
    let mut builder: CfgBlockBuilder = CfgBlockBuilder::new(label);
    let mut index: Avm1Index = start_index;
    let block: CfgBlock = loop {
      let action = parsed
        .remove(&index)
        .expect("`parsed` to have actions found during traversal");
      match action {
        Parsed::Action(next, action) => {
          builder.action(action);
          index = next
        }
        Parsed::Flow(flow) => break builder.flow(flow),
      };
      if traversal.top_layer().actions.get(&index) == Some(&Reachability::Jump) {
        let jump = cfg::Simple {
          next: traversal.get_target_label(index),
        };
        break builder.flow(CfgFlow::Simple(jump));
      }
    };
    blocks.push(block);
  }

  let blocks: Vec1<CfgBlock> = Vec1::try_from_vec(blocks).unwrap();
  Cfg { blocks }
}

struct CfgBlockBuilder {
  label: CfgLabel,
  actions: Vec<cfg::Action>,
}

impl CfgBlockBuilder {
  fn new(label: CfgLabel) -> Self {
    Self {
      label,
      actions: Vec::new(),
    }
  }

  fn action(&mut self, action: cfg::Action) -> &mut Self {
    self.actions.push(action);
    self
  }

  fn flow(self, flow: CfgFlow) -> CfgBlock {
    CfgBlock {
      label: self.label,
      actions: self.actions,
      flow,
    }
  }
}

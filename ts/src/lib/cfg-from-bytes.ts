import { Action as RawAction } from "avm1-tree/action";
import { ActionType } from "avm1-tree/action-type";
import { Return, Throw } from "avm1-tree/actions";
import { Cfg } from "avm1-tree/cfg";
import { CfgAction } from "avm1-tree/cfg-action";
import { CfgBlock } from "avm1-tree/cfg-block";
import { UintSize } from "semantic-types";
import { Avm1Parser } from "./index";

export function cfgFromBytes(avm1: Uint8Array): Cfg {
  const avm1Parser: Avm1Parser = new Avm1Parser(avm1);
  return innerFromBytes(avm1Parser, 0, avm1.length);
}

interface ParsedAction {
  action: RawAction;
  endOffset: UintSize;
}

function innerFromBytes(parser: Avm1Parser, sectionStart: UintSize, sectionEnd: UintSize): Cfg {
  const offsetToAction: Map<UintSize, ParsedAction> = new Map();
  const underflows: Set<UintSize> = new Set();
  const overflows: Set<UintSize> = new Set();
  const openSet: UintSize[] = [sectionStart];
  while (openSet.length > 0) {
    const curOffset: UintSize = openSet.pop()!;
    if (curOffset < sectionStart || curOffset >= sectionEnd) {
      (curOffset < sectionStart ? underflows : overflows).add(curOffset);
      continue;
    }
    const action: RawAction | undefined = parser.readAt(curOffset);
    if (action === undefined) {
      // End of Actions
      overflows.add(curOffset);
      continue;
    }
    const endOffset: UintSize = parser.getBytePos();
    if (endOffset <= curOffset) {
      throw new Error("ExpectedBytePos to advance");
    }
    offsetToAction.set(curOffset, {action, endOffset});
    const nextOffsets: UintSize[] = [];
    switch (action.action) {
      case ActionType.If: {
        nextOffsets.push(endOffset + action.offset);
        nextOffsets.push(endOffset);
        break;
      }
      case ActionType.Jump: {
        nextOffsets.push(endOffset + action.offset);
        break;
      }
      default: {
        if (!isNeverAction(action)) {
          nextOffsets.push(endOffset);
        }
        break;
      }
    }
    for (const nextOffset of nextOffsets) {
      if (!offsetToAction.has(nextOffset)) {
        openSet.push(nextOffset);
      }
    }
  }
  return toCfg(parser, offsetToAction, underflows, overflows);
}

function toCfg(
  parser: Avm1Parser,
  offsetToAction: ReadonlyMap<UintSize, ParsedAction>,
  underflows: ReadonlySet<UintSize>,
  overflows: ReadonlySet<UintSize>,
): Cfg {
  const unlabelledOffsets: ReadonlySet<UintSize> = getUnlabelledOffset(offsetToAction);
  const labelledOffsets: UintSize[] = [...underflows, ...overflows];
  for (const offset of offsetToAction.keys()) {
    if (!unlabelledOffsets.has(offset)) {
      labelledOffsets.push(offset);
    }
  }
  labelledOffsets.sort((a, b) => a - b);
  const blocks: CfgBlock[] = [];
  for (const [idx, labelledOffset] of labelledOffsets.entries()) {
    const label: string = offsetToLabel(labelledOffset);
    const actions: CfgAction[] = [];
    let next: string | undefined;
    if (underflows.has(labelledOffset)) {
      next = offsetToLabel(labelledOffsets[idx + 1]);
    } else if (!overflows.has(labelledOffset)) {
      let offset: UintSize = labelledOffset;
      do {
        const parsed: ParsedAction | undefined = offsetToAction.get(offset);
        if (parsed === undefined) {
          throw new Error("AssertionError: Expected `parsed` to be defined");
        }
        let action: CfgAction;
        switch (parsed.action.action) {
          case ActionType.DefineFunction: {
            const body: Cfg = innerFromBytes(parser, parsed.endOffset - parsed.action.body.length, parsed.endOffset);
            action = {
              action: ActionType.DefineFunction,
              name: parsed.action.name,
              parameters: parsed.action.parameters,
              body,
            };
            break;
          }
          case ActionType.DefineFunction2: {
            const body: Cfg = innerFromBytes(parser, parsed.endOffset - parsed.action.body.length, parsed.endOffset);
            action = {
              action: ActionType.DefineFunction2,
              name: parsed.action.name,
              preloadParent: parsed.action.preloadParent,
              preloadRoot: parsed.action.preloadRoot,
              suppressSuper: parsed.action.suppressSuper,
              preloadSuper: parsed.action.preloadSuper,
              suppressArguments: parsed.action.suppressArguments,
              preloadArguments: parsed.action.preloadArguments,
              suppressThis: parsed.action.suppressThis,
              preloadThis: parsed.action.preloadThis,
              preloadGlobal: parsed.action.preloadGlobal,
              registerCount: parsed.action.registerCount,
              parameters: parsed.action.parameters,
              body,
            };
            break;
          }
          case ActionType.If: {
            action = {action: ActionType.If, target: offsetToLabel(parsed.endOffset + parsed.action.offset)};
            break;
          }
          case ActionType.Jump: {
            action = {action: ActionType.Jump, target: offsetToLabel(parsed.endOffset + parsed.action.offset)};
            break;
          }
          case ActionType.Try: {
            let curEnd: number = parsed.endOffset;
            let finallyCfg: Cfg | undefined;
            if (parsed.action.finally !== undefined) {
              finallyCfg = innerFromBytes(parser, curEnd - parsed.action.finally.length, curEnd);
              curEnd -= parsed.action.finally.length;
            }
            let catchCfg: Cfg | undefined;
            if (parsed.action.catch !== undefined) {
              catchCfg = innerFromBytes(parser, curEnd - parsed.action.catch.length, curEnd);
              curEnd -= parsed.action.catch.length;
            }
            const tryCfg: Cfg = innerFromBytes(parser, curEnd - parsed.action.try.length, curEnd);
            action = {
              action: ActionType.Try,
              try: tryCfg,
              catch: catchCfg,
              catchTarget: parsed.action.catchTarget,
              finally: finallyCfg,
            };
            break;
          }
          case ActionType.With: {
            const body: Cfg = innerFromBytes(parser, parsed.endOffset - parsed.action.with.length, parsed.endOffset);
            action = {
              action: ActionType.With,
              with: body,
            };
            break;
          }
          default: {
            action = parsed.action;
            break;
          }
        }
        actions.push(action);
        if (parsed.action.action === ActionType.Jump || isNeverAction(parsed.action)) {
          next = undefined;
          break;
        } else {
          offset = parsed.endOffset;
          next = offsetToLabel(offset);
        }
      } while (unlabelledOffsets.has(offset));
    }
    blocks.push({label, actions, next});
  }

  return {blocks};

  function offsetToLabel(offset: UintSize): string {
    return `label_${offset < 0 ? "n" : "p"}${Math.abs(offset).toString(10)}`;
  }
}

function getUnlabelledOffset(offsetToAction: ReadonlyMap<UintSize, ParsedAction>): Set<UintSize> {
  // For each offset, number of actions ending at this offset
  const endOffsetCounts: Map<UintSize, UintSize> = new Map();
  for (const {action, endOffset} of offsetToAction.values()) {
    if (isNeverAction(action)) {
      continue;
    }
    let count: UintSize | undefined = endOffsetCounts.get(endOffset);
    if (count === undefined) {
      count = 0;
    }
    endOffsetCounts.set(endOffset, count + 1);
  }
  // Offsets that do not need a label: they immediately follow another simple action.
  // They are offset corresponding to a single end and which are not jump or branchIfTrue targets
  const unlabelledOffsets: Set<UintSize> = new Set();
  for (const [endOffset, count] of endOffsetCounts) {
    if (count === 1 && offsetToAction.has(endOffset)) {
      unlabelledOffsets.add(endOffset);
    }
  }
  // Ensure branch targets are labelled
  for (const {action, endOffset} of offsetToAction.values()) {
    if (action.action === ActionType.If || action.action === ActionType.Jump) {
      unlabelledOffsets.delete(endOffset + action.offset);
    }
  }
  return unlabelledOffsets;
}

// Checks if the provided actions ends the control flow
function isNeverAction(action: RawAction): action is Return | Throw {
  return action.action === ActionType.Return || action.action === ActionType.Throw;
}

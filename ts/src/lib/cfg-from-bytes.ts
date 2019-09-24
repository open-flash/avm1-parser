import { Action as RawAction } from "avm1-tree/action";
import { ActionType } from "avm1-tree/action-type";
import { Try } from "avm1-tree/actions/try";
import { Cfg } from "avm1-tree/cfg";
import { CfgAction } from "avm1-tree/cfg-action";
import { CfgBlock } from "avm1-tree/cfg-block";
import { CfgBlockType } from "avm1-tree/cfg-block-type";
import { CfgLabel, NullableCfgLabel } from "avm1-tree/cfg-label";
import { UintSize } from "semantic-types";
import { Avm1Parser } from "./index";

type IdProvider = () => number;

function createIdProvider(): IdProvider {
  let id: number = 0;
  return () => id++;
}

export function cfgFromBytes(avm1: Uint8Array): Cfg {
  const avm1Parser: Avm1Parser = new Avm1Parser(avm1);
  return parseHardBlock(avm1Parser, 0, avm1.length, createIdProvider());
}

interface SoftBlock {
  id: number;
  actions: Map<UintSize, ParsedAction>;
  outJumps: Set<UintSize>;
  jumpTargets: Set<UintSize>;
  simpleTargets: Map<UintSize, number>;
  endOfActions: Set<UintSize>;
  start: UintSize;
  end: UintSize;
}

interface ParsedAction {
  raw: RawAction;
  endOffset: UintSize;
}

function parseSoftBlock(parser: Avm1Parser, blockStart: UintSize, blockEnd: UintSize, idp: IdProvider): SoftBlock {
  const id: number = idp();
  // Map from start offset to raw action and end offest.
  const parsed: Map<UintSize, ParsedAction> = new Map();
  const outJumps: Set<UintSize> = new Set();
  const openSet: UintSize[] = [blockStart];
  const knownOffsets: Set<UintSize> = new Set(openSet);
  // Offsets that must be labeled because there exists `If` or `Jump` actions
  // jumping to these offsets.
  const jumpTargets: Set<UintSize> = new Set();
  // Offsets that are reached through simple linear control flow, with the
  // associated count. The count is usually 1, except in the case of overlapping
  // linear flow.
  const simpleTargets: Map<UintSize, number> = new Map();
  // Offsets of known `EndOfAction` (TODO: define a corresponding raw action)
  const endOfActions: Set<UintSize> = new Set();

  function incSimpleTarget(target: UintSize): void {
    let old: number | undefined = simpleTargets.get(target);
    if (old === undefined) {
      old = 0;
    }
    simpleTargets.set(target, old + 1);
  }

  while (openSet.length > 0) {
    const curOffset: UintSize = openSet.pop()!;
    if (curOffset < blockStart || curOffset >= blockEnd) {
      outJumps.add(curOffset);
      continue;
    }
    const raw: RawAction | undefined = parser.readAt(curOffset);
    if (raw === undefined) {
      // EndOfActions
      endOfActions.add(curOffset);
      continue;
    }
    const endOffset: UintSize = parser.getBytePos();
    if (endOffset <= curOffset) {
      throw new Error("ExpectedBytePos to advance");
    }

    const nextOffsets: Set<UintSize> = new Set();
    switch (raw.action) {
      case ActionType.DefineFunction:
        nextOffsets.add(endOffset + raw.bodySize);
        parsed.set(curOffset, {raw, endOffset});
        incSimpleTarget(endOffset + raw.bodySize);
        break;
      case ActionType.DefineFunction2:
        nextOffsets.add(endOffset + raw.bodySize);
        parsed.set(curOffset, {raw, endOffset});
        incSimpleTarget(endOffset + raw.bodySize);
        break;
      case ActionType.If: {
        nextOffsets.add(endOffset + raw.offset);
        nextOffsets.add(endOffset);
        parsed.set(curOffset, {raw, endOffset});
        jumpTargets.add(endOffset + raw.offset);
        jumpTargets.add(endOffset);
        break;
      }
      case ActionType.Jump: {
        nextOffsets.add(endOffset + raw.offset);
        parsed.set(curOffset, {raw, endOffset});
        jumpTargets.add(endOffset + raw.offset);
        break;
      }
      case ActionType.Return:
        parsed.set(curOffset, {raw, endOffset});
        break;
      case ActionType.Throw:
        parsed.set(curOffset, {raw, endOffset});
        break;
      case ActionType.Try: {
        let tryOffset: UintSize = endOffset;
        const softTry: SoftBlock = parseSoftBlock(parser, tryOffset, tryOffset + raw.trySize, idp);
        tryOffset += raw.trySize;
        let softCatch: SoftBlock | undefined;
        if (raw.catchSize !== undefined) {
          softCatch = parseSoftBlock(parser, tryOffset, tryOffset + raw.catchSize, idp);
          tryOffset += raw.catchSize;
        }
        let softFinally: SoftBlock | undefined;
        if (raw.finallySize !== undefined) {
          softFinally = parseSoftBlock(parser, tryOffset, tryOffset + raw.finallySize, idp);
          tryOffset += raw.finallySize;
        }
        for (const outJump of softTry.outJumps) {
          nextOffsets.add(outJump);
          jumpTargets.add(outJump);
        }
        if (softCatch !== undefined) {
          for (const outJump of softCatch.outJumps) {
            nextOffsets.add(outJump);
            jumpTargets.add(outJump);
          }
        }
        if (softFinally !== undefined) {
          // Jumps from `try` and `catch` to the start of `finally` are handled as direct jumps
          // to avoid duplication.
          nextOffsets.delete(softFinally.start);
          jumpTargets.delete(softFinally.start);
          for (const outJump of softFinally.outJumps) {
            nextOffsets.add(outJump);
            jumpTargets.add(outJump);
          }
        }
        parsed.set(curOffset, {raw, endOffset, try: softTry, catch: softCatch, finally: softFinally} as any);
        break;
      }
      case ActionType.WaitForFrame:
      case ActionType.WaitForFrame2: {
        const notLoadedOffset: UintSize = parser.skipFrom(endOffset, raw.skipCount);
        nextOffsets.add(notLoadedOffset);
        nextOffsets.add(endOffset);
        parsed.set(curOffset, {raw, endOffset, notLoadedOffset} as any);
        jumpTargets.add(notLoadedOffset);
        jumpTargets.add(endOffset);
        break;
      }
      case ActionType.With: {
        const withStart: UintSize = endOffset;
        const withEnd: UintSize = withStart + raw.withSize;
        const inner: SoftBlock = parseSoftBlock(parser, withStart, withEnd, idp);
        for (const outJump of inner.outJumps) {
          nextOffsets.add(outJump);
          jumpTargets.add(outJump);
        }
        parsed.set(curOffset, {raw, endOffset, with: inner} as any);
        break;
      }
      default: {
        nextOffsets.add(endOffset);
        parsed.set(curOffset, {raw, endOffset});
        incSimpleTarget(endOffset);
        break;
      }
    }

    for (const nextOffset of nextOffsets) {
      if (!knownOffsets.has(nextOffset)) {
        knownOffsets.add(nextOffset);
        openSet.push(nextOffset);
      }
    }
  }
  return {
    id,
    actions: parsed,
    outJumps,
    jumpTargets,
    simpleTargets,
    endOfActions,
    start: blockStart,
    end: blockEnd,
  };
}

/**
 *
 * @param soft
 * @param parentLabels `undefined` for hard blocks, or a map of parent labels
 *        for soft blocks.
 */
function resolveLabels(
  soft: SoftBlock,
  parentLabels?: Map<UintSize, CfgLabel | null>,
): Map<UintSize, string | null> {
  function toLabel(offset: number): CfgLabel {
    return `l${soft.id}_${offset}`;
  }

  const offsetToLabel: Map<UintSize, NullableCfgLabel> = new Map();
  if (soft.actions.has(soft.start)) {
    offsetToLabel.set(soft.start, toLabel(soft.start));
  }
  for (const offset of soft.actions.keys()) {
    if (soft.jumpTargets.has(offset) || soft.simpleTargets.get(offset) !== 1) {
      offsetToLabel.set(offset, toLabel(offset));
    }
  }
  for (const end of soft.endOfActions) {
    offsetToLabel.set(end, null);
  }
  if (parentLabels === undefined) {
    // hard block
    for (const outJump of soft.outJumps) {
      if (outJump < soft.start) {
        offsetToLabel.set(outJump, toLabel(soft.start));
      }
      if (outJump >= soft.end || soft.endOfActions.has(outJump)) {
        offsetToLabel.set(outJump, null);
      }
    }
  } else {
    // soft block
    for (const outJump of soft.outJumps) {
      const parentLabel: CfgLabel | null | undefined = parentLabels.get(outJump);
      if (parentLabel === undefined) {
        throw new Error("ExpectedOutJumpToExistInParentLabels");
      }
      offsetToLabel.set(outJump, parentLabel);
    }
  }
  const sortedResult: Map<UintSize, string | null> = new Map();
  const sortedOffsets: UintSize[] = [...offsetToLabel.keys()];
  sortedOffsets.sort((a, b) => a - b);
  for (const o of sortedOffsets) {
    sortedResult.set(o, offsetToLabel.get(o)!);
  }
  return sortedResult;
}

function parseHardBlock(parser: Avm1Parser, blockStart: UintSize, blockEnd: UintSize, idp: IdProvider): Cfg {
  const soft: SoftBlock = parseSoftBlock(parser, blockStart, blockEnd, idp);
  const labels: Map<UintSize, string | null> = resolveLabels(soft, undefined);
  return buildCfg(parser, soft, labels, idp);
}

function buildCfg(parser: Avm1Parser, soft: SoftBlock, labels: Map<UintSize, string | null>, idp: IdProvider): Cfg {
  const blocks: CfgBlock[] = [];
  iterateLabels: for (const [labelOffset, label] of labels) {
    if (label === null || !(soft.start <= labelOffset && labelOffset < soft.end)) {
      continue;
    }
    const actions: CfgAction[] = [];
    let offset: UintSize = labelOffset;
    do {
      if (soft.endOfActions.has(offset)) {
        blocks.push({type: CfgBlockType.Simple, label, actions, next: null});
        continue iterateLabels;
      }
      const parsedAction: ParsedAction | undefined = soft.actions.get(offset);
      if (parsedAction === undefined) {
        throw new Error("ExpectedParsedAction");
      }
      switch (parsedAction.raw.action) {
        case ActionType.DefineFunction: {
          const bodyEnd: UintSize = parsedAction.endOffset + parsedAction.raw.bodySize;
          const cfg: Cfg = parseHardBlock(parser, parsedAction.endOffset, bodyEnd, idp);
          actions.push({
            action: ActionType.DefineFunction,
            name: parsedAction.raw.name,
            parameters: parsedAction.raw.parameters,
            body: cfg,
          });
          offset = bodyEnd;
          break;
        }
        case ActionType.DefineFunction2: {
          const bodyEnd: UintSize = parsedAction.endOffset + parsedAction.raw.bodySize;
          const cfg: Cfg = parseHardBlock(parser, parsedAction.endOffset, bodyEnd, idp);
          actions.push({
            action: ActionType.DefineFunction2,
            name: parsedAction.raw.name,
            preloadParent: parsedAction.raw.preloadParent,
            preloadRoot: parsedAction.raw.preloadRoot,
            suppressSuper: parsedAction.raw.suppressSuper,
            preloadSuper: parsedAction.raw.preloadSuper,
            suppressArguments: parsedAction.raw.suppressArguments,
            preloadArguments: parsedAction.raw.preloadArguments,
            suppressThis: parsedAction.raw.suppressThis,
            preloadThis: parsedAction.raw.preloadThis,
            preloadGlobal: parsedAction.raw.preloadGlobal,
            registerCount: parsedAction.raw.registerCount,
            parameters: parsedAction.raw.parameters,
            body: cfg,
          });
          offset = bodyEnd;
          break;
        }
        case ActionType.If: {
          const ifTrue: string | null | undefined = labels.get(parsedAction.endOffset + parsedAction.raw.offset);
          if (ifTrue === undefined) {
            throw new Error("ExpectedIfTargetToHaveALabel");
          }
          const ifFalse: string | null | undefined = labels.get(parsedAction.endOffset);
          if (ifFalse === undefined) {
            throw new Error("ExpectedIfTargetToHaveALabel");
          }
          blocks.push({type: CfgBlockType.If, label, actions, ifTrue, ifFalse});
          continue iterateLabels;
        }
        case ActionType.Jump: {
          const target: string | null | undefined = labels.get(parsedAction.endOffset + parsedAction.raw.offset);
          if (target === undefined) {
            throw new Error("ExpectedJumpTargetToHaveALabel");
          }
          blocks.push({type: CfgBlockType.Simple, label, actions, next: target});
          continue iterateLabels;
        }
        case ActionType.Return: {
          blocks.push({type: CfgBlockType.Return, label, actions});
          continue iterateLabels;
        }
        case ActionType.Throw: {
          blocks.push({type: CfgBlockType.Throw, label, actions});
          continue iterateLabels;
        }
        case ActionType.Try: {
          const raw: Try = parsedAction.raw;

          const trySoftBlock: SoftBlock = (parsedAction as any).try;
          const catchSoftBlock: SoftBlock | undefined = (parsedAction as any).catch;
          const finallySoftBlock: SoftBlock | undefined = (parsedAction as any).finally;

          // Either `labels`, or `labels` with a jump to the start of the finally block.
          let tryCatchParentLabels: Map<UintSize, CfgLabel | null> = labels;

          let finallyCfg: Cfg | undefined;
          if (finallySoftBlock !== undefined) {
            const finallyLabels: Map<UintSize, CfgLabel | null> = resolveLabels(finallySoftBlock, labels);
            finallyCfg = buildCfg(parser, finallySoftBlock, finallyLabels, idp);
            tryCatchParentLabels = new Map([...tryCatchParentLabels]);
            tryCatchParentLabels.set(finallySoftBlock.start, finallyLabels.get(finallySoftBlock.start)!);
          }

          const tryLabels: Map<UintSize, CfgLabel | null> = resolveLabels(trySoftBlock, tryCatchParentLabels);
          const tryCfg: Cfg = buildCfg(parser, trySoftBlock, tryLabels, idp);

          let catchCfg: Cfg | undefined;
          if (catchSoftBlock !== undefined) {
            const catchLabels: Map<UintSize, CfgLabel | null> = resolveLabels(catchSoftBlock, tryCatchParentLabels);
            catchCfg = buildCfg(parser, catchSoftBlock, catchLabels, idp);
          }
          blocks.push({
            type: CfgBlockType.Try,
            label,
            actions,
            try: tryCfg,
            catchTarget: raw.catchTarget,
            catch: catchCfg,
            finally: finallyCfg,
          });
          continue iterateLabels;
        }
        case ActionType.With: {
          const withSoft: SoftBlock = (parsedAction as any).with;
          // tslint:disable-next-line
          const withLabels: Map<UintSize, CfgLabel | null> = resolveLabels(withSoft, labels);
          const withCfg: Cfg = buildCfg(parser, withSoft, withLabels, idp);
          blocks.push({type: CfgBlockType.With, label, actions, with: withCfg});
          continue iterateLabels;
        }
        case ActionType.WaitForFrame: {
          const ifLoaded: string | null | undefined = labels.get(parsedAction.endOffset);
          if (ifLoaded === undefined) {
            throw new Error("ExpectedWaitForFrameIfLoadedToHaveALabel");
          }
          const ifNotLoaded: string | null | undefined = labels.get((parsedAction as any).notLoadedOffset);
          if (ifNotLoaded === undefined) {
            throw new Error("ExpectedWaitForFrameIfNotLoadedToHaveALabel");
          }
          const frame: UintSize = parsedAction.raw.frame;
          blocks.push({type: CfgBlockType.WaitForFrame, label, actions, frame, ifLoaded, ifNotLoaded});
          continue iterateLabels;
        }
        case ActionType.WaitForFrame2: {
          const ifLoaded: string | null | undefined = labels.get(parsedAction.endOffset);
          if (ifLoaded === undefined) {
            throw new Error("ExpectedWaitForFrame2IfLoadedToHaveALabel");
          }
          const ifNotLoaded: string | null | undefined = labels.get((parsedAction as any).notLoadedOffset);
          if (ifNotLoaded === undefined) {
            throw new Error("ExpectedWaitForFrame2IfNotLoadedToHaveALabel");
          }
          blocks.push({type: CfgBlockType.WaitForFrame2, label, actions, ifLoaded, ifNotLoaded});
          continue iterateLabels;
        }
        default:
          actions.push(parsedAction.raw);
          offset = parsedAction.endOffset;
          break;
      }
    } while (!labels.has(offset));
    const next: string | null | undefined = labels.get(offset);
    if (next === undefined) {
      throw new Error("MissingLabel");
    }
    blocks.push({type: CfgBlockType.Simple, label, actions, next});
  }
  return {blocks};
}

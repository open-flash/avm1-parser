import { ActionType } from "avm1-types/action-type";
import { CatchTarget } from "avm1-types/catch-target";
import { Action as CfgAction } from "avm1-types/cfg/action";
import { CatchBlock } from "avm1-types/cfg/catch-block";
import { Cfg } from "avm1-types/cfg/cfg";
import { CfgBlock } from "avm1-types/cfg/cfg-block";
import { CfgFlow } from "avm1-types/cfg/cfg-flow";
import { CfgFlowType } from "avm1-types/cfg/cfg-flow-type";
import { CfgLabel, NullableCfgLabel } from "avm1-types/cfg/cfg-label";
import { Action as RawAction } from "avm1-types/raw/action";
import { UintSize } from "semantic-types";
import { Avm1Parser } from "./index";

export function parseCfg(avm1: Uint8Array): Cfg {
  const idg: IdGen = new IdGen();
  const parser: Avm1Parser = new Avm1Parser(avm1);
  const cx: ParseContext = new ParseContext(idg, 0, avm1.length);
  return innerParseCfg(parser, cx);
}

type Avm1Index = number;
interface Avm1Range {
  start: Avm1Index;
  end: Avm1Index;
}

class IdGen {
  private nextId: number;

  constructor() {
    this.nextId = 0;
  }

  next(): number {
    return this.nextId++;
  }
}

function getLabel(layerId: number, offset: Avm1Index): string {
  return `l${layerId}_${offset}`;
}

class ParseContext {
  private idg: IdGen;
  private head: LayerContext;
  private tail: LayerContext[];

  constructor(idg: IdGen, start: Avm1Index, end: Avm1Index) {
    const id: number = idg.next();
    const head: LayerContext = {id, start, end, actions: new Map(), newActions: []};
    head.actions.set(start, Reachability.Jump);
    head.newActions.push(start);
    this.idg = idg;
    this.head = head;
    this.tail = [];
  }

  child(start: Avm1Index, end: Avm1Index): ParseContext {
    return new ParseContext(this.idg, start, end);
  }

  withLayer<R>(range: Avm1Range | undefined, fn: (cx: ParseContext) => R): R {
    if (range === undefined) {
      return fn(this);
    }
    const id: number = this.idg.next();
    const layer: LayerContext = {id, ...range, actions: new Map(), newActions: []};
    layer.actions.set(range.start, Reachability.Jump);
    layer.newActions.push(range.start);
    this.tail.push(this.head);
    this.head = layer;
    try {
      return fn(this);
    } finally {
      const oldHead: LayerContext | undefined = this.tail.pop();
      // TODO: Debug assertion that `oldHead` is defined
      this.head = oldHead!;
    }
  }

  advance(offset: Avm1Index): void {
    const oldReachability: Reachability | undefined = this.head.actions.get(offset);
    if (oldReachability === undefined) {
      this.head.actions.set(offset, Reachability.Advance);
      this.head.newActions.push(offset);
    } else {
      this.head.actions.set(offset, Reachability.Jump);
    }
  }

  jump(offset: Avm1Index): NullableCfgLabel {
    const layer: LayerContext | null = this.findJumpLayer(offset);
    if (layer === null) {
      return null;
    }
    if (!layer.actions.has(offset)) {
      layer.newActions.push(offset);
    }
    layer.actions.set(offset, Reachability.Jump);
    return getLabel(layer.id, offset);
  }

  isJump(offset: Avm1Index): boolean {
    const layer: LayerContext | null = this.findJumpLayer(offset);
    return layer === null || layer.actions.get(offset) === Reachability.Jump;
  }

  getJump(offset: Avm1Index): NullableCfgLabel {
    const layer: LayerContext | null = this.findJumpLayer(offset);
    if (layer === null) {
      return null;
    }
    // TODO: Assert `layer.actions.get(offset) === Reachability.Jump`
    return getLabel(layer.id, offset);
  }

  getHeadJump(offset: Avm1Index): CfgLabel {
    // TODO: Assert `this.head.actions.get(offset) === Reachability.Jump`
    return getLabel(this.head.id, offset);
  }

  popAction(): Avm1Index | undefined {
    return this.head.newActions.pop();
  }

  contains(offset: Avm1Index): boolean {
    return this.head.start <= offset && offset < this.head.end;
  }

  getBlockStarts(): Avm1Index[] {
    const starts: Avm1Index[] = [];
    for (const [offset, reachability] of this.head.actions) {
      if (reachability === Reachability.Jump) {
        starts.push(offset);
      }
    }
    starts.sort((a, b) => a - b);
    return starts;
  }

  private findJumpLayer(offset: UintSize): LayerContext | null {
    if (this.contains(offset)) {
      return this.head;
    }
    for (let i: UintSize = this.tail.length - 1; i >= 0; i--) {
      const layer: LayerContext = this.tail[i];
      // We use `offset === layer.start` to support jumping to an empty parent
      // layer. This is used when jumping to an empty `finally` block.
      if (offset === layer.start || (layer.start <= offset && offset < layer.end)) {
        return layer;
      }
    }
    return null;
  }
}

interface LayerContext {
  id: number;
  start: Avm1Index;
  end: Avm1Index;
  actions: Map<Avm1Index, Reachability>;
  newActions: Avm1Index[];
}

/**
 * Enum representing how an offset is reached
 */
enum Reachability {
  /**
   * The action is only reached by advancing following a simple action.
   */
  Advance,
  /**
   * The action is reached through a jump:
   * - Entry point
   * - Jump action
   * - Multiple advances leading to the same offset.
   */
  Jump,
}

interface ParsedAction {
  type: "action";
  end: Avm1Index;
  action: CfgAction;
}

interface ParsedFlow {
  type: "flow";
  flow: CfgFlow;
}

type Parsed = ParsedAction | ParsedFlow;

function innerParseCfg(parser: Avm1Parser, cx: ParseContext): Cfg {
  const parsedMap: Map<Avm1Index, Parsed> = new Map();
  while (true) {
    const curOffset: Avm1Index | undefined = cx.popAction();
    if (curOffset === undefined) {
      break;
    }
    let curParsed: Parsed;
    if (cx.contains(curOffset)) {
      const raw: RawAction = parser.readAt(curOffset);
      const endOffset: UintSize = parser.getBytePos();
      curParsed = fromRaw(parser, cx, endOffset, raw);
    } else {
      curParsed = {
        type: "flow",
        flow: {type: CfgFlowType.Simple, next: cx.jump(curOffset)},
      };
    }
    if (curParsed.type === "action") {
      cx.advance(curParsed.end);
    }
    // TODO: Assert `!parsed.has(curOffset)`
    parsedMap.set(curOffset, curParsed);
  }
  const blocks: CfgBlock[] = [];
  for (const startIndex of cx.getBlockStarts()) {
    const label: CfgLabel = cx.getHeadJump(startIndex);
    const actions: CfgAction[] = [];
    let index: Avm1Index | undefined = startIndex;
    let flow: CfgFlow | undefined;
    while (flow === undefined) {
      const parsed: Parsed | undefined = parsedMap.get(index);
      if (parsed === undefined) {
        throw new Error(`AssertionError: Expected parsedMap to have ${index}`);
      }
      if (parsed.type === "action") {
        actions.push(parsed.action);
        index = parsed.end;
        if (cx.isJump(index)) {
          flow = {type: CfgFlowType.Simple, next: cx.getJump(index)};
        }
      } else {
        flow = parsed.flow;
      }
    }
    const block: CfgBlock = {label, actions, flow};
    blocks.push(block);
  }
  return {blocks};
}

/**
 * Converts a raw action to either a CFG action or a CFG flow.
 */
function fromRaw(parser: Avm1Parser, cx: ParseContext, endOffset: UintSize, raw: RawAction): Parsed {
  switch (raw.action) {
    case ActionType.DefineFunction: {
      const bodyStart: Avm1Index = endOffset;
      const bodyEnd: Avm1Index = endOffset + raw.bodySize;
      const body: Cfg = innerParseCfg(parser, cx.child(bodyStart, bodyEnd));
      return {
        type: "action",
        end: bodyEnd,
        action: {
          action: ActionType.DefineFunction,
          name: raw.name,
          parameters: raw.parameters,
          body,
        },
      };
    }
    case ActionType.DefineFunction2: {
      const bodyStart: Avm1Index = endOffset;
      const bodyEnd: Avm1Index = endOffset + raw.bodySize;
      const body: Cfg = innerParseCfg(parser, cx.child(bodyStart, bodyEnd));
      return {
        type: "action",
        end: bodyEnd,
        action: {
          action: ActionType.DefineFunction2,
          name: raw.name,
          registerCount: raw.registerCount,
          preloadThis: raw.preloadThis,
          suppressThis: raw.suppressThis,
          preloadArguments: raw.preloadArguments,
          suppressArguments: raw.suppressArguments,
          preloadSuper: raw.preloadSuper,
          suppressSuper: raw.suppressSuper,
          preloadRoot: raw.preloadRoot,
          preloadParent: raw.preloadParent,
          preloadGlobal: raw.preloadGlobal,
          parameters: raw.parameters,
          body,
        },
      };
    }
    case ActionType.End: {
      return {
        type: "flow",
        flow: { type: CfgFlowType.Simple, next: null},
      };
    }
    case ActionType.Error: {
      // TODO: Propagate error
      return {
        type: "flow",
        flow: { type: CfgFlowType.Error, error: undefined},
      };
    }
    case ActionType.If: {
      const trueOffset: Avm1Index = endOffset + raw.offset;
      const trueTarget: NullableCfgLabel = trueOffset >= 0 ? cx.jump(trueOffset) : null;
      const falseTarget: NullableCfgLabel = cx.jump(endOffset);
      return {
        type: "flow",
        flow: { type: CfgFlowType.If, trueTarget, falseTarget},
      };
    }
    case ActionType.Jump: {
      const nextOffset: Avm1Index = endOffset + raw.offset;
      const next: NullableCfgLabel = nextOffset >= 0 ? cx.jump(nextOffset) : null;
      return {
        type: "flow",
        flow: { type: CfgFlowType.Simple, next},
      };
    }
    case ActionType.Return: {
      return {
        type: "flow",
        flow: { type: CfgFlowType.Return},
      };
    }
    case ActionType.Throw: {
      return {
        type: "flow",
        flow: { type: CfgFlowType.Throw},
      };
    }
    case ActionType.Try: {
      const tryStart: Avm1Index = endOffset;
      const catchStart: Avm1Index = tryStart + raw.try;
      const finallyStart: Avm1Index = catchStart + (raw.catch !== undefined ? raw.catch.size : 0);

      let finallyRange: Avm1Range | undefined;
      if (raw.finally !== undefined) {
        finallyRange = {start: finallyStart, end: finallyStart + raw.finally};
      }
      return cx.withLayer(finallyRange, (cx: ParseContext): Parsed => {
        const finallyBody: Cfg | undefined = raw.finally !== undefined ? innerParseCfg(parser, cx) : undefined;
        const tryBody: Cfg = cx.withLayer(
          {start: tryStart, end: tryStart + raw.try},
          (cx: ParseContext): Cfg => innerParseCfg(parser, cx),
        );
        let catchBlock: CatchBlock | undefined;
        if (raw.catch !== undefined) {
          const target: CatchTarget = raw.catch.target;
          const body: Cfg = cx.withLayer(
            {start: catchStart, end: catchStart + raw.catch.size},
            (cx: ParseContext): Cfg => innerParseCfg(parser, cx),
          );
          catchBlock = {target, body};
        }
        return {
          type: "flow",
          flow: { type: CfgFlowType.Try, try: tryBody, catch: catchBlock, finally: finallyBody},
        };
      });
    }
    case ActionType.WaitForFrame: {
      const loadingOffset: UintSize = parser.skipFrom(endOffset, raw.skip);
      const loadingTarget: NullableCfgLabel = cx.jump(loadingOffset);
      const readyTarget: NullableCfgLabel = cx.jump(endOffset);
      return {
        type: "flow",
        flow: { type: CfgFlowType.WaitForFrame, frame: raw.frame, loadingTarget, readyTarget},
      };
    }
    case ActionType.WaitForFrame2: {
      const loadingOffset: UintSize = parser.skipFrom(endOffset, raw.skip);
      const loadingTarget: NullableCfgLabel = cx.jump(loadingOffset);
      const readyTarget: NullableCfgLabel = cx.jump(endOffset);
      return {
        type: "flow",
        flow: { type: CfgFlowType.WaitForFrame2, loadingTarget, readyTarget},
      };
    }
    case ActionType.With: {
      return cx.withLayer(
        {start: endOffset, end: endOffset + raw.size},
        (cx: ParseContext): Parsed => {
          const body: Cfg = innerParseCfg(parser, cx);
          return {
            type: "flow",
            flow: { type: CfgFlowType.With, body},
          };
        },
      );
    }
    default: {
      return {
        type: "action",
        end: endOffset,
        action: raw,
      };
    }
  }
}

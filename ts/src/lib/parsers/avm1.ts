import { ReadableBitStream, ReadableByteStream } from "@open-flash/stream";
import { Action } from "avm1-types/action";
import { ActionType } from "avm1-types/action-type";
import * as actions from "avm1-types/actions";
import { CatchTarget } from "avm1-types/catch-target";
import { CatchTargetType } from "avm1-types/catch-targets/_type";
import { GetUrl2Method } from "avm1-types/get-url2-method";
import { Parameter as DefineFunction2Parameter } from "avm1-types/parameter";
import { Value } from "avm1-types/value";
import { ValueType } from "avm1-types/value-type";
import { Incident } from "incident";
import { Uint16, Uint8, UintSize } from "semantic-types";

export interface ActionHeader {
  actionCode: Uint8;
  length: Uint16;
}

export function parseActionHeader(byteStream: ReadableByteStream): ActionHeader {
  const actionCode: Uint8 = byteStream.readUint8();
  const length: Uint16 = actionCode < 0x80 ? 0 : byteStream.readUint16LE();
  return {actionCode, length};
}

// tslint:disable-next-line:cyclomatic-complexity
export function parseAction(byteStream: ReadableByteStream): Action {
  // const startPos: number = byteStream.bytePos;
  const header: ActionHeader = parseActionHeader(byteStream);
  if (byteStream.available() < header.length) {
    // const headerLength: number = byteStream.bytePos - startPos;
    throw new Error("IncompleteStream");
    // throw createIncompleteStreamError(headerLength + header.length);
  }
  const actionDataStartPos: number = byteStream.bytePos;
  let result: Action;
  switch (header.actionCode) {
    case 0x04:
      result = {action: ActionType.NextFrame};
      break;
    case 0x05:
      result = {action: ActionType.PreviousFrame};
      break;
    case 0x06:
      result = {action: ActionType.Play};
      break;
    case 0x07:
      result = {action: ActionType.Stop};
      break;
    case 0x08:
      result = {action: ActionType.ToggleQuality};
      break;
    case 0x09:
      result = {action: ActionType.StopSounds};
      break;
    case 0x0a:
      result = {action: ActionType.Add};
      break;
    case 0x0b:
      result = {action: ActionType.Subtract};
      break;
    case 0x0c:
      result = {action: ActionType.Multiply};
      break;
    case 0x0d:
      result = {action: ActionType.Divide};
      break;
    case 0x0e:
      result = {action: ActionType.Equals};
      break;
    case 0x0f:
      result = {action: ActionType.Less};
      break;
    case 0x10:
      result = {action: ActionType.And};
      break;
    case 0x11:
      result = {action: ActionType.Or};
      break;
    case 0x12:
      result = {action: ActionType.Not};
      break;
    case 0x13:
      result = {action: ActionType.StringEquals};
      break;
    case 0x14:
      result = {action: ActionType.StringLength};
      break;
    case 0x15:
      result = {action: ActionType.StringExtract};
      break;
    case 0x17:
      result = {action: ActionType.Pop};
      break;
    case 0x18:
      result = {action: ActionType.ToInteger};
      break;
    case 0x1c:
      result = {action: ActionType.GetVariable};
      break;
    case 0x1d:
      result = {action: ActionType.SetVariable};
      break;
    case 0x20:
      result = {action: ActionType.SetTarget2};
      break;
    case 0x21:
      result = {action: ActionType.StringAdd};
      break;
    case 0x22:
      result = {action: ActionType.GetProperty};
      break;
    case 0x23:
      result = {action: ActionType.SetProperty};
      break;
    case 0x24:
      result = {action: ActionType.CloneSprite};
      break;
    case 0x25:
      result = {action: ActionType.RemoveSprite};
      break;
    case 0x26:
      result = {action: ActionType.Trace};
      break;
    case 0x27:
      result = {action: ActionType.StartDrag};
      break;
    case 0x28:
      result = {action: ActionType.EndDrag};
      break;
    case 0x29:
      result = {action: ActionType.StringLess};
      break;
    case 0x2a:
      result = {action: ActionType.Throw};
      break;
    case 0x2b:
      result = {action: ActionType.CastOp};
      break;
    case 0x2c:
      result = {action: ActionType.ImplementsOp};
      break;
    case 0x2d:
      result = {action: ActionType.FsCommand2};
      break;
    case 0x30:
      result = {action: ActionType.RandomNumber};
      break;
    case 0x31:
      result = {action: ActionType.MbStringLength};
      break;
    case 0x32:
      result = {action: ActionType.CharToAscii};
      break;
    case 0x33:
      result = {action: ActionType.AsciiToChar};
      break;
    case 0x34:
      result = {action: ActionType.GetTime};
      break;
    case 0x35:
      result = {action: ActionType.MbStringExtract};
      break;
    case 0x36:
      result = {action: ActionType.MbCharToAscii};
      break;
    case 0x37:
      result = {action: ActionType.MbAsciiToChar};
      break;
    case 0x3a:
      result = {action: ActionType.Delete};
      break;
    case 0x3b:
      result = {action: ActionType.Delete2};
      break;
    case 0x3c:
      result = {action: ActionType.DefineLocal};
      break;
    case 0x3d:
      result = {action: ActionType.CallFunction};
      break;
    case 0x3e:
      result = {action: ActionType.Return};
      break;
    case 0x3f:
      result = {action: ActionType.Modulo};
      break;
    case 0x40:
      result = {action: ActionType.NewObject};
      break;
    case 0x41:
      result = {action: ActionType.DefineLocal2};
      break;
    case 0x42:
      result = {action: ActionType.InitArray};
      break;
    case 0x43:
      result = {action: ActionType.InitObject};
      break;
    case 0x44:
      result = {action: ActionType.TypeOf};
      break;
    case 0x45:
      result = {action: ActionType.TargetPath};
      break;
    case 0x46:
      result = {action: ActionType.Enumerate};
      break;
    case 0x47:
      result = {action: ActionType.Add2};
      break;
    case 0x48:
      result = {action: ActionType.Less2};
      break;
    case 0x49:
      result = {action: ActionType.Equals2};
      break;
    case 0x4a:
      result = {action: ActionType.ToNumber};
      break;
    case 0x4b:
      result = {action: ActionType.ToString};
      break;
    case 0x4c:
      result = {action: ActionType.PushDuplicate};
      break;
    case 0x4d:
      result = {action: ActionType.StackSwap};
      break;
    case 0x4e:
      result = {action: ActionType.GetMember};
      break;
    case 0x4f:
      result = {action: ActionType.SetMember};
      break;
    case 0x50:
      result = {action: ActionType.Increment};
      break;
    case 0x51:
      result = {action: ActionType.Decrement};
      break;
    case 0x52:
      result = {action: ActionType.CallMethod};
      break;
    case 0x53:
      result = {action: ActionType.NewMethod};
      break;
    case 0x54:
      result = {action: ActionType.InstanceOf};
      break;
    case 0x55:
      result = {action: ActionType.Enumerate2};
      break;
    case 0x60:
      result = {action: ActionType.BitAnd};
      break;
    case 0x61:
      result = {action: ActionType.BitOr};
      break;
    case 0x62:
      result = {action: ActionType.BitXor};
      break;
    case 0x63:
      result = {action: ActionType.BitLShift};
      break;
    case 0x64:
      result = {action: ActionType.BitRShift};
      break;
    case 0x65:
      result = {action: ActionType.BitURShift};
      break;
    case 0x66:
      result = {action: ActionType.StrictEquals};
      break;
    case 0x67:
      result = {action: ActionType.Greater};
      break;
    case 0x68:
      result = {action: ActionType.StringGreater};
      break;
    case 0x69:
      result = {action: ActionType.Extends};
      break;
    case 0x81:
      result = parseGotoFrameAction(byteStream);
      break;
    case 0x83:
      result = parseGetUrlAction(byteStream);
      break;
    case 0x87:
      result = parseStoreRegisterAction(byteStream);
      break;
    case 0x88:
      result = parseConstantPoolAction(byteStream);
      break;
    case 0x8a:
      result = parseWaitForFrameAction(byteStream);
      break;
    case 0x8b:
      result = parseSetTargetAction(byteStream);
      break;
    case 0x8c:
      result = parseGotoLabelAction(byteStream);
      break;
    case 0x8d:
      result = parseWaitForFrame2Action(byteStream);
      break;
    case 0x8e:
      result = parseDefineFunction2Action(byteStream);
      break;
    case 0x8f:
      result = parseTryAction(byteStream);
      break;
    case 0x94:
      result = parseWithAction(byteStream);
      break;
    case 0x96:
      result = parsePushAction(byteStream.take(header.length));
      break;
    case 0x99:
      result = parseJumpAction(byteStream);
      break;
    case 0x9a:
      result = parseGetUrl2Action(byteStream);
      break;
    case 0x9b:
      result = parseDefineFunctionAction(byteStream);
      break;
    case 0x9d:
      result = parseIfAction(byteStream);
      break;
    case 0x9e:
      result = {action: ActionType.Call};
      break;
    case 0x9f:
      result = parseGotoFrame2Action(byteStream);
      break;
    default:
      result = {action: ActionType.Unknown, code: header.actionCode, data: byteStream.takeBytes(header.length)};
      break;
  }
  const actionDataLength: number = byteStream.bytePos - actionDataStartPos;
  if (actionDataLength < header.length) {
    byteStream.skip(header.length - actionDataLength);
  }

  return result;
}

export function parseGotoFrameAction(byteStream: ReadableByteStream): actions.GotoFrame {
  const frame: Uint16 = byteStream.readUint16LE();
  return {
    action: ActionType.GotoFrame,
    frame,
  };
}

export function parseGetUrlAction(byteStream: ReadableByteStream): actions.GetUrl {
  const url: string = byteStream.readCString();
  const target: string = byteStream.readCString();
  return {
    action: ActionType.GetUrl,
    url,
    target,
  };
}

export function parseStoreRegisterAction(byteStream: ReadableByteStream): actions.StoreRegister {
  const register: Uint8 = byteStream.readUint8();
  return {
    action: ActionType.StoreRegister,
    register,
  };
}

export function parseConstantPoolAction(byteStream: ReadableByteStream): actions.ConstantPool {
  const constantCount: UintSize = byteStream.readUint16LE();
  const constantPool: string[] = [];
  for (let i: number = 0; i < constantCount; i++) {
    constantPool.push(byteStream.readCString());
  }
  return {
    action: ActionType.ConstantPool,
    constantPool,
  };
}

export function parseWaitForFrameAction(byteStream: ReadableByteStream): actions.WaitForFrame {
  const frame: UintSize = byteStream.readUint16LE();
  const skipCount: UintSize = byteStream.readUint8();
  return {
    action: ActionType.WaitForFrame,
    frame,
    skipCount,
  };
}

export function parseSetTargetAction(byteStream: ReadableByteStream): actions.SetTarget {
  const targetName: string = byteStream.readCString();
  return {
    action: ActionType.SetTarget,
    targetName,
  };
}

export function parseGotoLabelAction(byteStream: ReadableByteStream): actions.GotoLabel {
  const label: string = byteStream.readCString();
  return {
    action: ActionType.GotoLabel,
    label,
  };
}

export function parseWaitForFrame2Action(byteStream: ReadableByteStream): actions.WaitForFrame2 {
  const skipCount: UintSize = byteStream.readUint8();
  return {
    action: ActionType.WaitForFrame2,
    skipCount,
  };
}

export function parseDefineFunction2Action(byteStream: ReadableByteStream): actions.DefineFunction2 {
  const name: string = byteStream.readCString();
  const parameterCount: UintSize = byteStream.readUint16LE();
  const registerCount: UintSize = byteStream.readUint8();

  const flags: Uint16 = byteStream.readUint16LE();
  const preloadThis: boolean = (flags & (1 << 0)) !== 0;
  const suppressThis: boolean = (flags & (1 << 1)) !== 0;
  const preloadArguments: boolean = (flags & (1 << 2)) !== 0;
  const suppressArguments: boolean = (flags & (1 << 3)) !== 0;
  const preloadSuper: boolean = (flags & (1 << 4)) !== 0;
  const suppressSuper: boolean = (flags & (1 << 5)) !== 0;
  const preloadRoot: boolean = (flags & (1 << 6)) !== 0;
  const preloadParent: boolean = (flags & (1 << 7)) !== 0;
  const preloadGlobal: boolean = (flags & (1 << 8)) !== 0;
  // Skip 7 bits

  const parameters: DefineFunction2Parameter[] = [];
  for (let i: number = 0; i < parameterCount; i++) {
    const register: Uint8 = byteStream.readUint8();
    const name: string = byteStream.readCString();
    parameters.push({register, name});
  }
  const bodySize: Uint16 = byteStream.readUint16LE();

  return {
    action: ActionType.DefineFunction2,
    name,
    preloadParent,
    preloadRoot,
    suppressSuper,
    preloadSuper,
    suppressArguments,
    preloadArguments,
    suppressThis,
    preloadThis,
    preloadGlobal,
    registerCount,
    parameters,
    bodySize,
  };
}

function parseCatchTarget(byteStream: ReadableByteStream, catchInRegister: boolean): CatchTarget {
  if (catchInRegister) {
    return {type: CatchTargetType.Register, target: byteStream.readUint8()};
  } else {
    return {type: CatchTargetType.Variable, target: byteStream.readCString()};
  }
}

export function parseTryAction(byteStream: ReadableByteStream): actions.Try {
  const flags: Uint8 = byteStream.readUint8();
  const hasCatchBlock: boolean = (flags & (1 << 0)) !== 0;
  const hasFinallyBlock: boolean = (flags & (1 << 1)) !== 0;
  const catchInRegister: boolean = (flags & (1 << 2)) !== 0;
  // (Skip bits [3,7])

  const trySize: Uint16 = byteStream.readUint16LE();
  const catchSize: Uint16 = byteStream.readUint16LE();
  const finallySize: Uint16 = byteStream.readUint16LE();
  const catchTarget: CatchTarget = parseCatchTarget(byteStream, catchInRegister);
  return {
    action: ActionType.Try,
    trySize,
    catchSize: hasCatchBlock ? catchSize : undefined,
    catchTarget,
    finallySize: hasFinallyBlock ? finallySize : undefined,
  };
}

export function parseWithAction(byteStream: ReadableByteStream): actions.With {
  const withSize: Uint16 = byteStream.readUint16LE();
  return {
    action: ActionType.With,
    withSize,
  };
}

export function parsePushAction(byteStream: ReadableByteStream): actions.Push | actions.Error {
  try {
    const values: Value[] = [];
    while (byteStream.available() > 0) {
      values.push(parseActionValue(byteStream));
    }
    return {
      action: ActionType.Push,
      values,
    };
  } catch (error) {
    return {action: ActionType.Error, error};
  }
}

export function parseActionValue(byteStream: ReadableByteStream): Value {
  const typeCode: Uint8 = byteStream.readUint8();
  switch (typeCode) {
    case 0:
      return {type: ValueType.String, value: byteStream.readCString()};
    case 1:
      return {type: ValueType.Float32, value: byteStream.readFloat32LE()};
    case 2:
      return {type: ValueType.Null};
    case 3:
      return {type: ValueType.Undefined};
    case 4:
      return {type: ValueType.Register, value: byteStream.readUint8()};
    case 5:
      return {type: ValueType.Boolean, value: byteStream.readUint8() !== 0};
    case 6:
      return {type: ValueType.Float64, value: byteStream.readFloat64LE()};
    case 7:
      return {type: ValueType.Sint32, value: byteStream.readSint32LE()};
    case 8:
      return {type: ValueType.Constant, value: byteStream.readUint8() as Uint16};
    case 9:
      return {type: ValueType.Constant, value: byteStream.readUint16LE()};
    default:
      throw new Incident("UnknownPushValueTypeCode", {typeCode});
  }
}

export function parseJumpAction(byteStream: ReadableByteStream): actions.Jump {
  const offset: Uint16 = byteStream.readSint16LE();
  return {
    action: ActionType.Jump,
    offset,
  };
}

export function parseGetUrl2Action(byteStream: ReadableByteStream): actions.GetUrl2 {
  const bitStream: ReadableBitStream = byteStream.asBitStream();

  let method: GetUrl2Method;
  switch (bitStream.readUint16Bits(2)) {
    case 0:
      method = GetUrl2Method.None;
      break;
    case 1:
      method = GetUrl2Method.Get;
      break;
    case 2:
      method = GetUrl2Method.Post;
      break;
    default:
      throw new Incident("UnexpectGetUrl2Method", "Unexpected value for the getUrl2 method");
  }
  bitStream.skipBits(4);
  const loadTarget: boolean = bitStream.readBoolBits();
  const loadVariables: boolean = bitStream.readBoolBits();

  bitStream.align();

  return {
    action: ActionType.GetUrl2,
    method,
    loadTarget,
    loadVariables,
  };
}

export function parseDefineFunctionAction(byteStream: ReadableByteStream): actions.DefineFunction {
  const name: string = byteStream.readCString();
  const parameterCount: UintSize = byteStream.readUint16LE();
  const parameters: string[] = [];
  for (let i: number = 0; i < parameterCount; i++) {
    parameters.push(byteStream.readCString());
  }
  const bodySize: Uint16 = byteStream.readUint16LE();

  return {
    action: ActionType.DefineFunction,
    name,
    parameters,
    bodySize,
  };
}

export function parseIfAction(byteStream: ReadableByteStream): actions.If {
  const offset: Uint16 = byteStream.readSint16LE();
  return {
    action: ActionType.If,
    offset,
  };
}

export function parseGotoFrame2Action(byteStream: ReadableByteStream): actions.GotoFrame2 {
  const flags: Uint8 = byteStream.readUint8();
  // (Skip first 6 bits)
  const play: boolean = (flags & (1 << 0)) !== 0;
  const hasSceneBias: boolean = (flags & (1 << 1)) !== 0;
  const sceneBias: Uint16 = hasSceneBias ? byteStream.readUint16LE() : 0;
  return {
    action: ActionType.GotoFrame2,
    play,
    sceneBias,
  };
}

import { ReadableStream } from "@open-flash/stream";
import { ActionType } from "avm1-types/action-type";
import { Action as RawAction } from "avm1-types/raw/action";
import { UintSize } from "semantic-types";
import { ActionHeader, parseAction, parseActionHeader } from "./avm1";
export { parseCfg } from "./cfg";

export class Avm1Parser {
  private readonly stream: ReadableStream;

  constructor(bytes: Uint8Array) {
    this.stream = new ReadableStream(bytes);
  }

  public getBytePos(): UintSize {
    return this.stream.bytePos;
  }

  readNext(): RawAction {
    if (this.stream.bytePos >= this.stream.byteEnd) {
      return {action: ActionType.End};
    } else if (this.stream.peekUint8() === 0) {
      this.stream.bytePos += 1;
      return {action: ActionType.End};
    }
    return parseAction(this.stream);
  }

  readAt(offset: UintSize): RawAction {
    this.stream.bytePos = offset;
    return this.readNext();
  }

  skipFrom(offset: UintSize, skipCount: UintSize): UintSize {
    this.stream.bytePos = offset;
    for (let skipped: UintSize = 0; skipped < skipCount; skipped++) {
      const header: ActionHeader = parseActionHeader(this.stream);
      this.stream.skip(header.length);
    }
    return this.stream.bytePos;
  }
}

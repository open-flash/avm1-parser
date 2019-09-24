import { ReadableStream } from "@open-flash/stream";
import { Action } from "avm1-tree/action";
import { UintSize } from "semantic-types";
import { ActionHeader, parseAction, parseActionHeader } from "./parsers/avm1";
import { ParseError } from "./parsers/parse-error";

export { cfgFromBytes } from "./cfg-from-bytes";

export class Avm1Parser {
  private readonly stream: ReadableStream;

  constructor(bytes: Uint8Array) {
    this.stream = new ReadableStream(bytes);
  }

  public getBytePos(): UintSize {
    return this.stream.bytePos;
  }

  readNext(): Action | ParseError | undefined {
    if (this.stream.bytePos === this.stream.byteEnd) {
      return undefined;
    } else if (this.stream.peekUint8() === 0) {
      this.stream.bytePos += 1;
      return undefined;
    }
    return parseAction(this.stream);
  }

  readAt(offset: UintSize): Action | ParseError | undefined {
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

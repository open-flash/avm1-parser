import { ReadableStream } from "@open-flash/stream";
import { Action } from "avm1-tree";
import { UintSize } from "semantic-types";
import { parseAction } from "./parsers/avm1";

export class Avm1Parser {
  private readonly stream: ReadableStream;

  constructor(bytes: Uint8Array) {
    this.stream = new ReadableStream(bytes);
  }

  public getBytePos(): UintSize {
    return this.stream.bytePos;
  }

  readNext(): Action | undefined {
    return parseAction(this.stream);
  }

  readAt(offset: UintSize): Action | undefined {
    this.stream.bytePos = offset;
    return parseAction(this.stream);
  }
}

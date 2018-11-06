import { ReadableStream } from "@open-flash/stream";
import { Action } from "avm1-tree";
import { parseActionString } from "./parsers/avm1";

export function parseBytes(bytes: Uint8Array): Action[] {
  const byteStream: ReadableStream = new ReadableStream(bytes);
  return parseActionString(byteStream);
}

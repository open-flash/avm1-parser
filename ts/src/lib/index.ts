import { Action } from "avm1-tree";
import { parseActionString } from "./parsers/avm1";
import { Stream } from "./stream";

export function parseBytes(bytes: Uint8Array): Action[] {
  const byteStream: Stream = new Stream(bytes);
  return parseActionString(byteStream);
}

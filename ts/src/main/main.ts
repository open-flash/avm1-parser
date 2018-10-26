import { $Action, Action } from "avm1-tree";
import * as fs from "fs";
import { JsonValueWriter } from "kryo/writers/json-value";
import * as sysPath from "path";
import { parseBytes } from "../lib";

async function main(): Promise<void> {
  if (process.argv.length < 3) {
    console.error("Missing input path");
    return;
  }
  const filePath: string = process.argv[2];
  const absFilePath: string = sysPath.resolve(filePath);
  const data: Buffer = fs.readFileSync(absFilePath);
  const result: Action[] = parseBytes(data);
  const writer: JsonValueWriter = new JsonValueWriter();
  console.log(JSON.stringify(result.map(action => $Action.write(writer, action)), null, 2));
}

main()
  .catch((err: Error): never => {
    console.error(err.stack);
    process.exit(1);
    return undefined as never;
  });

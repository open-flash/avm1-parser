import { $Action, Action } from "avm1-tree/action";
import chai from "chai";
import fs from "fs";
import { JsonReader } from "kryo/readers/json";
import { JsonValueWriter } from "kryo/writers/json-value";
import sysPath from "path";
import { Avm1Parser } from "../lib";
import { ParseError } from "../lib/parsers/parse-error";
import meta from "./meta.js";

const PROJECT_ROOT: string = sysPath.join(meta.dirname, "..", "..", "..");
const TEST_SAMPLES_ROOT: string = sysPath.join(PROJECT_ROOT, "..", "tests");

const JSON_READER: JsonReader = new JsonReader();
const JSON_VALUE_WRITER: JsonValueWriter = new JsonValueWriter();

describe("readJson", function () {
  for (const sample of getSamples()) {
    it(sample.name, async function () {
      const input: Uint8Array = fs.readFileSync(
        sysPath.join(TEST_SAMPLES_ROOT, "actions", `${sample.name}.avm1`),
        {encoding: null},
      );
      const expectedJson: string = fs.readFileSync(
        sysPath.join(TEST_SAMPLES_ROOT, "actions", `${sample.name}.json`),
        {encoding: "UTF-8"},
      );
      const expected: Action = $Action.read(JSON_READER, expectedJson);
      const parser: Avm1Parser = new Avm1Parser(input);
      const uncheckedActual: Action | ParseError | undefined = parser.readNext();
      chai.assert.isDefined(uncheckedActual);
      if (uncheckedActual === undefined || uncheckedActual.action === "error") {
        throw chai.assert.fail("Unexpected ParseError");
      }
      const actual: Action = uncheckedActual!;
      chai.assert.strictEqual(parser.getBytePos(), input.length, "Parsing should consume the whole input");
      try {
        chai.assert.isTrue($Action.equals(actual, expected));
      } catch (err) {
        chai.assert.strictEqual(
          JSON.stringify($Action.write(JSON_VALUE_WRITER, actual), null, 2),
          JSON.stringify($Action.write(JSON_VALUE_WRITER, expected), null, 2),
        );
        throw err;
      }
    });
  }
});

interface Sample {
  name: string;
}

function* getSamples(): IterableIterator<Sample> {
  yield {name: "push-hello-world"};
  yield {name: "trace"};
}

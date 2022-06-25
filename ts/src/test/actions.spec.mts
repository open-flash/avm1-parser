import { $Action, Action } from "avm1-types/raw/action";
import chai from "chai";
import fs from "fs";
import { JSON_READER } from "kryo-json/json-reader";
import { JSON_VALUE_WRITER } from "kryo-json/json-value-writer";
import sysPath from "path";

import { Avm1Parser } from "../lib/index.mjs";
import meta from "./meta.mjs";

const PROJECT_ROOT: string = sysPath.join(meta.dirname, "..");
const TEST_SAMPLES_ROOT: string = sysPath.join(PROJECT_ROOT, "..", "tests");

describe("readJson", function () {
  for (const sample of getSamples()) {
    it(sample.name, async function () {
      const input: Uint8Array = fs.readFileSync(
        sysPath.join(TEST_SAMPLES_ROOT, "actions", `${sample.name}.avm1`),
        {encoding: null},
      );
      const expectedJson: string = fs.readFileSync(
        sysPath.join(TEST_SAMPLES_ROOT, "actions", `${sample.name}.json`),
        {encoding: "utf-8"},
      );
      const expected: Action = $Action.read(JSON_READER, expectedJson);
      const parser: Avm1Parser = new Avm1Parser(input);
      const uncheckedActual: Action | undefined = parser.readNext();
      if (uncheckedActual === undefined) {
        chai.assert.isDefined(uncheckedActual);
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

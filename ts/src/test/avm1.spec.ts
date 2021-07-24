import { $Cfg, Cfg } from "avm1-types/cfg/cfg";
import chai from "chai";
import fs from "fs";
import { JSON_READER } from "kryo-json/json-reader";
import { JSON_VALUE_WRITER } from "kryo-json/json-value-writer";
import sysPath from "path";

import { parseCfg } from "../lib/index.js";
import meta from "./meta.js";
import { readFile, readTextFile, writeTextFile } from "./utils.js";

const PROJECT_ROOT: string = sysPath.join(meta.dirname, "..");
const REPO_ROOT: string = sysPath.join(PROJECT_ROOT, "..");
const AVM1_SAMPLES_ROOT: string = sysPath.join(REPO_ROOT, "tests", "avm1");

// `BLACKLIST` can be used to forcefully skip some tests.
const BLACKLIST: ReadonlySet<string> = new Set([
  // "haxe/hello-world",
  // "wait-for-frame/homestuck-beta2",
]);
// `WHITELIST` can be used to only enable a few tests.
const WHITELIST: ReadonlySet<string> = new Set([
  // "avm1-bytes/corrupted-push",
  // "with/with-shadow",
  // "avm1-bytes/misaligned-jump",
  // "try/try-catch-err",
  // "try/try-ok",
  // "haxe/hello-world",
]);

describe("avm1", function () {
  this.timeout(300000); // The timeout is this high due to CI being extremely slow

  for (const sample of getSamples()) {
    it(sample.name, async function () {
      const inputBytes: Buffer = await readFile(sample.avm1Path);
      const actualCfg: Cfg = parseCfg(inputBytes);
      const testErr: Error | undefined = $Cfg.testError!(actualCfg);
      try {
        chai.assert.isUndefined(testErr, "InvalidCfg");
      } catch (err) {
        console.error(testErr!.toString());
        throw err;
      }
      const actualCfgJson: string = `${JSON.stringify($Cfg.write(JSON_VALUE_WRITER, actualCfg), null, 2)}\n`;
      await writeTextFile(sysPath.join(sample.root, "local-cfg.ts.json"), actualCfgJson);
      const expectedCfgJson: string = await readTextFile(sample.cfgPath);
      const expectedCfg: Cfg = $Cfg.read(JSON_READER, expectedCfgJson);
      try {
        chai.assert.isTrue($Cfg.equals(actualCfg, expectedCfg));
      } catch (err) {
        chai.assert.deepEqual(actualCfgJson, expectedCfgJson);
        throw err;
      }
    });
  }
});

interface Sample {
  root: string;
  name: string;
  avm1Path: string;
  cfgPath: string;
}

function* getSamples(): IterableIterator<Sample> {
  for (const dirEnt of fs.readdirSync(AVM1_SAMPLES_ROOT, {withFileTypes: true})) {
    if (!dirEnt.isDirectory() || dirEnt.name.startsWith(".")) {
      continue;
    }

    const groupName: string = dirEnt.name;
    const groupPath: string = sysPath.join(AVM1_SAMPLES_ROOT, groupName);

    for (const dirEnt of fs.readdirSync(groupPath, {withFileTypes: true})) {
      if (!dirEnt.isDirectory()) {
        continue;
      }
      const testName: string = dirEnt.name;
      const testPath: string = sysPath.join(groupPath, testName);

      const name: string = `${groupName}/${testName}`;

      if (BLACKLIST.has(name)) {
        continue;
      } else if (WHITELIST.size > 0 && !WHITELIST.has(name)) {
        continue;
      }

      const avm1Path: string = sysPath.join(testPath, "main.avm1");
      const cfgPath: string = sysPath.join(testPath, "cfg.json");

      yield {root: testPath, name, avm1Path, cfgPath};
    }
  }
}

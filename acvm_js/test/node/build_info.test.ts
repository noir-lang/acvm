import { expect, test } from "@jest/globals";
import { buildInfo } from "../../pkg/";
import child_process from "child_process";
import pkg from "../../package.json";

test("returns the correct build into", () => {
  const info = buildInfo();

  // TODO: enforce that `package.json` and `Cargo.toml` are consistent.
  expect(info.version).toBe(pkg.version);

  const revision = child_process
    .execSync("git rev-parse HEAD")
    .toString()
    .trim();
  expect(info.git_hash).toBe(revision);
});

import { expect } from "chai";
import { buildInfo } from "../../result/";
import child_process from "child_process";
import pkg from "../../package.json";

it("returns the correct build into", () => {
  const info = buildInfo();

  // TODO: enforce that `package.json` and `Cargo.toml` are consistent.
  expect(info.version).to.be.eq(pkg.version);

  const revision = child_process
    .execSync("git rev-parse HEAD")
    .toString()
    .trim();
  expect(info.git_hash).to.be.eq(revision);
});

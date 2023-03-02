import { instantiate } from "./generated/acvm";
import { readFile } from "fs/promises";
import { resolve as pathResolve } from "path";
import { resolveAcvmWasm } from "./loader_proxy";

export async function loadAcvmWasmModule_byFileRead() {
  async function compileCore(name: string) {
    const buffer = await readFile(pathResolve(__dirname, name));
    return WebAssembly.compile(buffer);
  }

  const imports = {};

  const acvmWasmModule = await instantiate(compileCore, imports);
  resolveAcvmWasm(acvmWasmModule);
}

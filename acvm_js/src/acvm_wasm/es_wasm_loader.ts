import { instantiate } from "./generated/acvm";
import { resolveAcvmWasm } from "./loader_proxy";

export async function loadAcvmWasmModule_byImportMeta() {
  async function compileCore(name: string) {
    const url = new URL("./" + name, import.meta.url);
    return WebAssembly.compileStreaming(fetch(url));
  }

  const imports = {};

  const acvmWasmModule = await instantiate(compileCore, imports);
  resolveAcvmWasm(acvmWasmModule);
}

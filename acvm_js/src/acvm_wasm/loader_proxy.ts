declare type AcvmWasmModule =
  typeof import("./generated/acvm.component")["Acvm"]["component"];

let resolve: (acvmWasmModule: AcvmWasmModule) => void;
const acvmWasmModuleProm: Promise<AcvmWasmModule> = new Promise((r) => {
  resolve = r;
});

export function getAvcmWasmModule() {
  return acvmWasmModuleProm;
}

export function resolveAcvmWasm(acvmWasmModule: AcvmWasmModule) {
  resolve(acvmWasmModule);
}

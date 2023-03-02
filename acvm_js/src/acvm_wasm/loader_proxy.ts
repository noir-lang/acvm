declare type AcvmWasmModule = typeof import("./generated/acvm")["Acvm"];

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

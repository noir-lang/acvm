import { ACIR } from "./acir";
import { getAvcmWasmModule } from "./acvm_wasm/loader_proxy";
import { BlackboxSolvers } from "./blackbox_solvers";
import { InitialWitness, IntermediateWitness } from "./witnesses";

export async function solveIntermediateWitness(
  acir: ACIR,
  initialWitness: InitialWitness,
  blackboxSolvers: BlackboxSolvers
): Promise<IntermediateWitness> {
  const acvm = await getAvcmWasmModule();
  const taskId = acvm.openTask(
    acir,
    Object.entries(initialWitness).map(([idx, value]) => [Number(idx), value])
  );
  while (!acvm.stepTask(taskId)) {
    const [funcName, inputs] = acvm.getBlocker(taskId);
    const solution = await blackboxSolvers[funcName](inputs);
    acvm.unblockTask(taskId, solution);
  }
  const result = acvm.closeTask(taskId);
  return Object.fromEntries(result);
}

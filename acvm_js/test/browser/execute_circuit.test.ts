import { expect } from "@esm-bundle/chai";
import initACVM, {
  executeCircuit,
  WitnessMap,
  initLogLevel,
} from "../../result/";

beforeEach(async () => {
  await initACVM();

  initLogLevel("INFO");
});

it("successfully executes circuit and extracts return value", async () => {
  const { bytecode, initialWitnessMap, resultWitness, expectedResult } =
    await import("../shared/noir_program");

  const solvedWitness: WitnessMap = await executeCircuit(
    bytecode,
    initialWitnessMap,
    () => {
      throw Error("unexpected oracle");
    }
  );

  // Solved witness should be consistent with initial witness
  initialWitnessMap.forEach((value, key) => {
    expect(solvedWitness.get(key) as string).to.be.eq(value);
  });

  // Solved witness should contain expected return value
  expect(solvedWitness.get(resultWitness)).to.be.eq(expectedResult);
});

it("successfully executes a Pedersen opcode", async function () {
  this.timeout(10000);
  const { bytecode, initialWitnessMap, expectedWitnessMap } = await import(
    "../shared/pedersen"
  );

  const solvedWitness: WitnessMap = await executeCircuit(
    bytecode,
    initialWitnessMap,
    () => {
      throw Error("unexpected oracle");
    }
  );

  expect(solvedWitness).to.be.deep.eq(expectedWitnessMap);
});

it("successfully executes a FixedBaseScalarMul opcode", async () => {
  const { bytecode, initialWitnessMap, expectedWitnessMap } = await import(
    "../shared/fixed_base_scalar_mul"
  );

  const solvedWitness: WitnessMap = await executeCircuit(
    bytecode,
    initialWitnessMap,
    () => {
      throw Error("unexpected oracle");
    }
  );

  expect(solvedWitness).to.be.deep.eq(expectedWitnessMap);
});

it("successfully executes a SchnorrVerify opcode", async () => {
  const { bytecode, initialWitnessMap, expectedWitnessMap } = await import(
    "../shared/schnorr_verify"
  );

  const solvedWitness: WitnessMap = await executeCircuit(
    bytecode,
    initialWitnessMap,
    () => {
      throw Error("unexpected oracle");
    }
  );

  expect(solvedWitness).to.be.deep.eq(expectedWitnessMap);
});

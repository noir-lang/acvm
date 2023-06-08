import { expect } from "chai";
import {
  abiEncode,
  abiDecode,
  executeCircuit,
  WitnessMap,
  OracleCallback,
} from "../../result/";

it("successfully executes circuit and extracts return value", async () => {
  const { abi, bytecode, inputs, expectedResult } = await import(
    "../shared/noir_program"
  );

  const initial_witness: WitnessMap = abiEncode(abi, inputs, null);
  const solved_witness: WitnessMap = await executeCircuit(
    bytecode,
    initial_witness,
    () => {
      throw Error("unexpected oracle");
    }
  );

  // Solved witness should be consistent with initial witness
  initial_witness.forEach((value, key) => {
    expect(solved_witness.get(key) as string).to.be.eq(value);
  });

  // Solved witness should contain expected return value
  const return_witness: number = abi.return_witnesses[0];
  expect(solved_witness.get(return_witness)).to.be.eq(expectedResult);

  const decoded_inputs = abiDecode(abi, solved_witness);
  expect(decoded_inputs.return_value).to.be.eq(expectedResult);
});

it("successfully processes oracle opcodes", async () => {
  const {
    bytecode,
    initialWitnessMap,
    expectedWitnessMap,
    oracleResponse,
    oracleCallName,
  } = await import("../shared/oracle");

  let observedName = "";
  let observedInputs: string[] = [];
  const oracleCallback: OracleCallback = async (
    name: string,
    inputs: string[]
  ) => {
    // Throwing inside the oracle callback causes a timeout so we log the observed values
    // and defer the check against expected values until after the execution is complete.
    observedName = name;
    observedInputs = inputs;

    // Witness(1) + Witness(2) = 1 + 1 = 2
    return [oracleResponse];
  };
  const solved_witness: WitnessMap = await executeCircuit(
    bytecode,
    initialWitnessMap,
    oracleCallback
  );

  // Check that expected values were passed to oracle callback.
  expect(observedName).to.be.eq(oracleCallName);
  expect(observedInputs).to.be.deep.eq([
    initialWitnessMap.get(1) as string,
    initialWitnessMap.get(2) as string,
  ]);

  // If incorrect value is written into circuit then execution should halt due to unsatisfied constraint in
  // arithmetic opcode. Nevertheless, check that returned value was inserted correctly.
  expect(solved_witness).to.be.deep.eq(expectedWitnessMap);
});

it("successfully executes a Pedersen opcode", async function () {
  this.timeout(10000);
  const { abi, bytecode, inputs, expectedResult } = await import(
    "../shared/pedersen"
  );

  const initial_witness: WitnessMap = abiEncode(abi, inputs, null);
  const solved_witness: WitnessMap = await executeCircuit(
    bytecode,
    initial_witness,
    () => {
      throw Error("unexpected oracle");
    }
  );

  const decoded_inputs = abiDecode(abi, solved_witness);

  expect(decoded_inputs.return_value).to.be.deep.eq(expectedResult);
});

it("successfully executes a FixedBaseScalarMul opcode", async () => {
  const { abi, bytecode, inputs, expectedResult } = await import(
    "../shared/fixed_base_scalar_mul"
  );

  const initial_witness: WitnessMap = abiEncode(abi, inputs, null);
  const solved_witness: WitnessMap = await executeCircuit(
    bytecode,
    initial_witness,
    () => {
      throw Error("unexpected oracle");
    }
  );

  const decoded_inputs = abiDecode(abi, solved_witness);

  expect(decoded_inputs.return_value).to.be.deep.eq(expectedResult);
});

it("successfully executes a SchnorrVerify opcode", async () => {
  const { abi, bytecode, inputs, expectedResult } = await import(
    "../shared/schnorr_verify"
  );

  const initial_witness: WitnessMap = abiEncode(abi, inputs, null);
  const solved_witness: WitnessMap = await executeCircuit(
    bytecode,
    initial_witness,
    () => {
      throw Error("unexpected oracle");
    }
  );

  const decoded_inputs = abiDecode(abi, solved_witness);
  expect(decoded_inputs.return_value).to.be.eq(expectedResult);
});

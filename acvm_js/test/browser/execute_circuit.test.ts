import { expect } from "@esm-bundle/chai";
import initACVM, {
  abiEncode,
  abiDecode,
  executeCircuit,
  WitnessMap,
  initLogLevel,
} from "../../result/";

beforeEach(async () => {
  await initACVM();

  initLogLevel("INFO");
});

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
    expect(solved_witness.get(key) as string).to.equal(value);
  });
  // Solved witness should contain expected return value
  const return_witness: number = abi.return_witnesses[0];
  expect(solved_witness.get(return_witness)).to.equal(expectedResult);

  const decoded_inputs = abiDecode(abi, solved_witness);

  expect(decoded_inputs.return_value).to.equal(expectedResult);
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

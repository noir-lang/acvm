import { expect, test } from "@jest/globals";
import {
  abiEncode,
  abiDecode,
  executeCircuit,
  WitnessMap,
  OracleCallback,
} from "../../result/";

test("successfully executes circuit and extracts return value", async () => {
  // fn main(x : Field, y : pub Field) -> pub Field {
  //   assert(x != y);
  //   x + y
  // }
  const abi = {
    parameters: [
      { name: "x", type: { kind: "field" }, visibility: "private" },
      { name: "y", type: { kind: "field" }, visibility: "public" },
    ],
    param_witnesses: {
      x: [1],
      y: [2],
    },
    return_type: { kind: "field" },
    return_witnesses: [6],
  };
  const bytecode = Uint8Array.from([
    205, 147, 189, 13, 194, 48, 16, 133, 69, 254, 88, 199, 23, 219, 201, 185, 3,
    137, 134, 49, 72, 184, 8, 23, 80, 88, 86, 122, 111, 128, 99, 196, 8, 72,
    176, 17, 219, 64, 65, 147, 218, 70, 202, 13, 240, 73, 223, 123, 247, 110,
    235, 187, 123, 109, 141, 182, 167, 51, 89, 221, 135, 107, 152, 222, 27, 22,
    119, 176, 250, 50, 56, 107, 132, 160, 182, 38, 224, 112, 96, 181, 234, 80,
    50, 33, 187, 6, 1, 65, 162, 60, 214, 200, 57, 161, 192, 86, 117, 170, 101,
    10, 4, 39, 24, 164, 226, 195, 15, 146, 37, 96, 228, 209, 42, 204, 61, 119,
    218, 80, 111, 245, 72, 238, 177, 191, 140, 100, 236, 148, 23, 179, 200, 124,
    136, 79, 44, 47, 124, 2, 223, 50, 129, 111, 114, 179, 210, 47, 164, 201,
    217, 155, 47, 35, 110, 248, 207, 246, 98, 25, 41, 182, 87, 197, 55, 230, 51,
    95, 125, 0,
  ]);

  const inputs = {
    x: "1",
    y: "2",
  };
  const return_witness: number = abi.return_witnesses[0];

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
    expect(solved_witness.get(key) as string).toBe(value);
  });
  // Solved witness should contain expected return value
  expect(BigInt(solved_witness.get(return_witness) as string)).toBe(3n);

  const decoded_inputs = abiDecode(abi, solved_witness);

  expect(BigInt(decoded_inputs.return_value)).toBe(3n);
});

test("successfully processes oracle opcodes", async () => {
  // We use a handwritten circuit which uses an oracle to calculate the sum of witnesses 1 and 2
  // and stores the result in witness 3. This is then enforced by an arithmetic opcode to check the result is correct.

  // let oracle = OracleData {
  //     name: "example_oracle".to_owned(),
  //     inputs: vec![Witness(1).into(), Witness(2).into()],
  //     input_values: Vec::new(),
  //     outputs: vec![Witness(3)],
  //     output_values: Vec::new(),
  // };
  // let check: Expression = Expression {
  //     mul_terms: Vec::new(),
  //     linear_combinations: vec![
  //         (FieldElement::one(), Witness(1)),
  //         (FieldElement::one(), Witness(2)),
  //         (-FieldElement::one(), Witness(3)),
  //     ],
  //     q_c: FieldElement::zero(),
  // };

  // let circuit = Circuit {
  //     current_witness_index: 4,
  //     opcodes: vec![Opcode::Oracle(oracle), Opcode::Arithmetic(check)],
  //     public_parameters: PublicInputs::default(),
  //     return_values: PublicInputs::default(),
  // };
  const oracle_bytecode = new Uint8Array([
    173, 144, 177, 13, 194, 48, 16, 69, 5, 97, 32, 159, 207, 142, 207, 29, 76,
    192, 8, 200, 14, 23, 17, 41, 17, 40, 74, 65, 155, 13, 28, 27, 86, 160, 160,
    96, 31, 182, 1, 33, 54, 176, 127, 247, 155, 167, 255, 223, 109, 19, 231,
    199, 126, 116, 77, 207, 247, 23, 95, 221, 112, 233, 249, 112, 254, 245, 152,
    194, 18, 223, 91, 145, 23, 88, 101, 35, 68, 153, 33, 235, 252, 33, 97, 169,
    194, 252, 220, 141, 221, 116, 26, 120, 234, 154, 20, 82, 9, 67, 37, 206,
    125, 25, 40, 106, 165, 216, 72, 6, 4, 39, 164, 245, 164, 133, 210, 190, 38,
    32, 208, 164, 143, 146, 16, 153, 20, 25, 235, 173, 17, 22, 20, 50, 180, 218,
    98, 251, 135, 84, 5, 4, 133, 15,
  ]);

  const initial_witness: WitnessMap = new Map();
  initial_witness.set(
    1,
    "0x0000000000000000000000000000000000000000000000000000000000000001"
  );
  initial_witness.set(
    2,
    "0x0000000000000000000000000000000000000000000000000000000000000001"
  );

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
    return ["0x02"];
  };
  const solved_witness: WitnessMap = await executeCircuit(
    oracle_bytecode,
    initial_witness,
    oracleCallback
  );

  // Check that expected values were passed to oracle callback.
  expect(observedName).toBe("example_oracle");
  expect(observedInputs).toStrictEqual([
    initial_witness.get(1) as string,
    initial_witness.get(2) as string,
  ]);

  // If incorrect value is written into circuit then execution should halt due to unsatisfied constraint in
  // arithmetic opcode. Nevertheless, check that returned value was inserted correctly.
  expect(solved_witness.get(3) as string).toBe(
    "0x0000000000000000000000000000000000000000000000000000000000000002"
  );
});

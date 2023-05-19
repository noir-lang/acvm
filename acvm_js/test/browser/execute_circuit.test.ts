import initACVMSimulator, {
  abi_encode,
  abi_decode,
  execute_circuit,
} from "../../pkg/";

test("successfully executes circuit and extracts return value", async () => {
  await initACVMSimulator();

  // Noir program which enforces that x != y and returns x + y.
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
    0, 0, 0, 0, 7, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 1, 0, 0, 0, 6, 0, 0, 0, 6,
    0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 48,
    100, 78, 114, 225, 49, 160, 41, 184, 80, 69, 182, 129, 129, 88, 93, 40, 51,
    232, 72, 121, 185, 112, 145, 67, 225, 245, 147, 240, 0, 0, 0, 2, 0, 0, 0,
    48, 100, 78, 114, 225, 49, 160, 41, 184, 80, 69, 182, 129, 129, 88, 93, 40,
    51, 232, 72, 121, 185, 112, 145, 67, 225, 245, 147, 240, 0, 0, 0, 3, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0, 0, 1, 0, 0, 0, 1,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 0, 0, 0, 4, 0, 0, 0, 48, 100, 78, 114, 225,
    49, 160, 41, 184, 80, 69, 182, 129, 129, 88, 93, 40, 51, 232, 72, 121, 185,
    112, 145, 67, 225, 245, 147, 240, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 3, 0, 0, 0, 5, 0, 0, 0, 48,
    100, 78, 114, 225, 49, 160, 41, 184, 80, 69, 182, 129, 129, 88, 93, 40, 51,
    232, 72, 121, 185, 112, 145, 67, 225, 245, 147, 240, 0, 0, 0, 3, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 48, 100, 78, 114, 225, 49, 160,
    41, 184, 80, 69, 182, 129, 129, 88, 93, 40, 51, 232, 72, 121, 185, 112, 145,
    67, 225, 245, 147, 240, 0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0,
    0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2,
    0, 0, 0, 48, 100, 78, 114, 225, 49, 160, 41, 184, 80, 69, 182, 129, 129, 88,
    93, 40, 51, 232, 72, 121, 185, 112, 145, 67, 225, 245, 147, 240, 0, 0, 0, 6,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
  ]);

  const inputs = {
    x: "1",
    y: "2",
  };
  const return_witness: number = abi.return_witnesses[0];

  const initial_witness: Map<number, string> = abi_encode(abi, inputs, null);
  const solved_witness: Map<number, string> = await execute_circuit(
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

  const decoded_inputs = abi_decode(abi, solved_witness);

  expect(BigInt(decoded_inputs.return_value)).toBe(3n);
});

test("successfully processes oracle opcodes", async () => {
  await initACVMSimulator();

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
    0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 6, 14, 0, 0, 0,
    101, 120, 97, 109, 112, 108, 101, 95, 111, 114, 97, 99, 108, 101, 2, 0, 0,
    0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0,
    48, 100, 78, 114, 225, 49, 160, 41, 184, 80, 69, 182, 129, 129, 88, 93, 40,
    51, 232, 72, 121, 185, 112, 145, 67, 225, 245, 147, 240, 0, 0, 0, 3, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0,
  ]);

  const initial_witness: Map<number, string> = new Map();
  initial_witness.set(
    1,
    "0x0000000000000000000000000000000000000000000000000000000000000001"
  );
  initial_witness.set(
    2,
    "0x0000000000000000000000000000000000000000000000000000000000000001"
  );

  const solved_witness: Map<number, string> = await execute_circuit(
    oracle_bytecode,
    initial_witness,
    async (_name: string, _inputs: string[]) => {
      // We cannot use jest matchers here (or write to a variable in the outside scope) so cannot test that
      // the values for `name` and `inputs` are correct, we can `console.log` them however.
      // console.log(name)
      // console.log(inputs)

      // Witness(1) + Witness(2) = 1 + 1 = 2
      return ["0x02"];
    }
  );

  // If incorrect value is written into circuit then execution should halt due to unsatisfied constraint in
  // arithmetic opcode. Nevertheless, check that returned value was inserted correctly.
  expect(solved_witness.get(3) as string).toBe(
    "0x0000000000000000000000000000000000000000000000000000000000000002"
  );
});

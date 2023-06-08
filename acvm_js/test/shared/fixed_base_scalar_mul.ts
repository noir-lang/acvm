export const abi = {
  parameters: [{ name: "x", type: { kind: "field" }, visibility: "private" }],
  param_witnesses: {
    x: [1],
  },
  return_type: { kind: "array", type: { kind: "field" }, length: 2 },
  return_witnesses: [2, 3],
};

// let fixed_base_scalar_mul = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::FixedBaseScalarMul {
//     input: FunctionInput { witness: Witness(1), num_bits: FieldElement::max_num_bits() },
//     outputs: vec![Witness(2), Witness(3)],
// });

// let circuit = Circuit {
//     current_witness_index: 4,
//     opcodes: vec![fixed_base_scalar_mul],
//     public_parameters: PublicInputs::default(),
//     return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(2), Witness(3)])),
// };
export const bytecode = Uint8Array.from([
  1, 53, 0, 202, 255, 148, 4, 145, 129, 176, 66, 108, 97, 99, 107, 66, 111, 120,
  70, 117, 110, 99, 67, 97, 108, 108, 129, 178, 70, 105, 120, 101, 100, 66, 97,
  115, 101, 83, 99, 97, 108, 97, 114, 77, 117, 108, 146, 146, 1, 204, 254, 146,
  2, 3, 144, 146, 2, 3,
]);

export const inputs = {
  x: "1",
};

export const expectedResult = [
  "0x0000000000000000000000000000000000000000000000000000000000000001",
  "0x0000000000000002cf135e7506a45d632d270d45f1181294833fc48d823f272c",
];

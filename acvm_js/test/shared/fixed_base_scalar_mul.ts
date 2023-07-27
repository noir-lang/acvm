// let fixed_base_scalar_mul = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::FixedBaseScalarMul {
//     input: FunctionInput { witness: Witness(1), num_bits: FieldElement::max_num_bits() },
//     outputs: (Witness(2), Witness(3)),
// });

// let circuit = Circuit {
//     current_witness_index: 4,
//     opcodes: vec![fixed_base_scalar_mul],
//     public_parameters: PublicInputs::default(),
//     return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(2), Witness(3)])),
// };
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 138, 201, 9, 0, 0, 8, 195, 234, 241, 114, 255, 121,
  69, 69, 5, 49, 16, 242, 104, 21, 0, 161, 169, 218, 212, 83, 78, 229, 237, 11, 159, 214, 39,
  0, 55, 132, 28, 78, 72, 0, 0, 0,
]);

export const initialWitnessMap = new Map([
  [1, "0x0000000000000000000000000000000000000000000000000000000000000001"],
]);

export const expectedWitnessMap = new Map([
  [1, "0x0000000000000000000000000000000000000000000000000000000000000001"],
  [2, "0x0000000000000000000000000000000000000000000000000000000000000001"],
  [3, "0x0000000000000002cf135e7506a45d632d270d45f1181294833fc48d823f272c"],
]);

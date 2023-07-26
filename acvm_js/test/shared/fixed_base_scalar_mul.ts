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
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 77, 136, 81, 10, 0, 80, 4, 192, 246, 60,
  95, 238, 127, 94, 33, 138, 213, 90, 77, 129, 71, 83, 181, 169, 167, 146, 126,
  22, 57, 173, 31, 204, 136, 111, 48, 60, 0, 0, 0,
]);

export const initialWitnessMap = new Map([
  [1, "0x0000000000000000000000000000000000000000000000000000000000000001"],
]);

export const expectedWitnessMap = new Map([
  [1, "0x0000000000000000000000000000000000000000000000000000000000000001"],
  [2, "0x0000000000000000000000000000000000000000000000000000000000000001"],
  [3, "0x0000000000000002cf135e7506a45d632d270d45f1181294833fc48d823f272c"],
]);

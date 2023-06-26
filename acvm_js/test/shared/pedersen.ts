// let pedersen = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::Pedersen {
//     inputs: vec![FunctionInput {
//         witness: Witness(1),
//         num_bits: FieldElement::max_num_bits(),
//     }],
//     outputs: vec![Witness(2), Witness(3)],
//     domain_separator: 0,
// });

// let circuit = Circuit {
//     current_witness_index: 4,
//     opcodes: vec![pedersen],
//     public_parameters: PublicInputs::default(),
//     return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(2), Witness(3)])),
// };
export const bytecode = Uint8Array.from([
  1, 45, 0, 210, 255, 148, 4, 145, 129, 176, 66, 108, 97, 99, 107, 66, 111, 120,
  70, 117, 110, 99, 67, 97, 108, 108, 129, 168, 80, 101, 100, 101, 114, 115,
  101, 110, 147, 145, 146, 1, 204, 254, 0, 146, 2, 3, 144, 146, 2, 3,
]);

export const initialWitnessMap = new Map([
  [1, "0x0000000000000000000000000000000000000000000000000000000000000001"],
]);

export const expectedWitnessMap = new Map([
  [1, "0x0000000000000000000000000000000000000000000000000000000000000001"],
  [2, "0x09489945604c9686e698cb69d7bd6fc0cdb02e9faae3e1a433f1c342c1a5ecc4"],
  [3, "0x24f50d25508b4dfb1e8a834e39565f646e217b24cb3a475c2e4991d1bb07a9d8"],
]);

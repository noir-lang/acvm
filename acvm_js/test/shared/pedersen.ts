// let pedersen = Opcode::BlackBoxFuncCall(BlackBoxFuncCall::Pedersen {
//     inputs: vec![FunctionInput {
//         witness: Witness(1),
//         num_bits: FieldElement::max_num_bits(),
//     }],
//     outputs: (Witness(2), Witness(3)),
//     domain_separator: 0,
// });

// let circuit = Circuit {
//     current_witness_index: 4,
//     opcodes: vec![pedersen],
//     public_parameters: PublicInputs::default(),
//     return_values: PublicInputs(BTreeSet::from_iter(vec![Witness(2), Witness(3)])),
// };
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 136, 129, 9, 0, 48, 8, 195, 50, 39,
  254, 127, 177, 168, 40, 8, 6, 74, 219, 40, 240, 104, 170, 237, 124, 159, 45,
  153, 207, 34, 199, 7, 17, 37, 243, 208, 72, 0, 0, 0,
]);

export const initialWitnessMap = new Map([
  [1, "0x0000000000000000000000000000000000000000000000000000000000000001"],
]);

export const expectedWitnessMap = new Map([
  [1, "0x0000000000000000000000000000000000000000000000000000000000000001"],
  [2, "0x09489945604c9686e698cb69d7bd6fc0cdb02e9faae3e1a433f1c342c1a5ecc4"],
  [3, "0x24f50d25508b4dfb1e8a834e39565f646e217b24cb3a475c2e4991d1bb07a9d8"],
]);

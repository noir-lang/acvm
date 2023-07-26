import { WitnessMap } from "../../result/";

// let fe_0 = FieldElement::zero();
// let fe_1 = FieldElement::one();
// let a = Witness(1);
// let b = Witness(2);
// let c = Witness(3);

// let a_times_2 = Witness(4);
// let b_times_3 = Witness(5);
// let c_times_4 = Witness(6);
// let a_plus_b_plus_c = Witness(7);
// let a_plus_b_plus_c_times_2 = Witness(8);

// let brillig_data = Brillig {
//     inputs: vec![
//         // Input Register 0
//         BrilligInputs::Array(vec![
//             Expression {
//                 mul_terms: vec![],
//                 linear_combinations: vec![(fe_1, a)],
//                 q_c: fe_0,
//             },
//             Expression {
//                 mul_terms: vec![],
//                 linear_combinations: vec![(fe_1, b)],
//                 q_c: fe_0,
//             },
//             Expression {
//                 mul_terms: vec![],
//                 linear_combinations: vec![(fe_1, c)],
//                 q_c: fe_0,
//             },
//         ]),
//         // Input Register 1
//         BrilligInputs::Single(Expression {
//             mul_terms: vec![],
//             linear_combinations: vec![(fe_1, a), (fe_1, b), (fe_1, c)],
//             q_c: fe_0,
//         }),
//     ],
//     // This tells the BrilligSolver which witnesses its output registers correspond to
//     outputs: vec![
//         BrilligOutputs::Array(vec![a_times_2, b_times_3, c_times_4]), // Output Register 0
//         BrilligOutputs::Simple(a_plus_b_plus_c),                      // Output Register 1
//         BrilligOutputs::Simple(a_plus_b_plus_c_times_2),              // Output Register 2
//     ],
//     // stack of foreign call/oracle resolutions, starts empty
//     foreign_call_results: vec![],
//     bytecode: vec![
//         // Oracles are named 'foreign calls' in brillig
//         brillig::Opcode::ForeignCall {
//             function: "complex".into(),
//             inputs: vec![
//                 RegisterOrMemory::HeapArray(HeapArray { pointer: 0.into(), size: 3 }),
//                 RegisterOrMemory::RegisterIndex(RegisterIndex::from(1)),
//             ],
//             destinations: vec![
//                 RegisterOrMemory::HeapArray(HeapArray { pointer: 0.into(), size: 3 }),
//                 RegisterOrMemory::RegisterIndex(RegisterIndex::from(1)),
//                 RegisterOrMemory::RegisterIndex(RegisterIndex::from(2)),
//             ],
//         },
//     ],
//     predicate: None,
// };

// let opcodes = vec![Opcode::Brillig(brillig_data)];
// let circuit = Circuit {
//     current_witness_index: 8,
//     opcodes,
//     public_parameters: PublicInputs::default(),
//     return_values: PublicInputs::default(),
// };

export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 83, 91, 10, 128, 32, 16, 28, 221, 30,
  30, 199, 110, 208, 93, 162, 191, 162, 62, 59, 126, 138, 10, 171, 244, 17, 180,
  66, 14, 44, 35, 227, 42, 195, 14, 107, 0, 40, 4, 12, 174, 116, 60, 123, 141,
  144, 35, 245, 205, 145, 237, 55, 76, 74, 238, 47, 91, 203, 163, 110, 192, 35,
  85, 240, 8, 150, 255, 15, 243, 150, 204, 69, 116, 126, 244, 176, 63, 157, 171,
  30, 97, 191, 60, 198, 200, 134, 205, 90, 49, 45, 221, 47, 199, 126, 110, 235,
  69, 69, 11, 143, 166, 212, 117, 193, 111, 222, 100, 184, 1, 94, 206, 212, 58,
  16, 4, 0, 0,
]);
export const initialWitnessMap: WitnessMap = new Map([
  [1, "0x0000000000000000000000000000000000000000000000000000000000000001"],
  [2, "0x0000000000000000000000000000000000000000000000000000000000000002"],
  [3, "0x0000000000000000000000000000000000000000000000000000000000000003"],
]);

export const oracleCallName = "complex";
export const oracleCallInputs = [
  [
    "0x0000000000000000000000000000000000000000000000000000000000000001",
    "0x0000000000000000000000000000000000000000000000000000000000000002",
    "0x0000000000000000000000000000000000000000000000000000000000000003",
  ],
  ["0x0000000000000000000000000000000000000000000000000000000000000006"],
];

export const oracleResponse = [
  [
    "0x0000000000000000000000000000000000000000000000000000000000000002",
    "0x0000000000000000000000000000000000000000000000000000000000000006",
    "0x000000000000000000000000000000000000000000000000000000000000000c",
  ],
  "0x0000000000000000000000000000000000000000000000000000000000000006",
  "0x000000000000000000000000000000000000000000000000000000000000000c",
];

export const expectedWitnessMap = new Map([
  [1, "0x0000000000000000000000000000000000000000000000000000000000000001"],
  [2, "0x0000000000000000000000000000000000000000000000000000000000000002"],
  [3, "0x0000000000000000000000000000000000000000000000000000000000000003"],
  [4, "0x0000000000000000000000000000000000000000000000000000000000000002"],
  [5, "0x0000000000000000000000000000000000000000000000000000000000000006"],
  [6, "0x000000000000000000000000000000000000000000000000000000000000000c"],
  [7, "0x0000000000000000000000000000000000000000000000000000000000000006"],
  [8, "0x000000000000000000000000000000000000000000000000000000000000000c"],
]);

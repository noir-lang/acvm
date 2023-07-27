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
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 213, 83, 219, 10, 128, 48, 8, 245, 210,
  101, 159, 179, 254, 160, 127, 137, 222, 138, 122, 236, 243, 91, 228, 64, 44,
  232, 33, 7, 117, 64, 156, 206, 201, 193, 51, 3, 0, 32, 156, 224, 100, 36, 103,
  148, 88, 35, 215, 245, 226, 227, 59, 116, 232, 215, 43, 150, 226, 72, 63, 224,
  200, 5, 56, 230, 255, 240, 81, 189, 61, 117, 113, 157, 31, 223, 236, 79, 149,
  172, 78, 214, 72, 220, 138, 15, 106, 214, 168, 114, 249, 126, 88, 230, 117,
  26, 55, 54, 37, 90, 26, 155, 39, 227, 31, 223, 232, 230, 4, 215, 157, 63, 176,
  3, 89, 64, 134, 157, 36, 4, 0, 0,
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

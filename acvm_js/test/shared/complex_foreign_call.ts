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
//         brillig_vm::Opcode::ForeignCall {
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
  205, 146, 75, 10, 194, 64, 12, 134, 109, 235, 163, 199, 169, 55, 240, 1, 162,
  91, 61, 65, 209, 48, 12, 196, 105, 25, 93, 212, 101, 110, 48, 51, 209, 35,
  104, 17, 241, 14, 94, 195, 219, 72, 133, 110, 92, 184, 153, 89, 52, 171, 36,
  36, 225, 231, 203, 127, 78, 45, 213, 51, 45, 17, 165, 184, 56, 186, 78, 181,
  206, 79, 204, 198, 186, 247, 36, 243, 139, 113, 228, 125, 34, 11, 35, 36, 238,
  138, 144, 196, 95, 8, 221, 54, 82, 9, 4, 54, 28, 226, 69, 33, 232, 118, 3, 12,
  183, 230, 237, 15, 134, 13, 164, 125, 137, 48, 106, 147, 212, 88, 122, 44, 10,
  13, 82, 168, 121, 142, 200, 245, 182, 104, 250, 21, 211, 125, 9, 121, 249,
  221, 116, 189, 132, 158, 107, 16, 242, 112, 4, 189, 82, 59, 168, 162, 159, 58,
  118, 255, 199, 95, 198, 124, 0,
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

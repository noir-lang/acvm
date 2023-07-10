import { WitnessMap } from "../../result/";

// let fe_0 = FieldElement::zero();
// let fe_1 = FieldElement::one();
// let w_x = Witness(1);
// let w_y = Witness(2);
// let w_oracle = Witness(3);
// let w_z = Witness(4);
// let w_z_inverse = Witness(5);
// let w_x_plus_y = Witness(6);
// let w_equal_res = Witness(7);

// let equal_opcode = brillig_vm::Opcode::BinaryFieldOp {
//     op: BinaryFieldOp::Equals,
//     lhs: RegisterIndex::from(0),
//     rhs: RegisterIndex::from(1),
//     destination: RegisterIndex::from(2),
// };

// let brillig_data = Brillig {
//     inputs: vec![
//         BrilligInputs::Single(Expression {
//             // Input Register 0
//             mul_terms: vec![],
//             linear_combinations: vec![(fe_1, w_x), (fe_1, w_y)],
//             q_c: fe_0,
//         }),
//         BrilligInputs::Single(Expression::default()), // Input Register 1
//     ],
//     // This tells the BrilligSolver which witnesses its output registers correspond to
//     outputs: vec![
//         BrilligOutputs::Simple(w_x_plus_y), // Output Register 0 - from input
//         BrilligOutputs::Simple(w_oracle),   // Output Register 1
//         BrilligOutputs::Simple(w_equal_res), // Output Register 2
//     ],
//     // stack of foreign call/oracle resolutions, starts empty
//     foreign_call_results: vec![],
//     bytecode: vec![
//         equal_opcode,
//         // Oracles are named 'foreign calls' in brillig
//         brillig_vm::Opcode::ForeignCall {
//             function: "invert".into(),
//             destinations: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(1))],
//             inputs: vec![RegisterOrMemory::RegisterIndex(RegisterIndex::from(0))],
//         },
//     ],
//     predicate: None,
// };

// let opcodes = vec![
//     Opcode::Brillig(brillig_data),
//     Opcode::Arithmetic(Expression {
//         mul_terms: vec![],
//         linear_combinations: vec![(fe_1, w_x), (fe_1, w_y), (-fe_1, w_z)],
//         q_c: fe_0,
//     }),
//     Opcode::Directive(Directive::Invert { x: w_z, result: w_z_inverse }),
//     Opcode::Arithmetic(Expression {
//         mul_terms: vec![(fe_1, w_z, w_z_inverse)],
//         linear_combinations: vec![],
//         q_c: -fe_1,
//     }),
//     Opcode::Arithmetic(Expression {
//         mul_terms: vec![],
//         linear_combinations: vec![(-fe_1, w_oracle), (fe_1, w_z_inverse)],
//         q_c: fe_0,
//     }),
// ];
// let circuit = Circuit {
//     current_witness_index: 8,
//     opcodes,
//     public_parameters: PublicInputs::default(),
//     return_values: PublicInputs::default(),
// };

export const bytecode = Uint8Array.from([
  205, 145, 79, 78, 194, 64, 20, 198, 41, 180, 234, 113, 102, 58, 45, 157, 238,
  20, 149, 132, 149, 137, 158, 160, 165, 143, 250, 146, 161, 226, 80, 137, 46,
  231, 6, 195, 12, 92, 161, 70, 13, 119, 240, 26, 222, 198, 63, 145, 16, 216,
  78, 23, 188, 213, 151, 183, 248, 37, 223, 247, 91, 157, 173, 213, 235, 64,
  162, 16, 88, 174, 141, 106, 238, 176, 42, 5, 88, 109, 204, 215, 57, 113, 59,
  234, 181, 192, 232, 58, 35, 200, 174, 148, 118, 135, 217, 95, 218, 116, 38,
  224, 100, 27, 122, 219, 112, 170, 141, 218, 12, 176, 202, 228, 203, 16, 65,
  20, 55, 179, 85, 183, 185, 126, 124, 202, 196, 188, 227, 169, 143, 225, 131,
  4, 44, 171, 203, 76, 8, 219, 96, 181, 0, 89, 47, 213, 230, 22, 74, 156, 215,
  32, 71, 85, 1, 207, 222, 225, 163, 243, 169, 222, 47, 36, 214, 247, 83, 168,
  113, 108, 181, 61, 22, 43, 63, 12, 70, 250, 81, 4, 73, 8, 148, 209, 140, 132,
  105, 206, 99, 18, 197, 121, 159, 83, 78, 99, 30, 23, 33, 103, 12, 120, 196,
  147, 52, 79, 19, 146, 210, 136, 1, 157, 196, 41, 155, 252, 67, 252, 22, 204,
  190, 93, 161, 132, 113, 141, 11, 80, 205, 232, 111, 82, 227, 7, 123, 147, 45,
  173, 123, 91, 63, 208, 238, 117, 247, 69, 154, 22, 6, 236, 181, 32, 50, 112,
  151, 160, 245, 55,
]);
export const initialWitnessMap: WitnessMap = new Map([
  [1, "0x0000000000000000000000000000000000000000000000000000000000000002"],
  [2, "0x0000000000000000000000000000000000000000000000000000000000000003"],
]);

export const oracleCallName = "invert";
export const oracleCallInputs = [
  ["0x0000000000000000000000000000000000000000000000000000000000000005"],
];

export const oracleResponse = [
  "0x135b52945a13d9aa49b9b57c33cd568ba9ae5ce9ca4a2d06e7f3fbd4c6666667",
];

export const expectedWitnessMap = new Map([
  [1, "0x0000000000000000000000000000000000000000000000000000000000000002"],
  [2, "0x0000000000000000000000000000000000000000000000000000000000000003"],
  [3, "0x135b52945a13d9aa49b9b57c33cd568ba9ae5ce9ca4a2d06e7f3fbd4c6666667"],
  [4, "0x0000000000000000000000000000000000000000000000000000000000000005"],
  [5, "0x135b52945a13d9aa49b9b57c33cd568ba9ae5ce9ca4a2d06e7f3fbd4c6666667"],
  [6, "0x0000000000000000000000000000000000000000000000000000000000000005"],
  [7, "0x0000000000000000000000000000000000000000000000000000000000000000"],
]);

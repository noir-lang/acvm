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

// let equal_opcode = brillig::Opcode::BinaryFieldOp {
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
//         brillig::Opcode::ForeignCall {
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
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 181, 147, 75, 10, 195, 48, 12, 68, 149,
  111, 115, 28, 201, 159, 88, 222, 245, 42, 53, 117, 160, 155, 46, 74, 233, 249,
  91, 168, 3, 34, 241, 46, 202, 128, 145, 25, 204, 195, 35, 161, 9, 0, 6, 248,
  107, 252, 157, 22, 246, 90, 189, 107, 169, 120, 76, 212, 40, 178, 90, 61, 22,
  86, 162, 171, 177, 59, 193, 28, 75, 93, 189, 75, 165, 215, 242, 222, 11, 175,
  41, 117, 18, 156, 199, 243, 147, 95, 239, 166, 242, 74, 122, 85, 117, 186, 49,
  79, 27, 173, 197, 217, 185, 28, 76, 38, 75, 55, 52, 49, 177, 71, 231, 211,
  204, 196, 228, 217, 223, 13, 91, 155, 217, 113, 136, 41, 6, 140, 228, 108,
  166, 197, 71, 187, 20, 88, 175, 247, 47, 148, 99, 25, 54, 141, 214, 202, 46,
  217, 160, 216, 7, 129, 220, 173, 245, 81, 118, 167, 152, 127, 128, 243, 214,
  250, 11, 123, 58, 90, 201, 243, 4, 0, 0,
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

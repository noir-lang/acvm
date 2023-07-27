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
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 181, 148, 209, 10, 195, 32, 12, 69, 99,
  109, 183, 126, 78, 82, 181, 198, 183, 253, 202, 100, 22, 246, 178, 135, 49,
  246, 253, 219, 152, 131, 176, 250, 214, 244, 130, 68, 130, 28, 188, 55, 232,
  8, 0, 3, 124, 101, 223, 171, 131, 181, 126, 189, 83, 173, 184, 77, 100, 20,
  89, 157, 30, 11, 27, 214, 213, 216, 86, 48, 15, 34, 239, 143, 142, 141, 172,
  229, 190, 23, 61, 83, 235, 40, 56, 215, 219, 179, 220, 31, 166, 113, 74, 246,
  154, 178, 186, 54, 119, 27, 173, 195, 217, 251, 18, 167, 66, 142, 206, 56,
  165, 204, 1, 125, 200, 51, 19, 83, 224, 112, 153, 216, 185, 194, 158, 99, 202,
  41, 98, 34, 239, 10, 45, 33, 185, 165, 194, 122, 189, 123, 161, 28, 203, 240,
  23, 180, 150, 119, 201, 6, 197, 28, 4, 114, 245, 172, 183, 178, 173, 162, 255,
  97, 135, 121, 25, 104, 127, 111, 47, 112, 131, 248, 45, 3, 5, 0, 0,
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

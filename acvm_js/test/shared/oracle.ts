// We use a handwritten circuit which uses an oracle to calculate the sum of witnesses 1 and 2
// and stores the result in witness 3. This is then enforced by an arithmetic opcode to check the result is correct.

import { WitnessMap } from "../../result/";

// let oracle = OracleData {
//     name: "example_oracle".to_owned(),
//     inputs: vec![Witness(1).into(), Witness(2).into()],
//     input_values: Vec::new(),
//     outputs: vec![Witness(3)],
//     output_values: Vec::new(),
// };
// let check: Expression = Expression {
//     mul_terms: Vec::new(),
//     linear_combinations: vec![
//         (FieldElement::one(), Witness(1)),
//         (FieldElement::one(), Witness(2)),
//         (-FieldElement::one(), Witness(3)),
//     ],
//     q_c: FieldElement::zero(),
// };

// let circuit = Circuit {
//     current_witness_index: 4,
//     opcodes: vec![Opcode::Oracle(oracle), Opcode::Arithmetic(check)],
//     public_parameters: PublicInputs::default(),
//     return_values: PublicInputs::default(),
// };
export const bytecode = Uint8Array.from([
  173, 144, 177, 13, 194, 48, 16, 69, 5, 97, 32, 159, 207, 142, 207, 29, 76,
  192, 8, 200, 14, 23, 17, 41, 17, 40, 74, 65, 155, 13, 28, 27, 86, 160, 160,
  96, 31, 182, 1, 33, 54, 176, 127, 247, 155, 167, 255, 223, 109, 19, 231, 199,
  126, 116, 77, 207, 247, 23, 95, 221, 112, 233, 249, 112, 254, 245, 152, 194,
  18, 223, 91, 145, 23, 88, 101, 35, 68, 153, 33, 235, 252, 33, 97, 169, 194,
  252, 220, 141, 221, 116, 26, 120, 234, 154, 20, 82, 9, 67, 37, 206, 125, 25,
  40, 106, 165, 216, 72, 6, 4, 39, 164, 245, 164, 133, 210, 190, 38, 32, 208,
  164, 143, 146, 16, 153, 20, 25, 235, 173, 17, 22, 20, 50, 180, 218, 98, 251,
  135, 84, 5, 4, 133, 15,
]);

export const initialWitnessMap: WitnessMap = new Map([
  [1, "0x0000000000000000000000000000000000000000000000000000000000000001"],
  [2, "0x0000000000000000000000000000000000000000000000000000000000000001"],
]);

export const oracleCallName = "example_oracle";
export const oracleResponse =
  "0x0000000000000000000000000000000000000000000000000000000000000002";

export const expectedWitnessMap: WitnessMap = new Map(initialWitnessMap).set(
  3,
  "0x0000000000000000000000000000000000000000000000000000000000000002"
);

import { WitnessMap } from "../../result/web/acvm_js";

// let addition = Opcode::Arithmetic(Expression {
//   mul_terms: Vec::new(),
//   linear_combinations: vec![
//       (FieldElement::one(), Witness(1)),
//       (FieldElement::one(), Witness(2)),
//       (-FieldElement::one(), Witness(3)),
//   ],
//   q_c: FieldElement::zero(),
// });

// let circuit = Circuit {
//   current_witness_index: 4,
//   opcodes: vec![addition],
//   private_parameters: BTreeSet::from([Witness(1), Witness(2)]),
//   public_parameters: PublicInputs::default(),
//   return_values: PublicInputs([Witness(3)].into()),
// };
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 144, 187, 13, 192, 32, 12, 68, 249,
  100, 32, 27, 219, 96, 119, 89, 37, 40, 176, 255, 8, 81, 36, 23, 72, 41, 195,
  53, 215, 61, 221, 189, 35, 132, 16, 195, 55, 217, 251, 244, 134, 127, 193,
  184, 145, 149, 22, 22, 65, 101, 30, 173, 12, 36, 188, 160, 88, 87, 1, 150, 94,
  21, 21, 69, 229, 46, 74, 52, 148, 181, 89, 183, 6, 134, 76, 3, 167, 24, 77,
  135, 229, 125, 187, 32, 57, 231, 253, 154, 22, 151, 113, 113, 250, 0, 123, 50,
  20, 220, 112, 1, 0, 0,
]);

export const initialWitnessMap: WitnessMap = new Map([
  [1, "0x0000000000000000000000000000000000000000000000000000000000000001"],
  [2, "0x0000000000000000000000000000000000000000000000000000000000000002"],
]);

export const resultWitness = 3;
export const expectedResult =
  "0x0000000000000000000000000000000000000000000000000000000000000003";

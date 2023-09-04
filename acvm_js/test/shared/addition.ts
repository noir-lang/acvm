import { WitnessMap } from "../../../result/";

// See `addition_circuit` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 144, 187, 13, 192, 32, 12, 68, 249,
  100, 32, 27, 219, 96, 119, 89, 37, 40, 176, 255, 8, 17, 18, 5, 74, 202, 240,
  154, 235, 158, 238, 238, 112, 206, 121, 247, 37, 206, 60, 103, 194, 63, 208,
  111, 116, 133, 197, 69, 144, 153, 91, 73, 13, 9, 47, 72, 86, 85, 128, 165,
  102, 69, 69, 81, 185, 147, 18, 53, 101, 45, 86, 173, 128, 33, 83, 195, 46, 70,
  125, 202, 226, 190, 94, 16, 166, 103, 108, 13, 203, 151, 254, 245, 233, 224,
  1, 1, 52, 166, 127, 120, 1, 0, 0,
]);

export const initialWitnessMap: WitnessMap = new Map([
  [1, "0x0000000000000000000000000000000000000000000000000000000000000001"],
  [2, "0x0000000000000000000000000000000000000000000000000000000000000002"],
]);

export const resultWitness = 3;
export const expectedResult =
  "0x0000000000000000000000000000000000000000000000000000000000000003";

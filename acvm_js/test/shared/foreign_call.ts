import { WitnessMap } from "../../../result/";

// See `simple_brillig_foreign_call` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 173, 143, 65, 10, 0, 32, 8, 4, 205, 32,
  122, 142, 253, 160, 207, 116, 232, 210, 33, 162, 247, 23, 100, 96, 32, 93,
  106, 64, 92, 92, 144, 93, 15, 0, 6, 22, 86, 104, 201, 190, 69, 222, 244, 70,
  48, 255, 126, 145, 204, 139, 74, 102, 63, 199, 177, 206, 165, 167, 218, 110,
  13, 15, 80, 152, 168, 248, 3, 190, 43, 105, 200, 59, 1, 0, 0,
]);
export const initialWitnessMap: WitnessMap = new Map([
  [1, "0x0000000000000000000000000000000000000000000000000000000000000005"],
]);

export const oracleCallName = "invert";
export const oracleCallInputs = [
  ["0x0000000000000000000000000000000000000000000000000000000000000005"],
];

export const oracleResponse = [
  "0x135b52945a13d9aa49b9b57c33cd568ba9ae5ce9ca4a2d06e7f3fbd4c6666667",
];

export const expectedWitnessMap = new Map([
  [1, "0x0000000000000000000000000000000000000000000000000000000000000005"],
  [2, "0x135b52945a13d9aa49b9b57c33cd568ba9ae5ce9ca4a2d06e7f3fbd4c6666667"],
]);

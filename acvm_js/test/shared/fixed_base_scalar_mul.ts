// See `fixed_base_scalar_mul_circuit` integration test in `acir/tests/test_program_serialization.rs`.
export const bytecode = Uint8Array.from([
  31, 139, 8, 0, 0, 0, 0, 0, 0, 255, 93, 202, 65, 10, 0, 64, 8, 2, 64, 183, 246,
  212, 255, 223, 27, 21, 21, 72, 130, 12, 136, 31, 192, 67, 167, 180, 209, 73,
  201, 234, 249, 109, 132, 84, 218, 3, 23, 46, 165, 61, 88, 0, 0, 0,
]);
export const initialWitnessMap = new Map([
  [1, "0x0000000000000000000000000000000000000000000000000000000000000001"],
]);

export const expectedWitnessMap = new Map([
  [1, "0x0000000000000000000000000000000000000000000000000000000000000001"],
  [2, "0x0000000000000000000000000000000000000000000000000000000000000001"],
  [3, "0x0000000000000002cf135e7506a45d632d270d45f1181294833fc48d823f272c"],
]);

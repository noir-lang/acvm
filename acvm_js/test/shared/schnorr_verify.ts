export const abi = {
  parameters: [
    { name: "x", type: { kind: "field" }, visibility: "private" },
    { name: "y", type: { kind: "field" }, visibility: "private" },
    {
      name: "sig",
      type: {
        kind: "array",
        type: { kind: "integer", width: 8, sign: "unsigned" },
        length: 64,
      },
      visibility: "private",
    },
    {
      name: "msg",
      type: {
        kind: "array",
        type: { kind: "integer", width: 8, sign: "unsigned" },
        length: 10,
      },
      visibility: "private",
    },
  ],
  param_witnesses: {
    x: [1],
    y: [2],
    sig: Array.from({ length: 64 }, (_, i) => 3 + i),
    msg: Array.from({ length: 10 }, (_, i) => 3 + 64 + i),
  },
  return_type: { kind: "boolean" },
  return_witnesses: [3 + 10 + 64],
};

// let public_key_x = FunctionInput { witness: Witness(1), num_bits: FieldElement:: max_num_bits() };
// let public_key_y = FunctionInput { witness: Witness(2), num_bits: FieldElement:: max_num_bits() };
// let signature =
//   (3..(3 + 64)).map(| i | FunctionInput { witness: Witness(i), num_bits: 8 }).collect();
// let message = ((3 + 64)..(3 + 64 + 10))
//   .map(| i | FunctionInput { witness: Witness(i), num_bits: 8 })
//   .collect();
// let output = Witness(3 + 64 + 10);

// let schnorr = Opcode:: BlackBoxFuncCall(BlackBoxFuncCall:: SchnorrVerify {
//   public_key_x,
//   public_key_y,
//   signature,
//   message,
//   output,
// });

// let circuit = Circuit {
//   current_witness_index: 100,
//   opcodes: vec![schnorr],
//     public_parameters: PublicInputs::default(),
//     return_values: PublicInputs(BTreeSet::from_iter(vec![output])),
// };
export const bytecode = Uint8Array.from([
  5, 128, 71, 74, 3, 1, 0, 0, 237, 142, 189, 247, 222, 123, 239, 61, 38, 26,
  107, 78, 130, 247, 176, 42, 138, 75, 2, 11, 130, 30, 243, 3, 51, 232, 197, 63,
  8, 62, 36, 79, 240, 45, 33, 124, 61, 228, 115, 127, 241, 48, 29, 188, 198,
  179, 239, 201, 183, 76, 144, 72, 135, 97, 238, 247, 46, 120, 206, 100, 163,
  232, 254, 49, 122, 121, 250, 248, 182, 188, 80, 180, 162, 80, 252, 47, 139,
  89, 137, 85, 88, 141, 53, 88, 139, 96, 29, 214, 99, 3, 54, 98, 19, 54, 99, 11,
  182, 98, 27, 182, 99, 7, 118, 98, 23, 118, 99, 15, 246, 98, 31, 246, 227, 0,
  14, 226, 16, 14, 227, 8, 142, 226, 24, 142, 227, 4, 78, 226, 20, 78, 227, 12,
  206, 226, 28, 206, 227, 2, 46, 226, 18, 46, 227, 10, 174, 226, 26, 174, 227,
  6, 110, 226, 22, 110, 227, 14, 238, 226, 30, 238, 227, 1, 30, 226, 17, 30,
  227, 9, 198, 240, 20, 227, 252, 152, 192, 51, 60, 199, 36, 94, 224, 37, 94,
  225, 53, 222, 224, 45, 169, 207, 124, 170, 4,
]);

export const inputs = {
  x: "0x17cbd3ed3151ccfd170efe1d54280a6a4822640bf5c369908ad74ea21518a9c5",
  y: "0x0e0456e3795c1a31f20035b741cd6158929eeccd320d299cfcac962865a6bc74",
  sig: [
    5, 202, 31, 146, 81, 242, 246, 69, 43, 107, 249, 153, 198, 44, 14, 111, 191,
    121, 137, 166, 160, 103, 18, 181, 243, 233, 226, 95, 67, 16, 37, 128, 85,
    76, 19, 253, 30, 77, 192, 53, 138, 205, 69, 33, 236, 163, 83, 194, 84, 137,
    184, 221, 176, 121, 179, 27, 63, 70, 54, 16, 176, 250, 39, 239,
  ],
  msg: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
};

export const expectedResult =
  "0x0000000000000000000000000000000000000000000000000000000000000001";

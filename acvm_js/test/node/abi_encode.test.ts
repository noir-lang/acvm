import { expect, test } from "@jest/globals";
import { abiEncode, abiDecode, WitnessMap } from "../../result/";
import { DecodedInputs } from "../types";

test("recovers original inputs when abi encoding and decoding", () => {
  // TODO use ts-rs to get ABI type bindings.
  const abi = {
    parameters: [
      { name: "foo", type: { kind: "field" }, visibility: "private" },
      {
        name: "bar",
        type: { kind: "array", length: 2, type: { kind: "field" } },
        visibility: "private",
      },
    ],
    param_witnesses: { foo: [1], bar: [2, 3] },
    return_type: null,
    return_witnesses: [],
  };
  const inputs = {
    foo: "1",
    bar: ["1", "2"],
  };
  const initial_witness: WitnessMap = abiEncode(abi, inputs, null);
  const decoded_inputs: DecodedInputs = abiDecode(abi, initial_witness);

  expect(BigInt(decoded_inputs.inputs.foo)).toBe(BigInt(inputs.foo));
  expect(BigInt(decoded_inputs.inputs.bar[0])).toBe(BigInt(inputs.bar[0]));
  expect(BigInt(decoded_inputs.inputs.bar[1])).toBe(BigInt(inputs.bar[1]));
  expect(decoded_inputs.return_value).toBe(null);
});

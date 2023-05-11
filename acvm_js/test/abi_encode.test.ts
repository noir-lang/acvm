import { test } from "@jest/globals"
import { abi_encode, abi_decode } from "../pkg/"

test('recovers original inputs when abi encoding and decoding', () => {
  // TODO use ts-rs to get ABI type bindings.
  const abi = {
    parameters: [
      {name: "foo", type: { kind: "field" }, visibility: "private"},
      {name: "bar", type: { kind: "array", length: 2, type: { kind: "field" } }, visibility: "private"}
    ],
    param_witnesses: {"foo": [1], "bar": [2, 3]},
    return_type: null,
    return_witnesses: []
  };
  const inputs = { 
    foo: "1",
    bar: ["1", "2"]
  };
  const initial_witness = abi_encode(abi, inputs, null);
  const decoded_inputs = abi_decode(abi, initial_witness);
  console.log(decoded_inputs);
});
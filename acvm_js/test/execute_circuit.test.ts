import { expect, test } from "@jest/globals"
import { abi_encode, abi_decode, execute_circuit } from "../pkg/"

// Noir program which enforces that x != y and returns x + y.
const abi = {
  parameters:[
    { name:"x", type: { kind: "field" }, visibility:"private" },
    { name:"y", type: { kind: "field" }, visibility:"public" }
  ],
  param_witnesses:
    {
      x: [1],
      y: [2]
    },
  return_type: { kind: "field" },
  return_witnesses: [6]
};
const bytecode = Uint8Array.from([0,0,0,0,7,0,0,0,1,0,0,0,2,0,0,0,1,0,0,0,6,0,0,0,6,0,0,0,0,0,0,0,0,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,0,0,0,48,100,78,114,225,49,160,41,184,80,69,182,129,129,88,93,40,51,232,72,121,185,112,145,67,225,245,147,240,0,0,0,2,0,0,0,48,100,78,114,225,49,160,41,184,80,69,182,129,129,88,93,40,51,232,72,121,185,112,145,67,225,245,147,240,0,0,0,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,0,0,3,0,0,0,4,0,0,0,0,1,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,0,0,0,4,0,0,0,48,100,78,114,225,49,160,41,184,80,69,182,129,129,88,93,40,51,232,72,121,185,112,145,67,225,245,147,240,0,0,0,5,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,3,0,0,0,5,0,0,0,48,100,78,114,225,49,160,41,184,80,69,182,129,129,88,93,40,51,232,72,121,185,112,145,67,225,245,147,240,0,0,0,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,48,100,78,114,225,49,160,41,184,80,69,182,129,129,88,93,40,51,232,72,121,185,112,145,67,225,245,147,240,0,0,0,5,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,3,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1,2,0,0,0,48,100,78,114,225,49,160,41,184,80,69,182,129,129,88,93,40,51,232,72,121,185,112,145,67,225,245,147,240,0,0,0,6,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]);

test('recovers original inputs when abi encoding and decoding', () => {
  const inputs = { 
    x: "1",
    y: "2"
  };
  const return_witness: string = abi.return_witnesses[0].toString()

  const initial_witness: Map<string, string> = abi_encode(abi, inputs, null);
  const solved_witness: Map<string, string> = execute_circuit(bytecode, initial_witness)
  
  // Solved witness should be consistent with initial witness
  initial_witness.forEach((value, key) => {
    expect(solved_witness.get(key) as string).toBe(value)

  }) 
  // Solved witness should contain expected return value
  expect(BigInt(solved_witness.get(return_witness) as string)).toBe(3n)

  const decoded_inputs = abi_decode(abi, solved_witness);

  expect(BigInt(decoded_inputs.return_value)).toBe(3n)
});


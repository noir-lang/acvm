import { expect } from "@esm-bundle/chai";
import initACVMSimulator, {
  compressWitness,
  decompressWitness,
} from "../../result/";
import {
  expectedCompressedWitnessMap,
  expectedWitnessMap,
} from "../shared/witness_compression";

it("successfully compresses the witness", async () => {
  await initACVMSimulator();

  const compressedWitnessMap = compressWitness(expectedWitnessMap);

  expect(compressedWitnessMap).to.be.deep.eq(expectedCompressedWitnessMap);
});

it("successfully decompresses the witness", async () => {
  await initACVMSimulator();

  const witnessMap = decompressWitness(expectedCompressedWitnessMap);

  expect(witnessMap).to.be.deep.eq(expectedWitnessMap);
});

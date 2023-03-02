import { FieldElement } from "./field_element";

type WitnessIndex = number;
export type InitialWitness = Record<WitnessIndex, FieldElement>;
export type IntermediateWitness = Record<WitnessIndex, FieldElement>;
export type PublicWitness = Record<WitnessIndex, FieldElement>;

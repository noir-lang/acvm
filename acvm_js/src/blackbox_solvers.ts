import { BlackBoxFuncName } from "./blackbox_func";
import { FieldElement } from "./field_element";

export interface BlackboxSolvers {
  // The following black boxes are already solved deterministically in
  // ACVM's acvm/acvm/src/stepwise_pwg/attempt_blackbox.rs:
  // - SHA256
  // - Blake2s
  // - EcdsaSecp256k1
  // - AND
  // - XOR
  // - RANGE

  [BlackBoxFuncName.MerkleMembership]?: (
    xs: FieldElement[]
  ) => Promise<FieldElement>;

  [BlackBoxFuncName.SchnorrVerify]?: (
    xs: FieldElement[]
  ) => Promise<FieldElement>;

  [BlackBoxFuncName.Pedersen]?: (
    xs: FieldElement[]
  ) => Promise<[FieldElement, FieldElement]>;

  [BlackBoxFuncName.HashToField128Security]?: (
    xs: FieldElement[]
  ) => Promise<FieldElement>;

  [BlackBoxFuncName.FixedBaseScalarMul]?: (
    xs: FieldElement
  ) => Promise<[FieldElement, FieldElement]>;
}

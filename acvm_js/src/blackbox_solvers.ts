import { BlackBoxFunc } from "./blackbox_func";
import { FieldElement } from "./field_element";

export interface BlackboxSolvers {
  [BlackBoxFunc.AND]?: (
    x1: FieldElement,
    x2: FieldElement
  ) => Promise<FieldElement>;

  [BlackBoxFunc.XOR]?: (
    x1: FieldElement,
    x2: FieldElement
  ) => Promise<FieldElement>;

  [BlackBoxFunc.RANGE]?: () => Promise<[]>;

  [BlackBoxFunc.SHA256]?: (xs: FieldElement[]) => Promise<FieldElement[]>;

  [BlackBoxFunc.Blake2s]?: (xs: FieldElement[]) => Promise<FieldElement[]>;

  [BlackBoxFunc.MerkleMembership]?: (
    xs: FieldElement[]
  ) => Promise<FieldElement>;

  [BlackBoxFunc.SchnorrVerify]?: (xs: FieldElement[]) => Promise<FieldElement>;

  [BlackBoxFunc.Pedersen]?: (
    xs: FieldElement[]
  ) => Promise<[FieldElement, FieldElement]>;

  [BlackBoxFunc.HashToField128Security]?: (
    xs: FieldElement[]
  ) => Promise<FieldElement>;

  [BlackBoxFunc.EcdsaSecp256k1]?: (xs: FieldElement[]) => Promise<FieldElement>;

  [BlackBoxFunc.FixedBaseScalarMul]?: (
    xs: FieldElement
  ) => Promise<[FieldElement, FieldElement]>;
}

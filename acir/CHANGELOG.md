# Changelog

## [0.10.0](https://github.com/noir-lang/acvm/compare/acir-v0.9.0...acir-v0.10.0) (2023-04-26)


### ⚠ BREAKING CHANGES

* replace `MerkleMembership` opcode with `ComputeMerkleRoot` ([#233](https://github.com/noir-lang/acvm/issues/233))
* return `PartialWitnessGeneratorStatus` from `PartialWitnessGenerator.solve` ([#213](https://github.com/noir-lang/acvm/issues/213))
* organise operator implementations for Expression ([#190](https://github.com/noir-lang/acvm/issues/190))

### Features

* implement `add_mul` on `Expression` ([#207](https://github.com/noir-lang/acvm/issues/207)) ([f156e18](https://github.com/noir-lang/acvm/commit/f156e18cf7a0f1a99bbe1683b8e75fec8325e6dd))
* replace `MerkleMembership` opcode with `ComputeMerkleRoot` ([#233](https://github.com/noir-lang/acvm/issues/233)) ([74bfee8](https://github.com/noir-lang/acvm/commit/74bfee80e0ff0d205aee1eea548c97ade8bd0e41))
* return `PartialWitnessGeneratorStatus` from `PartialWitnessGenerator.solve` ([#213](https://github.com/noir-lang/acvm/issues/213)) ([e877bed](https://github.com/noir-lang/acvm/commit/e877bed2cca76bd486e9bed66b4230e65a01f0a2))


### Miscellaneous Chores

* organise operator implementations for Expression ([#190](https://github.com/noir-lang/acvm/issues/190)) ([a619df6](https://github.com/noir-lang/acvm/commit/a619df614bbb9b2518b788b42a7553b069823a0f))

## [0.9.0](https://github.com/noir-lang/acvm/compare/acir-v0.8.1...acir-v0.9.0) (2023-04-07)


### Bug Fixes

* Add test for Out of Memory  ([#188](https://github.com/noir-lang/acvm/issues/188)) ([c3db985](https://github.com/noir-lang/acvm/commit/c3db985893e7e59ea04005bb3a57eda5c6ce28c7))

## [0.8.1](https://github.com/noir-lang/acvm/compare/acir-v0.8.0...acir-v0.8.1) (2023-03-30)


### Bug Fixes

* unwraps if inputs is zero ([#171](https://github.com/noir-lang/acvm/issues/171)) ([10a3bb2](https://github.com/noir-lang/acvm/commit/10a3bb2a9930ccf422b3f08227aae07775686860))

## [0.8.0](https://github.com/noir-lang/acvm/compare/acir-v0.7.1...acir-v0.8.0) (2023-03-28)


### ⚠ BREAKING CHANGES

* **acir:** Read Log Directive ([#156](https://github.com/noir-lang/acvm/issues/156))

### Bug Fixes

* **acir:** Read Log Directive ([#156](https://github.com/noir-lang/acvm/issues/156)) ([1cc2b7f](https://github.com/noir-lang/acvm/commit/1cc2b7f2179cecc338fe0def72bb2dd17eaed0cd))

## [0.7.1](https://github.com/noir-lang/acvm/compare/acir-v0.7.0...acir-v0.7.1) (2023-03-27)


### Miscellaneous Chores

* **acir:** Synchronize undefined versions

## [0.7.0](https://github.com/noir-lang/acvm/compare/acir-v0.6.0...acir-v0.7.0) (2023-03-23)


### ⚠ BREAKING CHANGES

* Add initial oracle opcode ([#149](https://github.com/noir-lang/acvm/issues/149))
* **acir:** Add RAM and ROM opcodes
* **acir:** Add a public outputs field ([#56](https://github.com/noir-lang/acvm/issues/56))
* **acir:** remove `Linear` struct ([#145](https://github.com/noir-lang/acvm/issues/145))
* **acvm:** Remove truncate and oddrange directives ([#142](https://github.com/noir-lang/acvm/issues/142))

### Features

* **acir:** Add a public outputs field ([#56](https://github.com/noir-lang/acvm/issues/56)) ([5f358a9](https://github.com/noir-lang/acvm/commit/5f358a97aaa81d87956e182cd8a6d60de75f9752))
* **acir:** Add RAM and ROM opcodes ([73e9f25](https://github.com/noir-lang/acvm/commit/73e9f25dd87b2ca91245e93d2445eadc0f522fac))
* Add initial oracle opcode ([#149](https://github.com/noir-lang/acvm/issues/149)) ([88ee2f8](https://github.com/noir-lang/acvm/commit/88ee2f89f37abf5dd1d9f91b4d2eed44dc651348))


### Miscellaneous Chores

* **acir:** remove `Linear` struct ([#145](https://github.com/noir-lang/acvm/issues/145)) ([bbb6d92](https://github.com/noir-lang/acvm/commit/bbb6d92e25c43dd33b12f5fcd639fc9ad2a9c9d8))
* **acvm:** Remove truncate and oddrange directives ([#142](https://github.com/noir-lang/acvm/issues/142)) ([85dd6e8](https://github.com/noir-lang/acvm/commit/85dd6e85bfba85bfb97651f7e30e1f75deb986d5))

## [0.6.0](https://github.com/noir-lang/acvm/compare/acir-v0.5.0...acir-v0.6.0) (2023-03-03)


### ⚠ BREAKING CHANGES

* **acir:** rename `term_addition` to `push_addition_term`
* **acir:** rename `term_multiplication` to `push_multiplication_term` ([#122](https://github.com/noir-lang/acvm/issues/122))
* **acir:** remove `UnknownWitness` ([#123](https://github.com/noir-lang/acvm/issues/123))
* add block opcode ([#114](https://github.com/noir-lang/acvm/issues/114))

### Features

* **acir:** add useful methods from `noirc_evaluator` onto `Expression` ([#125](https://github.com/noir-lang/acvm/issues/125)) ([d3d5f89](https://github.com/noir-lang/acvm/commit/d3d5f8917482ce5649602695829862a5df4ea712))
* add block opcode ([#114](https://github.com/noir-lang/acvm/issues/114)) ([097cfb0](https://github.com/noir-lang/acvm/commit/097cfb069291705ddb4bf1fca77ddcef21dbbd08))


### Bug Fixes

* **acir:** correctly display expressions with non-unit coefficients ([d3d5f89](https://github.com/noir-lang/acvm/commit/d3d5f8917482ce5649602695829862a5df4ea712))


### Miscellaneous Chores

* **acir:** remove `UnknownWitness` ([#123](https://github.com/noir-lang/acvm/issues/123)) ([9f002c7](https://github.com/noir-lang/acvm/commit/9f002c7b49a5cf222d4a01732cc4917a47690863))
* **acir:** rename `term_addition` to `push_addition_term` ([d389385](https://github.com/noir-lang/acvm/commit/d38938542851a97dc01727438391e6a65e44c689))
* **acir:** rename `term_multiplication` to `push_multiplication_term` ([#122](https://github.com/noir-lang/acvm/issues/122)) ([d389385](https://github.com/noir-lang/acvm/commit/d38938542851a97dc01727438391e6a65e44c689))

## [0.5.0](https://github.com/noir-lang/acvm/compare/acir-v0.4.1...acir-v0.5.0) (2023-02-22)


### ⚠ BREAKING CHANGES

* **acir:** make PublicInputs use a BTreeSet rather than Vec ([#99](https://github.com/noir-lang/acvm/issues/99))
* refactor ToRadix to ToRadixLe and ToRadixBe ([#58](https://github.com/noir-lang/acvm/issues/58))
* **acir:** Add keccak256 Opcode ([#91](https://github.com/noir-lang/acvm/issues/91))
* reorganise compiler in terms of optimisers and transformers ([#88](https://github.com/noir-lang/acvm/issues/88))

### Features

* **acir:** Add keccak256 Opcode ([#91](https://github.com/noir-lang/acvm/issues/91)) ([b909146](https://github.com/noir-lang/acvm/commit/b9091461e199bacdd073cc9b31f03dade0b4fb2d))
* **acir:** make PublicInputs use a BTreeSet rather than Vec ([#99](https://github.com/noir-lang/acvm/issues/99)) ([53666b7](https://github.com/noir-lang/acvm/commit/53666b782d89c65cd755f9e4ded2c9cf5a141e46))


### Miscellaneous Chores

* refactor ToRadix to ToRadixLe and ToRadixBe ([#58](https://github.com/noir-lang/acvm/issues/58)) ([2427a27](https://github.com/noir-lang/acvm/commit/2427a275048e598c6d651cce8348a4c55148f235))
* reorganise compiler in terms of optimisers and transformers ([#88](https://github.com/noir-lang/acvm/issues/88)) ([9329307](https://github.com/noir-lang/acvm/commit/9329307e054de202cfc55207162ad952b70d515e))

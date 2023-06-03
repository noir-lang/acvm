# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.13.2](https://github.com/noir-lang/acvm/compare/root-v0.13.1...root-v0.13.2) (2023-06-02)


### Bug Fixes

* re-use intermediate vars during width reduction ([#278](https://github.com/noir-lang/acvm/issues/278)) ([5b32920](https://github.com/noir-lang/acvm/commit/5b32920263c4481c60faf0b84f0031aa8149b6b2))

## [0.13.1](https://github.com/noir-lang/acvm/compare/root-v0.13.0...root-v0.13.1) (2023-06-01)


### Bug Fixes

* **brillig:** Proper error handling for Brillig failures ([#329](https://github.com/noir-lang/acvm/issues/329)) ([cffa110](https://github.com/noir-lang/acvm/commit/cffa110c8df30ee3dd8b635d38b17b1fcd54b03e))
* **ci:** Add brillig_vm to release-please & link versions ([#332](https://github.com/noir-lang/acvm/issues/332)) ([84bd22e](https://github.com/noir-lang/acvm/commit/84bd22eea46cdfef3a5dbf534b878e819d44f755))
* **ci:** Correct typo to avoid `undefined` in changelogs ([#333](https://github.com/noir-lang/acvm/issues/333)) ([d3424c0](https://github.com/noir-lang/acvm/commit/d3424c04fd303c9cbe25d03118d8b358cbb84b83))

## [0.13.0](https://github.com/noir-lang/acvm/compare/root-v0.12.0...root-v0.13.0) (2023-06-01)


### ⚠ BREAKING CHANGES

* added hash index to pedersen ([#281](https://github.com/noir-lang/acvm/issues/281))
* Add variable length keccak opcode ([#314](https://github.com/noir-lang/acvm/issues/314))
* Remove AES opcode ([#302](https://github.com/noir-lang/acvm/issues/302))
* **acir, acvm:** Remove ComputeMerkleRoot opcode #296
* Remove manual serialization of `Opcode`s in favour of `serde` ([#286](https://github.com/noir-lang/acvm/issues/286))
* Remove backend solvable methods from the interface and solve them in ACVM ([#264](https://github.com/noir-lang/acvm/issues/264))
* Reorganize code related to `PartialWitnessGenerator` ([#287](https://github.com/noir-lang/acvm/issues/287))

### Features

* **acir, acvm:** Remove ComputeMerkleRoot opcode [#296](https://github.com/noir-lang/acvm/issues/296) ([8b3923e](https://github.com/noir-lang/acvm/commit/8b3923e191e4ac399400025496e8bb4453734040))
* Add `Brillig` opcode to introduce custom non-determinism to ACVM ([#152](https://github.com/noir-lang/acvm/issues/152)) ([3c6740a](https://github.com/noir-lang/acvm/commit/3c6740af75125afc8ebb4379f781f8274015e2e2))
* Add variable length keccak opcode ([#314](https://github.com/noir-lang/acvm/issues/314)) ([7bfd169](https://github.com/noir-lang/acvm/commit/7bfd1695b6f119cd70fce4866314c9bb4991eaab))
* added hash index to pedersen ([#281](https://github.com/noir-lang/acvm/issues/281)) ([61820b6](https://github.com/noir-lang/acvm/commit/61820b651900aac8d9557b4b9477ed0e1763c124))
* Remove backend solvable methods from the interface and solve them in ACVM ([#264](https://github.com/noir-lang/acvm/issues/264)) ([69916cb](https://github.com/noir-lang/acvm/commit/69916cbdd928875b2e8fe4775f2251f71c3f3c92))


### Bug Fixes

* Allow async functions without send on async trait ([#292](https://github.com/noir-lang/acvm/issues/292)) ([9f9fc21](https://github.com/noir-lang/acvm/commit/9f9fc216a6d09ca97352ffd365bfd347e94ad8eb))


### Miscellaneous Chores

* Remove AES opcode ([#302](https://github.com/noir-lang/acvm/issues/302)) ([a429a54](https://github.com/noir-lang/acvm/commit/a429a5422d6f001b6db0d0a0f30c79ec0f96de89))
* Remove manual serialization of `Opcode`s in favour of `serde` ([#286](https://github.com/noir-lang/acvm/issues/286)) ([8a3812f](https://github.com/noir-lang/acvm/commit/8a3812fe6ed3b267692284bdcd909d9dd32b9747))
* Reorganize code related to `PartialWitnessGenerator` ([#287](https://github.com/noir-lang/acvm/issues/287)) ([b9d61a1](https://github.com/noir-lang/acvm/commit/b9d61a16210d70e350a7e953951362c94f497f89))

## [0.12.0](https://github.com/noir-lang/acvm/compare/root-v0.11.0...root-v0.12.0) (2023-05-17)


### ⚠ BREAKING CHANGES

* remove deprecated circuit hash functions ([#288](https://github.com/noir-lang/acvm/issues/288))
* allow backends to specify support for all opcode variants ([#273](https://github.com/noir-lang/acvm/issues/273))
* **acvm:** Add CommonReferenceString backend trait ([#231](https://github.com/noir-lang/acvm/issues/231))
* Introduce WitnessMap data structure to avoid leaking internal structure ([#252](https://github.com/noir-lang/acvm/issues/252))
* use struct variants for blackbox function calls ([#269](https://github.com/noir-lang/acvm/issues/269))
* **acvm:** Backend trait must implement Debug ([#275](https://github.com/noir-lang/acvm/issues/275))
* remove `OpcodeResolutionError::UnexpectedOpcode` ([#274](https://github.com/noir-lang/acvm/issues/274))
* **acvm:** rename `hash_to_field128_security` to `hash_to_field_128_security` ([#271](https://github.com/noir-lang/acvm/issues/271))
* **acvm:** update black box solver interfaces to match `pwg:black_box::solve` ([#268](https://github.com/noir-lang/acvm/issues/268))
* **acvm:** expose separate solvers for AND and XOR opcodes ([#266](https://github.com/noir-lang/acvm/issues/266))
* **acvm:** Simplification pass for ACIR ([#151](https://github.com/noir-lang/acvm/issues/151))
* Remove `solve` from PWG trait & introduce separate solvers for each blackbox ([#257](https://github.com/noir-lang/acvm/issues/257))

### Features

* **acvm:** Add CommonReferenceString backend trait ([#231](https://github.com/noir-lang/acvm/issues/231)) ([eeddcf1](https://github.com/noir-lang/acvm/commit/eeddcf179880f246383f7f67a11e589269c4e3ff))
* **acvm:** Simplification pass for ACIR ([#151](https://github.com/noir-lang/acvm/issues/151)) ([7bc42c6](https://github.com/noir-lang/acvm/commit/7bc42c62b6e095f838b781c87cbb1ecd2af5f179))
* **acvm:** update black box solver interfaces to match `pwg:black_box::solve` ([#268](https://github.com/noir-lang/acvm/issues/268)) ([0098b7d](https://github.com/noir-lang/acvm/commit/0098b7d9640076d970e6c15d5fd6f368eb1513ff))
* Introduce WitnessMap data structure to avoid leaking internal structure ([#252](https://github.com/noir-lang/acvm/issues/252)) ([b248e60](https://github.com/noir-lang/acvm/commit/b248e606dd69c25d33ae77c5c5c0541adbf80cd6))
* Remove `solve` from PWG trait & introduce separate solvers for each blackbox ([#257](https://github.com/noir-lang/acvm/issues/257)) ([3f3dd74](https://github.com/noir-lang/acvm/commit/3f3dd7460b27ab06b55dfc3fe5dd733f08e30a9f))
* use struct variants for blackbox function calls ([#269](https://github.com/noir-lang/acvm/issues/269)) ([a83333b](https://github.com/noir-lang/acvm/commit/a83333b9e270dfcfd40a36271896840ec0201bc4))


### Bug Fixes

* **acir:** Hide variants of WitnessMapError and export it from package ([#283](https://github.com/noir-lang/acvm/issues/283)) ([bbd9ab7](https://github.com/noir-lang/acvm/commit/bbd9ab7ca5be3fb31f3e141fee2522704852f5de))


### Miscellaneous Chores

* **acvm:** Backend trait must implement Debug ([#275](https://github.com/noir-lang/acvm/issues/275)) ([3288b4c](https://github.com/noir-lang/acvm/commit/3288b4c7eb01f5621e577d5ff9e7c92c7757e021))
* **acvm:** expose separate solvers for AND and XOR opcodes ([#266](https://github.com/noir-lang/acvm/issues/266)) ([84b5d18](https://github.com/noir-lang/acvm/commit/84b5d18d29a111a42bfc1c3d122129c8f062c3db))
* **acvm:** rename `hash_to_field128_security` to `hash_to_field_128_security` ([#271](https://github.com/noir-lang/acvm/issues/271)) ([fad9af2](https://github.com/noir-lang/acvm/commit/fad9af27fb102fa34bf7511f8ed7b16b3ec2d115))
* allow backends to specify support for all opcode variants ([#273](https://github.com/noir-lang/acvm/issues/273)) ([efd37fe](https://github.com/noir-lang/acvm/commit/efd37fedcbbabb3fac810e662731439e07fef49a))
* remove `OpcodeResolutionError::UnexpectedOpcode` ([#274](https://github.com/noir-lang/acvm/issues/274)) ([0e71aac](https://github.com/noir-lang/acvm/commit/0e71aac7aa85b3e9142972a26ba122c2c7c51d9b))
* remove deprecated circuit hash functions ([#288](https://github.com/noir-lang/acvm/issues/288)) ([1a22c75](https://github.com/noir-lang/acvm/commit/1a22c752de3354a2a6d34892331ab6623b24c0b0))

## [0.11.0](https://github.com/noir-lang/acvm/compare/root-v0.10.3...root-v0.11.0) (2023-05-04)


### ⚠ BREAKING CHANGES

* **acvm:** Introduce Error type for fallible Backend traits ([#248](https://github.com/noir-lang/acvm/issues/248))

### Features

* **acvm:** Add generic error for failing to solve an opcode ([#251](https://github.com/noir-lang/acvm/issues/251)) ([bc89528](https://github.com/noir-lang/acvm/commit/bc8952820de610e585d505decfac6e590bbb1a35))
* **acvm:** Introduce Error type for fallible Backend traits ([#248](https://github.com/noir-lang/acvm/issues/248)) ([45c45f7](https://github.com/noir-lang/acvm/commit/45c45f7cdb79c3ccb0373ca0e698b282d4dabc39))
* Add Keccak Hash function ([#259](https://github.com/noir-lang/acvm/issues/259)) ([443c734](https://github.com/noir-lang/acvm/commit/443c73482eeef6cc42a1a254bf0d7706698ee353))


### Bug Fixes

* **acir:** Fix `Expression` multiplication to correctly handle degree 1 terms ([#255](https://github.com/noir-lang/acvm/issues/255)) ([e399396](https://github.com/noir-lang/acvm/commit/e399396f7e06deb6b831517af17018607df3f252))

## [0.10.3](https://github.com/noir-lang/acvm/compare/root-v0.10.2...root-v0.10.3) (2023-04-28)


### Bug Fixes

* add default feature flag to ACVM crate ([#245](https://github.com/noir-lang/acvm/issues/245)) ([455fddb](https://github.com/noir-lang/acvm/commit/455fddbc19af81cb01d54e29cad199691e1a1d98))

## [0.10.2](https://github.com/noir-lang/acvm/compare/root-v0.10.1...root-v0.10.2) (2023-04-28)


### Bug Fixes

* add default flag to `acvm_stdlib` ([#242](https://github.com/noir-lang/acvm/issues/242)) ([83b6fa8](https://github.com/noir-lang/acvm/commit/83b6fa8302569add7e3ac8481b2fd2a6a1ff3576))

## [0.10.1](https://github.com/noir-lang/acvm/compare/root-v0.10.0...root-v0.10.1) (2023-04-28)


### Bug Fixes

* **acir:** add `bn254` as default feature flag ([#240](https://github.com/noir-lang/acvm/issues/240)) ([e56973d](https://github.com/noir-lang/acvm/commit/e56973d8dc1745fe9bb844ec8347acd4d836d42f))

## [0.10.0](https://github.com/noir-lang/acvm/compare/root-v0.9.0...root-v0.10.0) (2023-04-26)


### ⚠ BREAKING CHANGES

* return `Result<OpcodeResolution, OpcodeResolutionError>` from `solve_range_opcode` ([#238](https://github.com/noir-lang/acvm/issues/238))
* **acvm:** have all black box functions return `Result<OpcodeResolution, OpcodeResolutionError>` ([#237](https://github.com/noir-lang/acvm/issues/237))
* **acvm:** implement `hash_to_field_128_security` ([#230](https://github.com/noir-lang/acvm/issues/230))
* replace `MerkleMembership` opcode with `ComputeMerkleRoot` ([#233](https://github.com/noir-lang/acvm/issues/233))
* require `Backend` to implement `Default` trait ([#223](https://github.com/noir-lang/acvm/issues/223))
* Make GeneralOptimizer crate visible ([#220](https://github.com/noir-lang/acvm/issues/220))
* return `PartialWitnessGeneratorStatus` from `PartialWitnessGenerator.solve` ([#213](https://github.com/noir-lang/acvm/issues/213))
* organise operator implementations for Expression ([#190](https://github.com/noir-lang/acvm/issues/190))

### Features

* **acvm:** have all black box functions return `Result&lt;OpcodeResolution, OpcodeResolutionError&gt;` ([#237](https://github.com/noir-lang/acvm/issues/237)) ([e8e93fd](https://github.com/noir-lang/acvm/commit/e8e93fda0db18f0d266dd1aacbb53ec787992dc9))
* **acvm:** implement `hash_to_field_128_security` ([#230](https://github.com/noir-lang/acvm/issues/230)) ([198fb69](https://github.com/noir-lang/acvm/commit/198fb69e90a5ed3c0a8716d888b4dc6c2f9b18aa))
* Add range opcode optimization ([#219](https://github.com/noir-lang/acvm/issues/219)) ([7abe6e5](https://github.com/noir-lang/acvm/commit/7abe6e5f6d6fea379c3748a910afd00db066eb45))
* implement `add_mul` on `Expression` ([#207](https://github.com/noir-lang/acvm/issues/207)) ([f156e18](https://github.com/noir-lang/acvm/commit/f156e18cf7a0f1a99bbe1683b8e75fec8325e6dd))
* implement `FieldElement::from&lt;bool&gt;()` ([#203](https://github.com/noir-lang/acvm/issues/203)) ([476cfa2](https://github.com/noir-lang/acvm/commit/476cfa247fddb515c64c2801c6868357c9375294))
* replace `MerkleMembership` opcode with `ComputeMerkleRoot` ([#233](https://github.com/noir-lang/acvm/issues/233)) ([74bfee8](https://github.com/noir-lang/acvm/commit/74bfee80e0ff0d205aee1eea548c97ade8bd0e41))
* require `Backend` to implement `Default` trait ([#223](https://github.com/noir-lang/acvm/issues/223)) ([00282dc](https://github.com/noir-lang/acvm/commit/00282dc5e2b03947bf709a088d829f3e0ba80eed))
* return `PartialWitnessGeneratorStatus` from `PartialWitnessGenerator.solve` ([#213](https://github.com/noir-lang/acvm/issues/213)) ([e877bed](https://github.com/noir-lang/acvm/commit/e877bed2cca76bd486e9bed66b4230e65a01f0a2))
* return `Result&lt;OpcodeResolution, OpcodeResolutionError&gt;` from `solve_range_opcode` ([#238](https://github.com/noir-lang/acvm/issues/238)) ([15d3c5a](https://github.com/noir-lang/acvm/commit/15d3c5a9be2dd92f266fcb7e672da17cada9fec5))


### Bug Fixes

* prevent `bn254` feature flag always being enabled ([#225](https://github.com/noir-lang/acvm/issues/225)) ([82eee6a](https://github.com/noir-lang/acvm/commit/82eee6ab08ae480f04904ca8571fd88f4466c000))


### Miscellaneous Chores

* Make GeneralOptimizer crate visible ([#220](https://github.com/noir-lang/acvm/issues/220)) ([64bb346](https://github.com/noir-lang/acvm/commit/64bb346524428a0ce196826ea1e5ccde08ad6201))
* organise operator implementations for Expression ([#190](https://github.com/noir-lang/acvm/issues/190)) ([a619df6](https://github.com/noir-lang/acvm/commit/a619df614bbb9b2518b788b42a7553b069823a0f))

## [0.9.0](https://github.com/noir-lang/acvm/compare/root-v0.8.1...root-v0.9.0) (2023-04-07)


### ⚠ BREAKING CHANGES

* **acvm:** Remove deprecated eth_contract_from_cs from SmartContract trait ([#185](https://github.com/noir-lang/acvm/issues/185))
* **acvm:** make `Backend` trait object safe ([#180](https://github.com/noir-lang/acvm/issues/180))

### Features

* **acvm:** make `Backend` trait object safe ([#180](https://github.com/noir-lang/acvm/issues/180)) ([fd28657](https://github.com/noir-lang/acvm/commit/fd28657426260ce3c53517b75a27eb5c4a74e234))


### Bug Fixes

* Add test for Out of Memory  ([#188](https://github.com/noir-lang/acvm/issues/188)) ([c3db985](https://github.com/noir-lang/acvm/commit/c3db985893e7e59ea04005bb3a57eda5c6ce28c7))


### Miscellaneous Chores

* **acvm:** Remove deprecated eth_contract_from_cs from SmartContract trait ([#185](https://github.com/noir-lang/acvm/issues/185)) ([ee59c9e](https://github.com/noir-lang/acvm/commit/ee59c9efe9a54ff6b97e4daaebf64f3e327e97d9))

## [0.8.1](https://github.com/noir-lang/acvm/compare/root-v0.8.0...root-v0.8.1) (2023-03-30)


### Bug Fixes

* unwraps if inputs is zero ([#171](https://github.com/noir-lang/acvm/issues/171)) ([10a3bb2](https://github.com/noir-lang/acvm/commit/10a3bb2a9930ccf422b3f08227aae07775686860))

## [0.8.0](https://github.com/noir-lang/acvm/compare/root-v0.7.1...root-v0.8.0) (2023-03-28)


### ⚠ BREAKING CHANGES

* **acir:** Read Log Directive ([#156](https://github.com/noir-lang/acvm/issues/156))

### Bug Fixes

* **acir:** Read Log Directive ([#156](https://github.com/noir-lang/acvm/issues/156)) ([1cc2b7f](https://github.com/noir-lang/acvm/commit/1cc2b7f2179cecc338fe0def72bb2dd17eaed0cd))

## [0.7.1](https://github.com/noir-lang/acvm/compare/root-v0.7.0...root-v0.7.1) (2023-03-27)


### Bug Fixes

* **pwg:** stall instead of fail for unassigned black box ([#154](https://github.com/noir-lang/acvm/issues/154)) ([412a1a6](https://github.com/noir-lang/acvm/commit/412a1a60b434bef53e12d37c3b2bb3d51a317994))

## [0.7.0](https://github.com/noir-lang/acvm/compare/root-v0.6.0...root-v0.7.0) (2023-03-23)


### ⚠ BREAKING CHANGES

* Add initial oracle opcode ([#149](https://github.com/noir-lang/acvm/issues/149))
* **acir:** Add RAM and ROM opcodes
* **acir:** Add a public outputs field ([#56](https://github.com/noir-lang/acvm/issues/56))
* **acir:** remove `Linear` struct ([#145](https://github.com/noir-lang/acvm/issues/145))
* **acvm:** remove `prove_with_meta` and `verify_from_cs` from `ProofSystemCompiler` ([#140](https://github.com/noir-lang/acvm/issues/140))
* **acvm:** Remove truncate and oddrange directives ([#142](https://github.com/noir-lang/acvm/issues/142))

### Features

* **acir:** Add a public outputs field ([#56](https://github.com/noir-lang/acvm/issues/56)) ([5f358a9](https://github.com/noir-lang/acvm/commit/5f358a97aaa81d87956e182cd8a6d60de75f9752))
* **acir:** Add RAM and ROM opcodes ([73e9f25](https://github.com/noir-lang/acvm/commit/73e9f25dd87b2ca91245e93d2445eadc0f522fac))
* Add initial oracle opcode ([#149](https://github.com/noir-lang/acvm/issues/149)) ([88ee2f8](https://github.com/noir-lang/acvm/commit/88ee2f89f37abf5dd1d9f91b4d2eed44dc651348))


### Miscellaneous Chores

* **acir:** remove `Linear` struct ([#145](https://github.com/noir-lang/acvm/issues/145)) ([bbb6d92](https://github.com/noir-lang/acvm/commit/bbb6d92e25c43dd33b12f5fcd639fc9ad2a9c9d8))
* **acvm:** remove `prove_with_meta` and `verify_from_cs` from `ProofSystemCompiler` ([#140](https://github.com/noir-lang/acvm/issues/140)) ([35dd181](https://github.com/noir-lang/acvm/commit/35dd181102203df17eef510666b327ef41f4b036))
* **acvm:** Remove truncate and oddrange directives ([#142](https://github.com/noir-lang/acvm/issues/142)) ([85dd6e8](https://github.com/noir-lang/acvm/commit/85dd6e85bfba85bfb97651f7e30e1f75deb986d5))

## [0.6.0](https://github.com/noir-lang/acvm/compare/root-v0.5.0...root-v0.6.0) (2023-03-03)


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
* **ci:** publish acvm_stdlib before acvm ([#117](https://github.com/noir-lang/acvm/issues/117)) ([ca6defc](https://github.com/noir-lang/acvm/commit/ca6defc9bb5f51241b2fc4d9cd732f9678b4688f))


### Miscellaneous Chores

* **acir:** remove `UnknownWitness` ([#123](https://github.com/noir-lang/acvm/issues/123)) ([9f002c7](https://github.com/noir-lang/acvm/commit/9f002c7b49a5cf222d4a01732cc4917a47690863))
* **acir:** rename `term_addition` to `push_addition_term` ([d389385](https://github.com/noir-lang/acvm/commit/d38938542851a97dc01727438391e6a65e44c689))
* **acir:** rename `term_multiplication` to `push_multiplication_term` ([#122](https://github.com/noir-lang/acvm/issues/122)) ([d389385](https://github.com/noir-lang/acvm/commit/d38938542851a97dc01727438391e6a65e44c689))

## [0.5.0](https://github.com/noir-lang/acvm/compare/root-v0.4.1...root-v0.5.0) (2023-02-22)


### ⚠ BREAKING CHANGES

* **acvm:** switch to accepting public inputs as a map ([#96](https://github.com/noir-lang/acvm/issues/96))
* **acvm:** add `eth_contract_from_vk` to `SmartContract
* update `ProofSystemCompiler` to not take ownership of keys ([#111](https://github.com/noir-lang/acvm/issues/111))
* update `ProofSystemCompiler` methods to take `&Circuit` ([#108](https://github.com/noir-lang/acvm/issues/108))
* **acir:** make PublicInputs use a BTreeSet rather than Vec ([#99](https://github.com/noir-lang/acvm/issues/99))
* refactor ToRadix to ToRadixLe and ToRadixBe ([#58](https://github.com/noir-lang/acvm/issues/58))
* **acir:** Add keccak256 Opcode ([#91](https://github.com/noir-lang/acvm/issues/91))
* reorganise compiler in terms of optimisers and transformers ([#88](https://github.com/noir-lang/acvm/issues/88))

### Features

* **acir:** Add keccak256 Opcode ([#91](https://github.com/noir-lang/acvm/issues/91)) ([b909146](https://github.com/noir-lang/acvm/commit/b9091461e199bacdd073cc9b31f03dade0b4fb2d))
* **acir:** make PublicInputs use a BTreeSet rather than Vec ([#99](https://github.com/noir-lang/acvm/issues/99)) ([53666b7](https://github.com/noir-lang/acvm/commit/53666b782d89c65cd755f9e4ded2c9cf5a141e46))
* **acvm:** add `eth_contract_from_vk` to `SmartContract ([#113](https://github.com/noir-lang/acvm/issues/113)) ([373c18f](https://github.com/noir-lang/acvm/commit/373c18fc05edf673cfec9e8bbb78bd7d7514999e))
* **acvm:** switch to accepting public inputs as a map ([#96](https://github.com/noir-lang/acvm/issues/96)) ([f57ba57](https://github.com/noir-lang/acvm/commit/f57ba57c2bb2597edf2b02fb1321c69cf11993ee))
* **ci:** Add release workflow ([#89](https://github.com/noir-lang/acvm/issues/89)) ([db8e828](https://github.com/noir-lang/acvm/commit/db8e828341f59241ef7f437c908277fb8fbca9e3))
* **ci:** Publish crates upon release ([#104](https://github.com/noir-lang/acvm/issues/104)) ([b265920](https://github.com/noir-lang/acvm/commit/b265920bc1b0c776d20326a0b74fc635c22af4b9))
* update `ProofSystemCompiler` methods to take `&Circuit` ([#108](https://github.com/noir-lang/acvm/issues/108)) ([af56ca9](https://github.com/noir-lang/acvm/commit/af56ca9da06068c650c66e76bfd09e65eb0ec213))
* update `ProofSystemCompiler` to not take ownership of keys ([#111](https://github.com/noir-lang/acvm/issues/111)) ([39b8a41](https://github.com/noir-lang/acvm/commit/39b8a41293e567971f700f61103852cb987a8d16))
* Update Arkworks' dependencies on `acir_field` ([#69](https://github.com/noir-lang/acvm/issues/69)) ([65d6130](https://github.com/noir-lang/acvm/commit/65d61307a12f25e04afad2d50e4c4db5ce97dd8c))


### Bug Fixes

* **ci:** Update dependency versions in the workspace file ([#103](https://github.com/noir-lang/acvm/issues/103)) ([9acc266](https://github.com/noir-lang/acvm/commit/9acc266c7dc5a6ad2fa9c466cc82cb81d984b7ed))
* Clean up Log Directive hex output  ([#97](https://github.com/noir-lang/acvm/issues/97)) ([d23c735](https://github.com/noir-lang/acvm/commit/d23c7352523ffb42f3e8f4229b61f9803ab78a7e))


### Miscellaneous Chores

* refactor ToRadix to ToRadixLe and ToRadixBe ([#58](https://github.com/noir-lang/acvm/issues/58)) ([2427a27](https://github.com/noir-lang/acvm/commit/2427a275048e598c6d651cce8348a4c55148f235))
* reorganise compiler in terms of optimisers and transformers ([#88](https://github.com/noir-lang/acvm/issues/88)) ([9329307](https://github.com/noir-lang/acvm/commit/9329307e054de202cfc55207162ad952b70d515e))

## [0.4.1] - 2023-02-08

### Added

### Fixed

- Removed duplicated logic in match branch

### Changed

### Removed

## [0.4.0] - 2023-02-08

### Added

- Add log directive
- Expose `acir_field` through `acir` crate
- Add permutation directive
- Add preprocess methods to ACVM interface

### Fixed

### Changed

- Changed spellings of many functions to be correct using spellchecker

### Removed

## [0.3.1] - 2023-01-18

### Added

### Fixed

### Changed

- ACVM compile method now returns an Error for when a function cannot be reduced to arithmetic gates

- Backtrack changes from noir-lang/noir/587

### Removed

## [0.3.0] - 2022-12-31

### Added

- Added stdlib module to hold all of the standard opcodes
- added `read` , `write` methods for circuit

### Fixed

### Changed

- XOR, Range and AND gates are no longer special case. They are now another opcode in the GadgetCall
- Move fallback module to `stdlib`
- Optimizer code and any other passes will live in acvm. acir is solely for defining the IR now.
- ACIR passes now live under the compiler parent module
- Moved opcode module in acir crate to circuit/opcode
- Rename GadgetCall to BlackBoxFuncCall
- Rename opcode file to blackbox_functions . Similarly OPCODE is now BlackBoxFunc
- Renamed GateResolution::UnsupportedOpcode to GateResolution::UnsupportedBlackBoxFunc
- Renamed GadgetDefinition to FuncDefinition
- Rename GadgetInput to FunctionInput
- Rename Gate -> Opcode . Similarly gate.rs is now opcodes.rs
- Rename CustomGate::supports_gate -> CustomGate::supports_opcode
- Rename GateResolution to OpcodeResolution
- Rename Split directive to ToBits
- Field element printing function was modified to uses ascii superscript numbers and ascii multiplication
- Refactor the way we print ACIR (This is a first draft and will change with more feedback)
- Rename `solve_gadget_call` trait method on ProofSystemCompile to `solve_blackbox_function_call`
- API for `compile` now requires a function pointer which tells us whether a blackbox function is supported
- Renamed Directive::Oddrange to Directive::OddRange
- Renamed FieldElement::to_bytes to FieldElement::to_be_bytes

### Removed

- Selector struct has been removed as it is no longer being used. It is also not being used by Noir.
- CustomGate trait -- There is a method in the ProofSystemCompiler Trait that backends can use to indicate whether
they support a particular black box function
- Remove OpcodeResolution enum from pwg. The happy case is strictly when the witness has been solved

## [0.2.1] - 2022-12-23

- Removed ToBits and ToBytes opcode

# Changelog

## [0.11.0](https://github.com/noir-lang/acvm/compare/acvm-v0.10.3...acvm-v0.11.0) (2023-05-04)


### ⚠ BREAKING CHANGES

* **acvm:** Introduce Error type for fallible Backend traits ([#248](https://github.com/noir-lang/acvm/issues/248))

### Features

* **acvm:** Add generic error for failing to solve an opcode ([#251](https://github.com/noir-lang/acvm/issues/251)) ([bc89528](https://github.com/noir-lang/acvm/commit/bc8952820de610e585d505decfac6e590bbb1a35))
* **acvm:** Introduce Error type for fallible Backend traits ([#248](https://github.com/noir-lang/acvm/issues/248)) ([45c45f7](https://github.com/noir-lang/acvm/commit/45c45f7cdb79c3ccb0373ca0e698b282d4dabc39))
* Add Keccak Hash function ([#259](https://github.com/noir-lang/acvm/issues/259)) ([443c734](https://github.com/noir-lang/acvm/commit/443c73482eeef6cc42a1a254bf0d7706698ee353))

## [0.10.3](https://github.com/noir-lang/acvm/compare/acvm-v0.10.2...acvm-v0.10.3) (2023-04-28)


### Bug Fixes

* add default feature flag to ACVM crate ([#245](https://github.com/noir-lang/acvm/issues/245)) ([455fddb](https://github.com/noir-lang/acvm/commit/455fddbc19af81cb01d54e29cad199691e1a1d98))

## [0.10.2](https://github.com/noir-lang/acvm/compare/acvm-v0.10.1...acvm-v0.10.2) (2023-04-28)


### Miscellaneous Chores

* **acvm:** Synchronize undefined versions

## [0.10.1](https://github.com/noir-lang/acvm/compare/acvm-v0.10.0...acvm-v0.10.1) (2023-04-28)


### Miscellaneous Chores

* **acvm:** Synchronize undefined versions

## [0.10.0](https://github.com/noir-lang/acvm/compare/acvm-v0.9.0...acvm-v0.10.0) (2023-04-26)


### ⚠ BREAKING CHANGES

* return `Result<OpcodeResolution, OpcodeResolutionError>` from `solve_range_opcode` ([#238](https://github.com/noir-lang/acvm/issues/238))
* **acvm:** have all black box functions return `Result<OpcodeResolution, OpcodeResolutionError>` ([#237](https://github.com/noir-lang/acvm/issues/237))
* **acvm:** implement `hash_to_field_128_security` ([#230](https://github.com/noir-lang/acvm/issues/230))
* require `Backend` to implement `Default` trait ([#223](https://github.com/noir-lang/acvm/issues/223))
* Make GeneralOptimizer crate visible ([#220](https://github.com/noir-lang/acvm/issues/220))
* return `PartialWitnessGeneratorStatus` from `PartialWitnessGenerator.solve` ([#213](https://github.com/noir-lang/acvm/issues/213))
* organise operator implementations for Expression ([#190](https://github.com/noir-lang/acvm/issues/190))

### Features

* **acvm:** have all black box functions return `Result&lt;OpcodeResolution, OpcodeResolutionError&gt;` ([#237](https://github.com/noir-lang/acvm/issues/237)) ([e8e93fd](https://github.com/noir-lang/acvm/commit/e8e93fda0db18f0d266dd1aacbb53ec787992dc9))
* **acvm:** implement `hash_to_field_128_security` ([#230](https://github.com/noir-lang/acvm/issues/230)) ([198fb69](https://github.com/noir-lang/acvm/commit/198fb69e90a5ed3c0a8716d888b4dc6c2f9b18aa))
* Add range opcode optimization ([#219](https://github.com/noir-lang/acvm/issues/219)) ([7abe6e5](https://github.com/noir-lang/acvm/commit/7abe6e5f6d6fea379c3748a910afd00db066eb45))
* require `Backend` to implement `Default` trait ([#223](https://github.com/noir-lang/acvm/issues/223)) ([00282dc](https://github.com/noir-lang/acvm/commit/00282dc5e2b03947bf709a088d829f3e0ba80eed))
* return `PartialWitnessGeneratorStatus` from `PartialWitnessGenerator.solve` ([#213](https://github.com/noir-lang/acvm/issues/213)) ([e877bed](https://github.com/noir-lang/acvm/commit/e877bed2cca76bd486e9bed66b4230e65a01f0a2))
* return `Result&lt;OpcodeResolution, OpcodeResolutionError&gt;` from `solve_range_opcode` ([#238](https://github.com/noir-lang/acvm/issues/238)) ([15d3c5a](https://github.com/noir-lang/acvm/commit/15d3c5a9be2dd92f266fcb7e672da17cada9fec5))


### Bug Fixes

* prevent `bn254` feature flag always being enabled ([#225](https://github.com/noir-lang/acvm/issues/225)) ([82eee6a](https://github.com/noir-lang/acvm/commit/82eee6ab08ae480f04904ca8571fd88f4466c000))


### Miscellaneous Chores

* Make GeneralOptimizer crate visible ([#220](https://github.com/noir-lang/acvm/issues/220)) ([64bb346](https://github.com/noir-lang/acvm/commit/64bb346524428a0ce196826ea1e5ccde08ad6201))
* organise operator implementations for Expression ([#190](https://github.com/noir-lang/acvm/issues/190)) ([a619df6](https://github.com/noir-lang/acvm/commit/a619df614bbb9b2518b788b42a7553b069823a0f))

## [0.9.0](https://github.com/noir-lang/acvm/compare/acvm-v0.8.1...acvm-v0.9.0) (2023-04-07)


### ⚠ BREAKING CHANGES

* **acvm:** Remove deprecated eth_contract_from_cs from SmartContract trait ([#185](https://github.com/noir-lang/acvm/issues/185))
* **acvm:** make `Backend` trait object safe ([#180](https://github.com/noir-lang/acvm/issues/180))

### Features

* **acvm:** make `Backend` trait object safe ([#180](https://github.com/noir-lang/acvm/issues/180)) ([fd28657](https://github.com/noir-lang/acvm/commit/fd28657426260ce3c53517b75a27eb5c4a74e234))


### Miscellaneous Chores

* **acvm:** Remove deprecated eth_contract_from_cs from SmartContract trait ([#185](https://github.com/noir-lang/acvm/issues/185)) ([ee59c9e](https://github.com/noir-lang/acvm/commit/ee59c9efe9a54ff6b97e4daaebf64f3e327e97d9))

## [0.8.1](https://github.com/noir-lang/acvm/compare/acvm-v0.8.0...acvm-v0.8.1) (2023-03-30)


### Miscellaneous Chores

* **acvm:** Synchronize undefined versions

## [0.8.0](https://github.com/noir-lang/acvm/compare/acvm-v0.7.1...acvm-v0.8.0) (2023-03-28)


### Miscellaneous Chores

* **acvm:** Synchronize undefined versions

## [0.7.1](https://github.com/noir-lang/acvm/compare/acvm-v0.7.0...acvm-v0.7.1) (2023-03-27)


### Bug Fixes

* **pwg:** stall instead of fail for unassigned black box ([#154](https://github.com/noir-lang/acvm/issues/154)) ([412a1a6](https://github.com/noir-lang/acvm/commit/412a1a60b434bef53e12d37c3b2bb3d51a317994))

## [0.7.0](https://github.com/noir-lang/acvm/compare/acvm-v0.6.0...acvm-v0.7.0) (2023-03-23)


### ⚠ BREAKING CHANGES

* Add initial oracle opcode ([#149](https://github.com/noir-lang/acvm/issues/149))
* **acir:** Add RAM and ROM opcodes
* **acir:** Add a public outputs field ([#56](https://github.com/noir-lang/acvm/issues/56))
* **acvm:** remove `prove_with_meta` and `verify_from_cs` from `ProofSystemCompiler` ([#140](https://github.com/noir-lang/acvm/issues/140))
* **acvm:** Remove truncate and oddrange directives ([#142](https://github.com/noir-lang/acvm/issues/142))

### Features

* **acir:** Add a public outputs field ([#56](https://github.com/noir-lang/acvm/issues/56)) ([5f358a9](https://github.com/noir-lang/acvm/commit/5f358a97aaa81d87956e182cd8a6d60de75f9752))
* **acir:** Add RAM and ROM opcodes ([73e9f25](https://github.com/noir-lang/acvm/commit/73e9f25dd87b2ca91245e93d2445eadc0f522fac))
* Add initial oracle opcode ([#149](https://github.com/noir-lang/acvm/issues/149)) ([88ee2f8](https://github.com/noir-lang/acvm/commit/88ee2f89f37abf5dd1d9f91b4d2eed44dc651348))


### Miscellaneous Chores

* **acvm:** remove `prove_with_meta` and `verify_from_cs` from `ProofSystemCompiler` ([#140](https://github.com/noir-lang/acvm/issues/140)) ([35dd181](https://github.com/noir-lang/acvm/commit/35dd181102203df17eef510666b327ef41f4b036))
* **acvm:** Remove truncate and oddrange directives ([#142](https://github.com/noir-lang/acvm/issues/142)) ([85dd6e8](https://github.com/noir-lang/acvm/commit/85dd6e85bfba85bfb97651f7e30e1f75deb986d5))

## [0.6.0](https://github.com/noir-lang/acvm/compare/acvm-v0.5.0...acvm-v0.6.0) (2023-03-03)


### ⚠ BREAKING CHANGES

* add block opcode ([#114](https://github.com/noir-lang/acvm/issues/114))

### Features

* add block opcode ([#114](https://github.com/noir-lang/acvm/issues/114)) ([097cfb0](https://github.com/noir-lang/acvm/commit/097cfb069291705ddb4bf1fca77ddcef21dbbd08))

## [0.5.0](https://github.com/noir-lang/acvm/compare/acvm-v0.4.1...acvm-v0.5.0) (2023-02-22)


### ⚠ BREAKING CHANGES

* **acvm:** switch to accepting public inputs as a map ([#96](https://github.com/noir-lang/acvm/issues/96))
* **acvm:** add `eth_contract_from_vk` to `SmartContract
* update `ProofSystemCompiler` to not take ownership of keys ([#111](https://github.com/noir-lang/acvm/issues/111))
* update `ProofSystemCompiler` methods to take `&Circuit` ([#108](https://github.com/noir-lang/acvm/issues/108))
* refactor ToRadix to ToRadixLe and ToRadixBe ([#58](https://github.com/noir-lang/acvm/issues/58))
* reorganise compiler in terms of optimisers and transformers ([#88](https://github.com/noir-lang/acvm/issues/88))

### Features

* **acvm:** add `eth_contract_from_vk` to `SmartContract ([#113](https://github.com/noir-lang/acvm/issues/113)) ([373c18f](https://github.com/noir-lang/acvm/commit/373c18fc05edf673cfec9e8bbb78bd7d7514999e))
* **acvm:** switch to accepting public inputs as a map ([#96](https://github.com/noir-lang/acvm/issues/96)) ([f57ba57](https://github.com/noir-lang/acvm/commit/f57ba57c2bb2597edf2b02fb1321c69cf11993ee))
* update `ProofSystemCompiler` methods to take `&Circuit` ([#108](https://github.com/noir-lang/acvm/issues/108)) ([af56ca9](https://github.com/noir-lang/acvm/commit/af56ca9da06068c650c66e76bfd09e65eb0ec213))
* update `ProofSystemCompiler` to not take ownership of keys ([#111](https://github.com/noir-lang/acvm/issues/111)) ([39b8a41](https://github.com/noir-lang/acvm/commit/39b8a41293e567971f700f61103852cb987a8d16))


### Bug Fixes

* Clean up Log Directive hex output  ([#97](https://github.com/noir-lang/acvm/issues/97)) ([d23c735](https://github.com/noir-lang/acvm/commit/d23c7352523ffb42f3e8f4229b61f9803ab78a7e))


### Miscellaneous Chores

* refactor ToRadix to ToRadixLe and ToRadixBe ([#58](https://github.com/noir-lang/acvm/issues/58)) ([2427a27](https://github.com/noir-lang/acvm/commit/2427a275048e598c6d651cce8348a4c55148f235))
* reorganise compiler in terms of optimisers and transformers ([#88](https://github.com/noir-lang/acvm/issues/88)) ([9329307](https://github.com/noir-lang/acvm/commit/9329307e054de202cfc55207162ad952b70d515e))

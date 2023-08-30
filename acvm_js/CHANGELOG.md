# Changelog

## [0.23.0](https://github.com/noir-lang/acvm/compare/acvm_js-v0.22.0...acvm_js-v0.23.0) (2023-08-30)


### ⚠ BREAKING CHANGES

* **acvm:** Remove `BlackBoxFunctionSolver` from `Backend` trait ([#494](https://github.com/noir-lang/acvm/issues/494))
* **acvm:** Pass `BlackBoxFunctionSolver` to `ACVM` by reference

### Features

* **acvm_js:** Add `execute_circuit_with_black_box_solver` to prevent reinitialization of `BlackBoxFunctionSolver` ([3877e0e](https://github.com/noir-lang/acvm/commit/3877e0e438a8d0e5545a4da7210767dec05c342f))
* Expose a `BlackBoxFunctionSolver` containing a barretenberg wasm from `blackbox_solver` ([#494](https://github.com/noir-lang/acvm/issues/494)) ([a1d4b71](https://github.com/noir-lang/acvm/commit/a1d4b71256dfbf1e883e770dd9c45479235aa860))


### Miscellaneous Chores

* **acvm:** Pass `BlackBoxFunctionSolver` to `ACVM` by reference ([3877e0e](https://github.com/noir-lang/acvm/commit/3877e0e438a8d0e5545a4da7210767dec05c342f))
* **acvm:** Remove `BlackBoxFunctionSolver` from `Backend` trait ([#494](https://github.com/noir-lang/acvm/issues/494)) ([a1d4b71](https://github.com/noir-lang/acvm/commit/a1d4b71256dfbf1e883e770dd9c45479235aa860))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * acvm bumped from 0.22.0 to 0.23.0

## [0.22.0](https://github.com/noir-lang/acvm/compare/acvm_js-v0.21.0...acvm_js-v0.22.0) (2023-08-18)


### Bug Fixes

* Empty commit to trigger release-please ([e8f0748](https://github.com/noir-lang/acvm/commit/e8f0748042ef505d59ab63266d3c36c5358ee30d))


### Dependencies

* The following workspace dependencies were updated
  * dependencies
    * acvm bumped from 0.21.0 to 0.22.0

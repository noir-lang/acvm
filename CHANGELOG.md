# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added stdlib module to hold all of the standard opcodes

### Fixed

### Changed

- XOR, Range and AND gates are no longer special case. They are now another opcode in the GadgetCall
- Move fallback module to `stdlib`
- optimiser code and any other passes will live in acvm. acir is solely for defining the IR now.
- ACIR passes now live under the compiler parent moduel
### Removed

## [0.2.1] - 2022-12-23

- Removed ToBits and ToBytes opcode

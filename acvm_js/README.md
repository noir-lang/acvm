# acvm-js

This project serves to wrap each build variant of ACVM into a node module that can be used for invoking utilities for the purposes of proving and verifying — whether that be in the browser or node.js.

Currently the ACVM rust project selects its field element representation at compile time, hence why multiple builds are necessary. Running `build_all.sh` will assemble the following packages into the `./build/` directory:

- `@noir-lang/acvm-bn254`
- `@noir-lang/acvm-bls12_381`

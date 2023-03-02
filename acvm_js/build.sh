#! /bin/bash

npm run wrap:component
rm -rf dist
npx rollup -c --bundleConfigAsCjs
cp src/acvm_wasm/generated/*.wasm dist/

#!/usr/bin/env bash

mkdir -p $out
cp README.md $out/
cp -r ./pkg/* $out/

# The main package.json contains several keys which are incorrect/unwanted when distributing.
cat package.json \
| jq 'del(.private, .devDependencies, .scripts, .packageManager)' \
> $out/package.json

# Cleanup temporary pkg directory
rm -r ./pkg
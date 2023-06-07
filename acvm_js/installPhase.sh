#!/usr/bin/env bash

if [ -v out ]; then
  echo "Will install package to $out (defined outside installPhase.sh script)"
else
  out="./result"
  echo "Will install package to $out"
fi

mkdir -p $out
cp README.md $out/
cp -r ./pkg/* $out/

# The main package.json contains several keys which are incorrect/unwanted when distributing.
cat package.json \
| jq 'del(.private, .devDependencies, .scripts, .packageManager)' \
> $out/package.json

# Cleanup temporary pkg directory
rm -r ./pkg
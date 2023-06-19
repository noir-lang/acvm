#!/usr/bin/env bash

rm -rf ./outputs >/dev/null 2>&1
rm -rf ./result >/dev/null 2>&1

if [ -v out ]; then
  echo "Will install package to $out (defined outside installPhase.sh script)"
else
  out="./outputs/out"
  echo "Will install package to $out"
fi

./buildPhaseCargoCommand.sh
./installPhase.sh

ln -s $out ./result
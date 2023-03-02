#! /bin/bash

main_dir=$(pwd)
ACIRJS_REV_SHORT=$(git rev-parse --short HEAD)
ACIR_REV_SHORT="stub"

function build_for_curve() {
    CURVE=$1

    # TODO: Pull and compile ACVM with appropriate field element build config

    cat $main_dir/package.json \
        | jq '.name = "@noir-lang/acvm-bn254"' \
        | jq ".version += \"-$ACIR_REV_SHORT-$ACIRJS_REV_SHORT\"" \
        | jq '.repository = { "type" : "git", "url" : "https://github.com/noir-lang/acvm-js.git" }' \
        | tee $main_dir/package-$CURVE.json

    npx rollup -c --bundleConfigAsCjs

    mkdir -p $main_dir/build/$CURVE
    mv $main_dir/dist $main_dir/build/$CURVE/
    mv $main_dir/package-$CURVE.json $main_dir/build/$CURVE/package.json
}

rm -rf $main_dir/build
build_for_curve bn254
build_for_curve bls12_381

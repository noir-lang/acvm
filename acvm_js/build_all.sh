#! /bin/bash

main_dir=$(pwd)
ACIRJS_REV_SHORT=$(git rev-parse --short HEAD)
ACIR_REV_SHORT="stub"

function build_for_curve() {
    CURVE=$1

    # TODO: Pull and compile ACVM with appropriate field element build config
    npm run wrap:component

    cat $main_dir/package.json \
        | jq '.name = "@noir-lang/acvm-bn254"' \
        | jq ".version += \"-$ACIR_REV_SHORT-$ACIRJS_REV_SHORT\"" \
        | tee $main_dir/package-$CURVE.json

    rm -rf dist
    npx rollup -c --bundleConfigAsCjs
    cp src/acvm_wasm/generated/*.wasm dist/

    mkdir -p $main_dir/build/$CURVE
    mv $main_dir/dist $main_dir/build/$CURVE/
    mv $main_dir/package-$CURVE.json $main_dir/build/$CURVE/package.json
}

rm -rf $main_dir/build
build_for_curve bn254
# TODO: support more curves by building over multiple configs
# build_for_curve bls12_381

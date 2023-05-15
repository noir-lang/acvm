use acvm::{acir::native_types::Witness, FieldElement};
use std::collections::BTreeMap;

pub(crate) fn js_map_to_witness_map(js_map: js_sys::Map) -> BTreeMap<Witness, FieldElement> {
    let mut witness_map: BTreeMap<Witness, FieldElement> = BTreeMap::new();
    js_map.for_each(&mut |value, key| {
        let witness_index = Witness(key.as_string().unwrap().parse::<u32>().unwrap());
        // let witness_value: String = js_sys::BigInt::from(value)
        //     .to_string(16)
        //     .expect("Could not get value of witness")
        //     .into();
        let witness_value: String = value.as_string().expect("Could not get value of witness");

        let witness_value =
            FieldElement::from_hex(&witness_value).expect("could not convert bigint to fields");
        witness_map.insert(witness_index, witness_value);
    });
    witness_map
}

pub(crate) fn witness_map_to_js_map(witness_map: BTreeMap<Witness, FieldElement>) -> js_sys::Map {
    let js_map = js_sys::Map::new();
    for (key, value) in witness_map {
        // This currently maps `0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000000`
        // to the bigint `-1n`. This fails when converting back to a `FieldElement`.

        // let witness_bigint = js_sys::BigInt::from_str(&value.to_hex())
        // .expect("could not convert field to bigint");

        let witness_bigint = wasm_bindgen::JsValue::from_str(&value.to_hex());

        js_map.set(
            &wasm_bindgen::JsValue::from_str(&key.witness_index().to_string()),
            &witness_bigint,
        );
    }
    js_map
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use acvm::{acir::native_types::Witness, FieldElement};
    use wasm_bindgen::JsValue;
    use wasm_bindgen_test::*;

    use super::witness_map_to_js_map;

    #[wasm_bindgen_test]
    fn test_witness_map_to_js() {
        let witness_map = BTreeMap::from([
            (Witness(1), FieldElement::one()),
            (Witness(2), FieldElement::zero()),
            (Witness(3), -FieldElement::one()),
        ]);

        let js_map = witness_map_to_js_map(witness_map);

        assert_eq!(js_map.get(&JsValue::from("1")), JsValue::from_str("1"));
    }
}

use acvm::{acir::native_types::Witness, FieldElement};
use js_sys::JsString;
use std::collections::BTreeMap;
use wasm_bindgen::JsValue;

pub(crate) fn js_value_to_field_element(js_value: JsValue) -> Result<FieldElement, JsString> {
    let hex_str =
        js_value.as_string().ok_or_else(|| "failed to parse field element from non-string")?;

    FieldElement::from_hex(&hex_str)
        .ok_or_else(|| format!("Invalid hex string: '{}'", hex_str).into())
}

pub(crate) fn field_element_to_js_string(field_element: &FieldElement) -> JsString {
    // This currently maps `0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000000`
    // to the bigint `-1n`. This fails when converting back to a `FieldElement`.
    // js_sys::BigInt::from_str(&value.to_hex()).unwrap()

    format!("0x{}", field_element.to_hex()).into()
}

pub(crate) fn js_map_to_witness_map(js_map: js_sys::Map) -> BTreeMap<Witness, FieldElement> {
    let mut witness_map: BTreeMap<Witness, FieldElement> = BTreeMap::new();
    js_map.for_each(&mut |value, key| {
        let witness_index = Witness(key.as_f64().unwrap() as u32);
        let witness_value = js_value_to_field_element(value).unwrap();
        witness_map.insert(witness_index, witness_value);
    });
    witness_map
}

pub(crate) fn witness_map_to_js_map(witness_map: BTreeMap<Witness, FieldElement>) -> js_sys::Map {
    let js_map = js_sys::Map::new();
    for (key, value) in witness_map {
        js_map.set(&js_sys::Number::from(key.witness_index()), &field_element_to_js_string(&value));
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

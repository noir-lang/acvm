use acvm::acir::{circuit::Circuit, native_types::Witness, FieldElement};
use lazy_static::lazy_static;
use spwg_manager::StepwisePwgManager;
use std::{
    collections::BTreeMap,
    sync::{Mutex, MutexGuard},
};
mod spwg_manager;

lazy_static! {
    static ref SINGLETON: Mutex<StepwisePwgManager> = Mutex::new(StepwisePwgManager::new());
}

pub struct Singleton;

impl Singleton {
    pub fn instance() -> MutexGuard<'static, StepwisePwgManager> {
        SINGLETON.lock().unwrap()
    }
}

wit_bindgen::generate!("acvm-wasm");

pub struct ConcreteAcvmWasm;

impl AcvmWasm for ConcreteAcvmWasm {
    fn open_task(acir_bytes: Vec<u8>, initial_witness: Vec<(u32, String)>) -> u32 {
        let acir = Circuit::read(&*acir_bytes).unwrap();
        let initial_witness: BTreeMap<Witness, FieldElement> = initial_witness
            .iter()
            .map(|(idx, hex_str)| (Witness(*idx), FieldElement::from_hex(hex_str).unwrap()))
            .collect();
        Singleton::instance().open_task(initial_witness, acir.opcodes)
    }

    fn step_task(task_id: u32) -> bool {
        Singleton::instance().step_task(task_id)
    }

    fn get_blocker(task_id: u32) -> (String, Vec<String>) {
        let blocker = Singleton::instance().blocker(task_id);
        let name: String = blocker.name.name().into();
        let inputs = blocker.inputs.iter().map(|input| input.to_hex()).collect();
        (name, inputs)
    }

    fn unblock_task(task_id: u32, solution: Vec<String>) {
        let solution = solution
            .iter()
            .map(|hex_str| FieldElement::from_hex(hex_str).unwrap())
            .collect();
        Singleton::instance().unblock_task(task_id, solution);
    }

    fn close_task(task_id: u32) -> Vec<(u32, String)> {
        let intermediate_witness = Singleton::instance().close_task(task_id);
        intermediate_witness
            .into_iter()
            .map(|(Witness(witness_idx), field_element)| (witness_idx, field_element.to_hex()))
            .collect()
    }
}

export_acvm_wasm!(ConcreteAcvmWasm);

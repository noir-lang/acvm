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

    fn step_task(task_id: u32) -> Result<bool, String> {
        match Singleton::instance().step_task(task_id) {
            Ok(done) => Ok(done),
            Err(err) => Err(err.to_string()),
        }
    }

    fn get_blocker(task_id: u32) -> Option<(String, Vec<String>)> {
        match Singleton::instance().blocker(task_id) {
            None => None,
            Some(blocker) => {
                let name: String = blocker.name.name().into();
                let inputs = blocker.inputs.iter().map(|input| input.to_hex()).collect();
                Some((name, inputs))
            }
        }
    }

    fn unblock_task(task_id: u32, solution: Vec<String>) -> Result<(), String> {
        let solution = solution
            .iter()
            .map(|hex_str| FieldElement::from_hex(hex_str).unwrap())
            .collect();
        match Singleton::instance().unblock_task(task_id, solution) {
            Ok(_) => Ok(()),
            Err(err) => Err(err.to_string()),
        }
    }

    fn close_task(task_id: u32) -> Result<Vec<(u32, String)>, String> {
        match Singleton::instance().close_task(task_id) {
            Ok(intermediate_witness) => {
                let pairs = intermediate_witness
                    .into_iter()
                    .map(|(Witness(witness_idx), field_element)| {
                        (witness_idx, field_element.to_hex())
                    })
                    .collect();
                Ok(pairs)
            }
            Err(err) => Err(err.to_string()),
        }
    }
}

export_acvm_wasm!(ConcreteAcvmWasm);

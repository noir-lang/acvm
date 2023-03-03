use self::task_id::{TaskId, TaskIdGen};
use acvm::{
    acir::{circuit::Opcode, native_types::Witness},
    stepwise_pwg::BlackBoxCallResolvedInputs,
    FieldElement, StepwisePartialWitnessGenerator, StepwisePwgError,
};
use std::collections::BTreeMap;

mod task_id;

pub struct StepwisePwgManager {
    task_id_gen: TaskIdGen,
    tasks: BTreeMap<u32, StepwisePartialWitnessGenerator>,
}

impl Default for StepwisePwgManager {
    fn default() -> Self {
        StepwisePwgManager::new()
    }
}

impl StepwisePwgManager {
    pub fn new() -> Self {
        StepwisePwgManager { task_id_gen: TaskIdGen::new(), tasks: BTreeMap::new() }
    }

    pub fn open_task(
        &mut self,
        initial_witness: BTreeMap<Witness, FieldElement>,
        opcodes: Vec<Opcode>,
    ) -> TaskId {
        let task_id = self.task_id_gen.get_unique_id();
        let spwg = StepwisePartialWitnessGenerator::new(initial_witness, opcodes);
        self.tasks.insert(task_id, spwg);
        task_id
    }

    pub fn step_task(&mut self, task_id: TaskId) -> Result<bool, StepwisePwgError> {
        let spwg = self.tasks.get_mut(&task_id).unwrap();
        spwg.step()?;
        Ok(spwg.is_done())
    }

    pub fn blocker(&self, task_id: TaskId) -> Option<BlackBoxCallResolvedInputs> {
        self.tasks.get(&task_id).unwrap().required_black_box_func_call()
    }

    pub fn unblock_task(
        &mut self,
        task_id: TaskId,
        solution: Vec<FieldElement>,
    ) -> Result<(), StepwisePwgError> {
        self.tasks.get_mut(&task_id).unwrap().apply_blackbox_call_solution(solution)
    }

    pub fn close_task(
        &mut self,
        task_id: TaskId,
    ) -> Result<BTreeMap<Witness, FieldElement>, StepwisePwgError> {
        let spwg = self.tasks.remove(&task_id).unwrap();
        spwg.intermediate_witness()
    }
}

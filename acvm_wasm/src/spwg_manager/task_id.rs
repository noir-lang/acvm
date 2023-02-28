pub type TaskId = u32;

pub struct TaskIdGen {
    next_id: TaskId,
}

impl TaskIdGen {
    pub fn new() -> Self {
        TaskIdGen { next_id: 0 }
    }

    pub fn get_unique_id(&mut self) -> TaskId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

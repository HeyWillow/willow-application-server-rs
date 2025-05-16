use crate::willow::worker::WorkerData;

#[allow(dead_code)]
#[derive(Debug)]
pub struct WasState {
    worker_data: WorkerData,
}

impl WasState {
    #[must_use]
    pub fn new(worker_data: WorkerData) -> Self {
        Self { worker_data }
    }

    #[must_use]
    pub fn worker_data(&self) -> &WorkerData {
        &self.worker_data
    }
}

use async_runtime::runtime::async_runtime::{AsyncRuntime, AsyncRuntimeBuilder};
use async_runtime::scheduler::execution_engine::ExecutionEngineBuilder;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::debug;

#[derive(Serialize, Deserialize, Debug)]
pub struct Runtime {
    task_queue_size: u32,
    workers: usize,
    thread_priority: Option<u8>,
    thread_affinity: Option<usize>,
    thread_stack_size: Option<u64>,
}

impl Runtime {
    pub fn new(inputs: &Option<String>) -> Self {
        let v: Value = serde_json::from_str(inputs.as_deref().unwrap()).unwrap();
        serde_json::from_value(v["runtime"].clone()).unwrap()
    }

    pub fn build(&self) -> AsyncRuntime {
        debug!("Creating AsyncRuntime with: {:?}", self);
        let mut execution_engine_builder = ExecutionEngineBuilder::new().task_queue_size(self.task_queue_size).workers(self.workers);
        if let Some(thread_priority) = self.thread_priority {
            execution_engine_builder = execution_engine_builder.thread_priority(thread_priority);
        }
        if let Some(thread_affinity) = self.thread_affinity {
            execution_engine_builder = execution_engine_builder.thread_affinity(thread_affinity);
        }
        if let Some(thread_stack_size) = self.thread_stack_size {
            execution_engine_builder = execution_engine_builder.thread_stack_size(thread_stack_size);
        }

        let (builder, _engine_id) = AsyncRuntimeBuilder::new().with_engine(execution_engine_builder);
        builder.build().unwrap()
    }
}

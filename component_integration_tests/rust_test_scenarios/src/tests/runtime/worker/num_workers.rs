use crate::internals::execution_barrier::{MultiExecutionBarrier, RuntimeJoiner};
use crate::internals::runtime_helper::Runtime;
use async_runtime::spawn;
use foundation::threading::thread_wait_barrier::ThreadReadyNotifier;
use std::hint;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use test_scenarios_rust::scenario::Scenario;
use tracing::info;

async fn blocking_task(
    id: usize,
    trace_ids: Arc<AtomicBool>,
    counter: Arc<AtomicUsize>,
    counter_unblock_value: usize,
    notifier: ThreadReadyNotifier,
) {
    // Allow only required workers to trace.
    if trace_ids.load(Ordering::Acquire) {
        let id = format!("worker_{id}");
        info!(id);
    }
    counter.fetch_add(1, Ordering::Release);
    notifier.ready();
    while counter.load(Ordering::Acquire) < counter_unblock_value {
        hint::spin_loop()
    }
}

pub struct NumWorkers;

impl Scenario for NumWorkers {
    fn name(&self) -> &str {
        "num_workers"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let builder = Runtime::new(&input);
        let exec_engine = builder.exec_engines().get(0).unwrap();
        let num_workers = exec_engine.workers;
        let mut rt = builder.build();

        let _ = rt.block_on(async move {
            let mut joiner = RuntimeJoiner::new();
            let mid_barrier = MultiExecutionBarrier::new(num_workers);
            let mut mid_notifiers = mid_barrier.get_notifiers();

            let counter = Arc::new(AtomicUsize::new(0));
            let trace_ids = Arc::new(AtomicBool::new(true));

            // Current worker is spawning tasks for other workers.
            info!(id = "worker_0");

            // Spawn tasks.
            for id in 1..=num_workers {
                let notifier = mid_notifiers.pop().unwrap();
                joiner.add_handle(spawn(blocking_task(id, trace_ids.clone(), counter.clone(), num_workers, notifier)));
            }

            // Wait is expected to fail on timeout.
            // There's always one task more than available workers.
            // Wait time must be longer for more tasks.
            let wait_s = if num_workers > 32 { 10 } else { 3 };
            let result = mid_barrier.wait_for_notification(Duration::from_secs(wait_s));

            // Disable tracing.
            trace_ids.store(false, Ordering::Release);

            match result {
                // Expected `RuntimeErrors` are not exactly suitable to express intent.
                Ok(_) => info!(wait_result = "ok"),
                // Only timeout is expected.
                Err(e) => {
                    let expected_message = format!("Failed to join tasks after {wait_s} seconds");
                    if e == expected_message {
                        info!(wait_result = "timeout");
                    } else {
                        info!(wait_result = "other", error = e)
                    }
                }
            }

            Ok(joiner.wait_for_all().await)
        });

        Ok(())
    }
}

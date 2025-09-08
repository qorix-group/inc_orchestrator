use crate::internals::execution_barrier::RuntimeJoiner;
use crate::internals::runtime_helper::Runtime;
use async_runtime::spawn;
use std::mem::MaybeUninit;
use test_scenarios_rust::scenario::Scenario;
use tracing::info;

async fn show_thread_affinity(id: usize) {
    unsafe {
        let current_thread = 0;
        let mut cpu_set = MaybeUninit::<libc::cpu_set_t>::zeroed().assume_init();
        let cpu_set_size = std::mem::size_of::<libc::cpu_set_t>();
        let rc = libc::sched_getaffinity(current_thread, cpu_set_size, &mut cpu_set);
        if rc != 0 {
            let errno = *libc::__errno_location();
            panic!("libc::sched_getaffinity failed, rc: {rc}, errno: {errno}");
        }

        let mut affinity = Vec::new();
        for i in 0..libc::CPU_SETSIZE as usize {
            if libc::CPU_ISSET(i, &cpu_set) {
                affinity.push(i);
            }
        }

        let id = format!("worker_{id}");
        let affinity = format!("{affinity:?}");

        info!(id, affinity);
    }
}

pub struct ThreadAffinity;

impl Scenario for ThreadAffinity {
    fn name(&self) -> &str {
        "thread_affinity"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let builder = Runtime::new(&input);
        let exec_engine = builder.exec_engines().first().unwrap();
        let num_workers = exec_engine.workers;
        let mut rt = builder.build();

        let _ = rt.block_on(async move {
            let mut joiner = RuntimeJoiner::new();

            // Show parameters of current thread.
            show_thread_affinity(0).await;

            // Show parameters of other threads.
            for id in 1..num_workers {
                joiner.add_handle(spawn(show_thread_affinity(id)));
            }

            Ok(joiner.wait_for_all().await)
        });

        Ok(())
    }
}

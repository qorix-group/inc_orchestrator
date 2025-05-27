//
// Copyright (c) 2025 Contributors to the Eclipse Foundation
//
// See the NOTICE file(s) distributed with this work for additional
// information regarding copyright ownership.
//
// This program and the accompanying materials are made available under the
// terms of the Apache License Version 2.0 which is available at
// <https://www.apache.org/licenses/LICENSE-2.0>
//
// SPDX-License-Identifier: Apache-2.0
//
use iceoryx2_bb_container::vec::Vec;
use std::future::Future;

use crate::scheduler::execution_engine::{ExecutionEngine, ExecutionEngineBuilder};
use foundation::containers::growable_vec::GrowableVec;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RuntimeErrors {
    EngineNotAvailable, // An ExecutionEngine with the requested id was not built before.
    ResultNotThere,     // Could not get the return value from a Future.
    NoTaskRunning,      // There is currently no Task running to wait/join for.
    TaskAlreadyRunning, // There is already a Task running.
    NoResultAvailable,
}

pub struct AsyncRuntimeBuilder {
    engine_builders: GrowableVec<ExecutionEngineBuilder>,
    next_id: usize,
}

impl Default for AsyncRuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncRuntimeBuilder {
    pub fn new() -> Self {
        Self {
            engine_builders: GrowableVec::<ExecutionEngineBuilder>::new(1),
            next_id: 0,
        }
    }

    ///
    /// Adds engine to the runtime using `builder` configured by caller.
    ///
    pub fn with_engine(mut self, builder: ExecutionEngineBuilder) -> (Self, usize) {
        let id = self.next_id;
        self.engine_builders.push(builder);
        self.next_id += 1;
        (self, id)
    }

    #[allow(clippy::result_unit_err)]
    pub fn build(mut self) -> Result<AsyncRuntime, ()> {
        let mut engines = GrowableVec::<ExecutionEngine>::new(1);
        loop {
            let item = self.engine_builders.pop();
            if let Some(builder) = item {
                engines.push(builder.build());
            } else {
                break;
            }
        }

        engines.reverse();
        Ok(AsyncRuntime { engines: engines.into() })
    }
}

///
/// The main runtime for managing and executing asynchronous tasks across one or more engines.
///
/// `AsyncRuntime` owns a set of [`ExecutionEngine`] instances and provides a unified API to
/// execute, wait for, and shut down tasks. Engines are configured and added via [`AsyncRuntimeBuilder`].
pub struct AsyncRuntime {
    engines: Vec<ExecutionEngine>,
}

impl AsyncRuntime {
    /// Runs the given future to completion on the default engine, blocking the current thread.
    ///
    /// Returns the result of the future or an error if the engine is not available.
    pub fn block_on<T: Future<Output = Result<u32, RuntimeErrors>> + 'static + Send>(&mut self, future: T) -> Result<u32, RuntimeErrors> {
        self.block_on_engine(0, future)
    }

    /// Runs the given future to completion on the specified engine, blocking the current thread.
    ///
    /// Returns the result of the future or an error if the engine is not available.
    pub fn block_on_engine<T: Future<Output = Result<u32, RuntimeErrors>> + 'static + Send>(
        &mut self,
        engine_id: usize,
        future: T,
    ) -> Result<u32, RuntimeErrors> {
        // The following line does:
        // 1. Get the engine with the given `engine_id` as index into the `self.engines` vector.
        // 2. If the engine is not found, return an `RuntimeErrors::EngineNotAvailable` error. This
        // is the `ok_or()?` part.
        // 3. Call `block_on` on the found engine with the provided future.
        self.engines.get_mut(engine_id).ok_or(RuntimeErrors::EngineNotAvailable)?.block_on(future)
    }

    /// Starts the given future asynchronously on the default engine.
    ///
    /// The result can be retrieved later using [`wait_for_engine_id`] or just the completion of
    /// all engines with [`wait_for_all_engines`].
    pub fn spawn<T: Future<Output = Result<u32, RuntimeErrors>> + 'static + Send>(&mut self, future: T) -> Result<(), RuntimeErrors> {
        self.spawn_in_engine(0, future)
    }

    /// Starts the given future asynchronously on the specified engine.
    ///
    /// The result can be retrieved later using [`wait_for_engine_id`] or just the completion of
    /// all engines with [`wait_for_all_engines`].
    pub fn spawn_in_engine<T: Future<Output = Result<u32, RuntimeErrors>> + 'static + Send>(
        &mut self,
        engine_id: usize,
        future: T,
    ) -> Result<(), RuntimeErrors> {
        self.engines
            .get_mut(engine_id)
            .ok_or(RuntimeErrors::EngineNotAvailable)?
            .run_in_engine(future)
    }

    /// Stops all engines and their worker threads.
    ///
    /// Any running tasks will be aborted after their current iteration, even if they still return
    /// `Poll::Pending`. They will be run until they return the next `Poll::Ready` or
    /// `Poll::Pending` and then stopped.
    pub fn shutdown(&mut self) {
        for engine in self.engines.iter_mut() {
            engine.stop();
        }
    }

    /// Waits for the result of the currently running task on the specified engine.
    ///
    /// This is a companion for [`spawn_in_engine`] or [`spawn`]. Tasks started with these
    /// functions can be joined here. This waits for the Future you gave to the [`spawn`] /
    /// [`spawn_in_engine`] function to finish. If there where any addtional tasks spawned from
    /// within the starting Future, they will not be waited for. You should take care of
    /// waiting for them yourself, if you have the need to do so.
    /// Returns the result or an error if no task is running.
    pub fn wait_for_engine(&mut self, engine_id: usize) -> Result<u32, RuntimeErrors> {
        self.engines.get_mut(engine_id).ok_or(RuntimeErrors::EngineNotAvailable)?.wait_for()
    }

    /// Waits for all engines to finish their currently running tasks.
    ///
    /// The results of the engines are consumed but not delivered anywhere. After calling this
    /// function the engines do not have a result.
    pub fn wait_for_all_engines(&mut self) {
        for engine in self.engines.iter_mut() {
            let _ = engine.wait_for();
        }
    }
}

#[cfg(test)]
#[cfg(not(loom))]
// This is because of the disabled miri tests below
#[allow(unused_imports)]
mod tests {
    use super::*;
    use foundation::threading::thread_wait_barrier::ThreadWaitBarrier;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use std::{fs, thread, time::Duration};

    fn count_threads() -> usize {
        fs::read_dir("/proc/self/task").unwrap().count()
    }

    /// Because this test is counting the running threads before while and after creation of
    /// an ExecutionEngine and out whole testsuite is running multithreaded, this test can
    /// only work in isolation. Without any other instance creating or destroying threads.
    /// So this test is not run with the whole testsuite. If you want to run it, you can do so with:
    /// ```
    /// cargo t -- runtime_drop_cleanup --include-ignored
    /// ```
    #[test]
    #[ignore]
    fn test_async_runtime_drop_cleanup() {
        let before = count_threads();

        let drop_counter1 = Arc::new(AtomicUsize::new(0));
        let drop_counter1_clone = drop_counter1.clone();

        let threads_while: usize = {
            let (builder, engine_id) = AsyncRuntimeBuilder::new().with_engine(ExecutionEngineBuilder::new().task_queue_size(8).workers(3));
            let mut runtime = builder.build().unwrap();
            let t = runtime.block_on_engine(engine_id, async move {
                drop_counter1_clone.fetch_add(1, Ordering::SeqCst);
                Ok(count_threads() as u32)
            });

            t.unwrap().try_into().unwrap()
        };

        thread::sleep(Duration::from_millis(100));

        let after = count_threads();

        println!("{before}-{threads_while}-{after}");
        assert!(threads_while == before + 3);
        assert!(drop_counter1.load(Ordering::SeqCst) > 0, "Future was not excuted.");
        assert_eq!(after, before);
    }

    #[test]
    // miri does not like this test for some reason. Disable it for now. The message is
    // ```
    // error: unsupported operation: can't call foreign function `pthread_attr_init` on OS `linux`
    // ```
    // See https://github.com/qorix-group/inc_orchestrator_internal/actions/runs/15675294733/job/44154074863?pr=47
    // for an example CI run.
    #[cfg(not(miri))]
    fn test_async_runtime_return_value() {
        let (builder, engine_id) = AsyncRuntimeBuilder::new().with_engine(ExecutionEngineBuilder::new().task_queue_size(8).workers(3));
        let mut runtime = builder.build().unwrap();
        let ret = runtime.block_on_engine(engine_id, async move { Ok(23) });

        assert_eq!(ret, Ok(23));

        let (builder, engine_id) = AsyncRuntimeBuilder::new().with_engine(ExecutionEngineBuilder::new().task_queue_size(8).workers(3));
        let mut runtime = builder.build().unwrap();
        let ret = runtime.block_on_engine(engine_id, async move { Ok(42) });

        assert_eq!(ret, Ok(42));
    }

    #[test]
    // miri does not like this test for some reason. Disable it for now. The message is
    // ```
    // error: unsupported operation: can't call foreign function `pthread_attr_init` on OS `linux`
    // ```
    // See https://github.com/qorix-group/inc_orchestrator_internal/actions/runs/15675294733/job/44154074863?pr=47
    // for an example CI run.
    #[cfg(not(miri))]
    fn test_async_runtime_async_run_and_late_wait() {
        let (builder, engine_id) = AsyncRuntimeBuilder::new().with_engine(ExecutionEngineBuilder::new().task_queue_size(8).workers(2));
        let mut runtime = builder.build().unwrap();

        let barrier = Arc::new(ThreadWaitBarrier::new(1));
        let ready_notifier = barrier.get_notifier().unwrap();

        runtime
            .spawn_in_engine(engine_id, async move {
                thread::sleep(Duration::from_millis(50));
                ready_notifier.ready();
                Ok(123u32)
            })
            .unwrap();

        assert_eq!(Ok(()), barrier.wait_for_all(Duration::from_secs(1)));

        let result = runtime.wait_for_engine(engine_id);
        assert_eq!(result, Ok(123));
    }

    #[test]
    fn test_async_runtime_wait_for_without_task() {
        let (builder, engine_id) = AsyncRuntimeBuilder::new().with_engine(ExecutionEngineBuilder::new().task_queue_size(8).workers(1));
        let mut runtime = builder.build().unwrap();

        // no task running, try to get result immediately
        let result = runtime.wait_for_engine(engine_id);
        assert_eq!(result, Err(RuntimeErrors::NoTaskRunning));
    }

    #[test]
    // miri does not like this test for some reason. Disable it for now. The message is
    // ```
    // error: unsupported operation: can't call foreign function `pthread_attr_init` on OS `linux`
    // ```
    // See https://github.com/qorix-group/inc_orchestrator_internal/actions/runs/15675294733/job/44154074863?pr=47
    // for an example CI run.
    #[cfg(not(miri))]
    fn test_async_runtime_wait_for_all_engines() {
        let (builder, engine_id1) = AsyncRuntimeBuilder::new().with_engine(ExecutionEngineBuilder::new().task_queue_size(8).workers(2));
        let (builder, engine_id2) = builder.with_engine(ExecutionEngineBuilder::new().task_queue_size(8).workers(2));
        let mut runtime = builder.build().unwrap();

        runtime
            .spawn_in_engine(engine_id1, async move {
                thread::sleep(Duration::from_millis(50));
                Ok(1u32)
            })
            .unwrap();

        runtime
            .spawn_in_engine(engine_id2, async move {
                thread::sleep(Duration::from_millis(100));
                Ok(2u32)
            })
            .unwrap();

        runtime.wait_for_all_engines();

        // After wait_for_all_engines the engines are stopped and do not deliver an result anymore.
        // There is no task running in them.
        let res1 = runtime.wait_for_engine(engine_id1);
        let res2 = runtime.wait_for_engine(engine_id2);

        assert!(matches!(res1, Err(RuntimeErrors::NoTaskRunning)));
        assert!(matches!(res2, Err(RuntimeErrors::NoTaskRunning)));
    }

    #[test]
    fn test_async_runtime_builder_build_preserves_order() {
        // three builders with worker_count as a distinguishable property
        let builder1 = ExecutionEngineBuilder::new().workers(1);
        let builder2 = ExecutionEngineBuilder::new().workers(2);
        let builder3 = ExecutionEngineBuilder::new().workers(3);

        // Add them in order
        let (builder, _id1) = AsyncRuntimeBuilder::new().with_engine(builder1);
        let (builder, _id2) = builder.with_engine(builder2);
        let (builder, _id3) = builder.with_engine(builder3);

        // Build the runtime
        let runtime = builder.build().unwrap();

        // Check that the default engine (index 0) is the one that was added first (worker_count = 1)
        assert_eq!(runtime.engines.get(0).unwrap().worker_count(), 1);

        // Check the order of engines
        for (i, engine) in runtime.engines.iter().enumerate() {
            assert_eq!(i + 1, engine.worker_count());
        }
    }
}

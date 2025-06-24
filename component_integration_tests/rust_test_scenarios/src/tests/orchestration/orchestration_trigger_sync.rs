use crate::internals::helpers::runtime_helper::Runtime;
use crate::internals::scenario::Scenario;

use super::*;
use foundation::prelude::*;
use futures::future;
use orchestration::{
    api::{design::Design, Orchestration},
    common::DesignConfig,
};
use std::vec::Vec;

fn simple_checkpoint(id: &str) {
    info!(id = id);
}

fn location_checkpoint(id: &str, location: &str) {
    info!(id = id, location = location);
}

async fn blocking_sleep_task() -> InvokeResult {
    location_checkpoint("blocking_sleep_task", "begin");
    std::thread::sleep(std::time::Duration::from_secs(1));
    location_checkpoint("blocking_sleep_task", "end");
    Ok(())
}

async fn basic_task_a() -> InvokeResult {
    simple_checkpoint("basic_task_A");
    Ok(())
}

async fn basic_task_b() -> InvokeResult {
    simple_checkpoint("basic_task_B");
    Ok(())
}

pub struct OneTriggerOneSyncTwoPrograms;

fn trigger_sync_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("TriggerSequence".into(), DesignConfig::default());

    let blocking_sleep_task_tag = design.register_invoke_async("blocking_sleep".into(), blocking_sleep_task)?;
    let basic_task_tag = design.register_invoke_async("basic_task_a".into(), basic_task_a)?;
    let evt_sync = design.register_event(Tag::from_str_static("evt_sync"))?;

    design.add_program("trigger_program", move |_design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(Invoke::from_tag(&blocking_sleep_task_tag))
                .with_step(TriggerBuilder::from_tag(&evt_sync))
                .build(),
        );

        Ok(())
    });

    design.add_program("sync_program", move |_design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(SyncBuilder::from_design("evt_sync", _design_instance))
                .with_step(Invoke::from_tag(&basic_task_tag))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks trigger and sync between two programs
impl Scenario for OneTriggerOneSyncTwoPrograms {
    fn get_name(&self) -> &'static str {
        "1_trigger_1_sync_2_programs"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();

        let mut orch = Orchestration::new()
            .add_design(trigger_sync_design().expect("Failed to create design"))
            .design_done();

        let mut deployment = orch.get_deployment_mut();
        deployment.bind_events_as_local(&["evt_sync".into()]).expect("Failed to specify event");

        let mut programs = orch.create_programs().unwrap();
        let _ = rt.block_on(async move {
            let mut joiner = Vec::new();
            for program in programs.programs.as_mut_slice() {
                joiner.push(program.run_n(1));
            }

            future::join_all(joiner).await;
            Ok(0)
        });

        Ok(())
    }
}

pub struct OneTriggerTwoSyncsThreePrograms;

fn trigger_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("TriggerSequence".into(), DesignConfig::default());

    let blocking_sleep_task_tag = design.register_invoke_async("blocking_sleep".into(), blocking_sleep_task)?;
    let evt_sync = design.register_event(Tag::from_str_static("evt_sync"))?;

    design.add_program("trigger_program", move |_design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(Invoke::from_tag(&blocking_sleep_task_tag))
                .with_step(TriggerBuilder::from_tag(&evt_sync))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

fn sync_design_a() -> Result<Design, CommonErrors> {
    let mut design = Design::new("SyncA".into(), DesignConfig::default());

    let basic_task_tag = design.register_invoke_async("basic_task_a".into(), basic_task_a)?;
    let evt_sync = design.register_event(Tag::from_str_static("evt_sync"))?;

    design.add_program("sync_program_A", move |_design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(SyncBuilder::from_tag(&evt_sync))
                .with_step(Invoke::from_tag(&basic_task_tag))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

fn sync_design_b() -> Result<Design, CommonErrors> {
    let mut design = Design::new("SyncB".into(), DesignConfig::default());

    let basic_task_tag = design.register_invoke_async("basic_task".into(), basic_task_b)?;
    let evt_sync = design.register_event(Tag::from_str_static("evt_sync"))?;

    design.add_program("sync_program_B", move |_design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(SyncBuilder::from_tag(&evt_sync))
                .with_step(Invoke::from_tag(&basic_task_tag))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks trigger in one program and sync in other two programs
impl Scenario for OneTriggerTwoSyncsThreePrograms {
    fn get_name(&self) -> &'static str {
        "1_trigger_2_syncs_3_programs"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();

        let mut orch = Orchestration::new()
            .add_design(sync_design_a().expect("Failed to create design"))
            .add_design(sync_design_b().expect("Failed to create design"))
            .add_design(trigger_design().expect("Failed to create design"))
            .design_done();

        let mut deployment = orch.get_deployment_mut();
        deployment.bind_events_as_local(&["evt_sync".into()]).expect("Failed to specify event");

        let mut programs = orch.create_programs().unwrap();

        let _ = rt.block_on(async move {
            let mut joiner = Vec::new();
            for program in programs.programs.as_mut_slice() {
                joiner.push(program.run_n(1));
            }

            future::join_all(joiner).await;
            Ok(0)
        });

        Ok(())
    }
}

pub struct TriggerAndSyncInNestedBranches;

fn nested_trigger_sync_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("TriggerSequence".into(), DesignConfig::default());

    let blocking_sleep_task_tag = design.register_invoke_async("blocking_sleep".into(), blocking_sleep_task)?;
    let basic_task_tag = design.register_invoke_async("basic_task".into(), basic_task_a)?;
    let evt_sync = design.register_event(Tag::from_str_static("evt_sync"))?;

    design.add_program("trigger_sync_program", move |_design_instance, builder| {
        builder.with_run_action(
            ConcurrencyBuilder::new()
                .with_branch(
                    SequenceBuilder::new()
                        .with_step(Invoke::from_tag(&blocking_sleep_task_tag))
                        .with_step(TriggerBuilder::from_tag(&evt_sync))
                        .build(),
                )
                .with_branch(
                    SequenceBuilder::new()
                        .with_step(SyncBuilder::from_design("evt_sync", _design_instance))
                        .with_step(Invoke::from_tag(&basic_task_tag))
                        .build(),
                )
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks trigger and sync in the separate concurrency branches in a single program
impl Scenario for TriggerAndSyncInNestedBranches {
    fn get_name(&self) -> &'static str {
        "trigger_and_sync_in_nested_branches"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();

        let mut orch = Orchestration::new()
            .add_design(nested_trigger_sync_design().expect("Failed to create design"))
            .design_done();

        let mut deployment = orch.get_deployment_mut();
        deployment.bind_events_as_local(&["evt_sync".into()]).expect("Failed to specify event");

        let mut programs = orch.create_programs().unwrap();

        let _ = rt.block_on(async move {
            let mut joiner = Vec::new();
            for program in programs.programs.as_mut_slice() {
                joiner.push(program.run_n(1));
            }

            future::join_all(joiner).await;
            Ok(0)
        });

        Ok(())
    }
}

pub struct TriggerSyncOneAfterAnother;

fn trigger_sync_oaa_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("TriggerSequence".into(), DesignConfig::default());

    let basic_task_a_tag = design.register_invoke_async("basic_task_a".into(), basic_task_a)?;
    let basic_task_b_tag = design.register_invoke_async("basic_task_b".into(), basic_task_b)?;
    let evt_sync = design.register_event(Tag::from_str_static("evt_sync"))?;

    design.add_program("trigger_sync_program", move |_design_instance, builder| {
        builder.with_run_action(
            SequenceBuilder::new()
                .with_step(Invoke::from_tag(&basic_task_a_tag))
                .with_step(TriggerBuilder::from_tag(&evt_sync))
                .with_step(SyncBuilder::from_tag(&evt_sync))
                .with_step(Invoke::from_tag(&basic_task_b_tag))
                .build(),
        );

        Ok(())
    });

    Ok(design)
}

/// Checks trigger and sync as sequential steps in a single program
impl Scenario for TriggerSyncOneAfterAnother {
    fn get_name(&self) -> &'static str {
        "trigger_sync_one_after_another"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();

        let mut orch = Orchestration::new()
            .add_design(trigger_sync_oaa_design().expect("Failed to create design"))
            .design_done();

        let mut deployment = orch.get_deployment_mut();
        deployment.bind_events_as_local(&["evt_sync".into()]).expect("Failed to specify event");

        let mut programs = orch.create_programs().unwrap();

        let _ = rt.block_on(async move {
            let mut joiner = Vec::new();
            for program in programs.programs.as_mut_slice() {
                joiner.push(program.run_n(1));
            }

            future::join_all(joiner).await;
            Ok(0)
        });

        Ok(())
    }
}

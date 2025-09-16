use crate::internals::runtime_helper::Runtime;
use test_scenarios_rust::scenario::Scenario;

use core::time::Duration;
use foundation::prelude::*;
use orchestration::common::tag::Tag;
use orchestration::core::metering::MeterTrait;

use super::*;
use orchestration::{
    api::{design::Design, Orchestration},
    common::DesignConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Custom Meter implementation with tracing instead of print
pub struct InfoMeter {
    id: Tag,
}

impl MeterTrait for InfoMeter {
    fn new(id: Tag) -> Self {
        Self { id }
    }

    fn reset(&mut self) {}

    fn meter<T: core::fmt::Debug>(&mut self, duration: &Duration, info: T) {
        info!(meter_id = self.id.tracing_str(), iteration_time_us = duration.as_micros(), info = ?info);
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DesignTypeTestInput {
    run_type: String,
    run_count: u64,
    run_delay: u64,
}

impl DesignTypeTestInput {
    pub fn new(inputs: &Option<String>) -> Self {
        let input_string = inputs.as_ref().expect("Test input is expected");
        let v: Value = serde_json::from_str(input_string).expect("Failed to parse input string");
        serde_json::from_value(v["test"].clone()).expect("Failed to parse \"test\" field")
    }
}

fn simple_run_design() -> Result<Design, CommonErrors> {
    let mut design = Design::new("simple_run_design".into(), DesignConfig::default());

    let start_tag = design.register_invoke_async("Start".into(), move || async move {
        info!(id = "start", message = "Program was started");
        Ok(())
    })?;
    let basic_task_tag = design.register_invoke_async("basic_task".into(), basic_task)?;
    let stop_tag = design.register_invoke_async("Stop".into(), move || async move {
        info!(id = "stop", message = "Program was stopped");
        Ok(())
    })?;

    design.add_program("simple_run_program", move |design, builder| {
        builder
            .with_start_action(Invoke::from_tag(&start_tag, design.config()))
            .with_run_action(
                SequenceBuilder::new()
                    .with_step(Invoke::from_tag(&basic_task_tag, design.config()))
                    .build(),
            )
            .with_stop_action(Invoke::from_tag(&stop_tag, design.config()), std::time::Duration::from_secs(1));

        Ok(())
    });

    Ok(design)
}

pub struct ProgramRun;

impl Scenario for ProgramRun {
    fn name(&self) -> &str {
        "program_run"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();
        let logic = DesignTypeTestInput::new(&input);

        let orch = Orchestration::new()
            .add_design(simple_run_design().expect("Failed to create simple design"))
            .design_done();

        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        match logic.run_type.as_str() {
            "run" => {
                let _ = rt.block_on(async move {
                    let mut program = programs.pop().expect("Failed to pop program");
                    let _ = program.run().await;
                    Ok(0)
                });
            }
            "run_cycle" => {
                let _ = rt.block_on(async move {
                    let mut program = programs.pop().expect("Failed to pop program");
                    let _ = program.run_cycle(std::time::Duration::from_millis(logic.run_delay)).await;
                    Ok(0)
                });
            }
            "run_n" => {
                let _ = rt.block_on(async move {
                    let mut program = programs.pop().expect("Failed to pop program");
                    let _ = program.run_n(logic.run_count as usize).await;
                    Ok(0)
                });
            }
            "run_n_cycle" => {
                let _ = rt.block_on(async move {
                    let mut program = programs.pop().expect("Failed to pop program");
                    let _ = program
                        .run_n_cycle(logic.run_count as usize, std::time::Duration::from_millis(logic.run_delay))
                        .await;
                    Ok(0)
                });
            }
            _ => {
                return Err(format!("Unknown run_type: {}", logic.run_type));
            }
        }

        Ok(())
    }
}

pub struct ProgramRunMetered;

impl Scenario for ProgramRunMetered {
    fn name(&self) -> &str {
        "program_run_metered"
    }

    fn run(&self, input: Option<String>) -> Result<(), String> {
        let mut rt = Runtime::new(&input).build();
        let logic = DesignTypeTestInput::new(&input);

        let orch = Orchestration::new()
            .add_design(simple_run_design().expect("Failed to create simple design"))
            .design_done();

        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        match logic.run_type.as_str() {
            "run_metered" => {
                let _ = rt.block_on(async move {
                    let mut program = programs.pop().expect("Failed to pop program");
                    let _ = program.run_metered::<InfoMeter>().await;
                    Ok(0)
                });
            }
            "run_cycle_metered" => {
                let _ = rt.block_on(async move {
                    let mut program = programs.pop().expect("Failed to pop program");
                    let _ = program
                        .run_cycle_metered::<InfoMeter>(std::time::Duration::from_millis(logic.run_delay))
                        .await;
                    Ok(0)
                });
            }
            "run_n_metered" => {
                let _ = rt.block_on(async move {
                    let mut program = programs.pop().expect("Failed to pop program");
                    let _ = program.run_n_metered::<InfoMeter>(logic.run_count as usize).await;
                    Ok(0)
                });
            }
            "run_n_cycle_metered" => {
                let _ = rt.block_on(async move {
                    let mut program = programs.pop().expect("Failed to pop program");
                    let _ = program
                        .run_n_cycle_metered::<InfoMeter>(logic.run_count as usize, std::time::Duration::from_millis(logic.run_delay))
                        .await;
                    Ok(0)
                });
            }
            _ => {
                return Err(format!("Unknown run_type: {}", logic.run_type));
            }
        }

        Ok(())
    }
}

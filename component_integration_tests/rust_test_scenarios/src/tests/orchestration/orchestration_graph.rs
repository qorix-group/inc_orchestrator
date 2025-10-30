use super::*;
use crate::internals::runtime_helper::Runtime;
use kyron_foundation::prelude::*;
use orchestration::api::design::Design;
use orchestration::api::Orchestration;
use orchestration::common::DesignConfig;
use serde::Deserialize;
use serde_json::Value;
use test_scenarios_rust::scenario::Scenario;

#[derive(Deserialize, Debug)]
struct TestInput {
    graph_name: String,
}

impl TestInput {
    pub fn new(input: &str) -> Self {
        let v: Value = serde_json::from_str(input).expect("Failed to parse input string");
        serde_json::from_value(v["test"].clone()).expect("Failed to parse \"test\" field")
    }
}

struct GraphHandler;

impl GraphHandler {
    fn create_graph_design(graph_name: &str) -> Result<Design, CommonErrors> {
        let mut design = Design::new("GraphDesign".into(), DesignConfig::default());

        let target_graph = Self::choose_graph(graph_name);
        let graph_action = target_graph(&design);

        // Create a program with some actions
        design.add_program("GraphDesignProgram", move |_design_instance, builder| {
            builder.with_run_action(SequenceBuilder::new().with_step(graph_action).build());
            Ok(())
        });

        Ok(design)
    }

    fn graph_two_nodes(design: &Design) -> Box<LocalGraphAction> {
        let mut graph_builder = LocalGraphActionBuilder::new();

        let n0 = graph_builder.add_node(JustLogAction::new("node0"));
        let n1 = graph_builder.add_node(JustLogAction::new("node1"));

        graph_builder.add_edges(n0, &[n1]).build(design)
    }

    fn graph_no_edges(design: &Design) -> Box<LocalGraphAction> {
        let mut graph_builder = LocalGraphActionBuilder::new();

        graph_builder.add_node(JustLogAction::new("node1"));
        graph_builder.add_node(JustLogAction::new("node0"));

        graph_builder.build(design)
    }

    fn graph_one_node(design: &Design) -> Box<LocalGraphAction> {
        let mut graph_builder = LocalGraphActionBuilder::new();

        graph_builder.add_node(JustLogAction::new("node0"));

        graph_builder.build(design)
    }

    fn graph_empty_edges(design: &Design) -> Box<LocalGraphAction> {
        let mut graph_builder = LocalGraphActionBuilder::new();

        let n0 = graph_builder.add_node(JustLogAction::new("node0"));
        let n1 = graph_builder.add_node(JustLogAction::new("node1"));
        let n2 = graph_builder.add_node(JustLogAction::new("node2"));

        graph_builder.add_edges(n0, &[]).add_edges(n1, &[]).add_edges(n2, &[]).build(design)
    }

    fn graph_multiple_edges(design: &Design) -> Box<LocalGraphAction> {
        let mut graph_builder = LocalGraphActionBuilder::new();

        let n0 = graph_builder.add_node(JustLogAction::new("node0"));
        let n1 = graph_builder.add_node(JustLogAction::new("node1"));
        let n2 = graph_builder.add_node(JustLogAction::new("node2"));
        let n3 = graph_builder.add_node(JustLogAction::new("node3"));
        let n4 = graph_builder.add_node(JustLogAction::new("node4"));

        graph_builder
            .add_edges(n0, &[n1, n2, n3, n4])
            .add_edges(n1, &[n3])
            .add_edges(n2, &[n3, n4])
            .add_edges(n3, &[n4])
            .build(design)
    }

    fn graph_cube(design: &Design) -> Box<LocalGraphAction> {
        let mut graph_builder = LocalGraphActionBuilder::new();

        let n0 = graph_builder.add_node(JustLogAction::new("node0"));
        let n1 = graph_builder.add_node(JustLogAction::new("node1"));
        let n2 = graph_builder.add_node(JustLogAction::new("node2"));
        let n3 = graph_builder.add_node(JustLogAction::new("node3"));
        let n4 = graph_builder.add_node(JustLogAction::new("node4"));
        let n5 = graph_builder.add_node(JustLogAction::new("node5"));
        let n6 = graph_builder.add_node(JustLogAction::new("node6"));
        let n7 = graph_builder.add_node(JustLogAction::new("node7"));

        graph_builder
            .add_edges(n0, &[n1, n2, n4])
            .add_edges(n1, &[n3, n5])
            .add_edges(n2, &[n3, n6])
            .add_edges(n3, &[n7])
            .add_edges(n4, &[n5, n6])
            .add_edges(n5, &[n7])
            .add_edges(n6, &[n7])
            .build(design)
    }

    fn graph_loop(design: &Design) -> Box<LocalGraphAction> {
        let mut graph_builder = LocalGraphActionBuilder::new();

        let n0 = graph_builder.add_node(JustLogAction::new("node0"));
        let n1 = graph_builder.add_node(JustLogAction::new("node1"));

        graph_builder.add_edges(n0, &[n1]).add_edges(n1, &[n0]).build(design)
    }

    fn graph_self_loop(design: &Design) -> Box<LocalGraphAction> {
        let mut graph_builder = LocalGraphActionBuilder::new();

        let n0 = graph_builder.add_node(JustLogAction::new("node0"));
        let n1 = graph_builder.add_node(JustLogAction::new("node1"));

        graph_builder.add_edges(n0, &[n1]).add_edges(n1, &[n1]).build(design)
    }

    fn graph_not_enough_nodes(design: &Design) -> Box<LocalGraphAction> {
        let mut graph_builder = LocalGraphActionBuilder::new();

        let n0 = graph_builder.add_node(JustLogAction::new("node0"));
        let n1 = 1_usize;

        graph_builder.add_edges(n0, &[n1]).build(design)
    }

    fn graph_invalid_node(design: &Design) -> Box<LocalGraphAction> {
        let mut graph_builder = LocalGraphActionBuilder::new();

        let n0 = graph_builder.add_node(JustLogAction::new("node0"));
        let n1 = graph_builder.add_node(JustLogAction::new("node1"));
        let n2 = 2_usize;

        graph_builder.add_edges(n2, &[n0, n1]).build(design)
    }

    fn graph_invalid_edge(design: &Design) -> Box<LocalGraphAction> {
        let mut graph_builder = LocalGraphActionBuilder::new();

        let n0 = graph_builder.add_node(JustLogAction::new("node0"));
        let n1 = graph_builder.add_node(JustLogAction::new("node1"));
        let n2 = 2_usize;

        graph_builder.add_edges(n0, &[n1, n2]).build(design)
    }

    fn graph_duplicated_edge(design: &Design) -> Box<LocalGraphAction> {
        let mut graph_builder = LocalGraphActionBuilder::new();

        let n0 = graph_builder.add_node(JustLogAction::new("node0"));
        let n1 = graph_builder.add_node(JustLogAction::new("node1"));
        let n2 = graph_builder.add_node(JustLogAction::new("node2"));

        graph_builder.add_edges(n0, &[n1]).add_edges(n1, &[n2, n2]).build(design)
    }

    fn choose_graph(name: &str) -> fn(&Design) -> Box<LocalGraphAction> {
        match name {
            // positive scenarios
            "two_nodes" => Self::graph_two_nodes,
            "no_edges" => Self::graph_no_edges,
            "one_node" => Self::graph_one_node,
            "multiple_edges" => Self::graph_multiple_edges,
            "empty_edges" => Self::graph_empty_edges,
            "cube" => Self::graph_cube,
            // negative scenarios
            "loop" => Self::graph_loop,
            "self_loop" => Self::graph_self_loop,
            "not_enough_nodes" => Self::graph_not_enough_nodes,
            "invalid_edge" => Self::graph_invalid_edge,
            "invalid_node" => Self::graph_invalid_node,
            "duplicated_edge" => Self::graph_duplicated_edge,
            _ => panic!("Unknown graph name: {}", name),
        }
    }
}

struct CorrectGraph;

impl Scenario for CorrectGraph {
    fn name(&self) -> &str {
        "correct_graph"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let logic = TestInput::new(input);

        let builder = Runtime::from_json(input)?;
        let mut rt = builder.build();

        let orch = Orchestration::new()
            .add_design(GraphHandler::create_graph_design(&logic.graph_name).expect("Failed to create design"))
            .design_done();

        let mut program_manager = orch.into_program_manager().expect("Failed to create programs");
        let mut programs = program_manager.get_programs();

        rt.block_on(async move {
            let mut program = programs.pop().expect("Failed to pop program");
            let _ = program.run_n(1).await;
        });
        Ok(())
    }
}

struct InvalidGraph;

impl Scenario for InvalidGraph {
    fn name(&self) -> &str {
        "invalid_graph"
    }

    fn run(&self, input: &str) -> Result<(), String> {
        let logic = TestInput::new(input);

        let _ = Orchestration::new()
            .add_design(GraphHandler::create_graph_design(&logic.graph_name).expect("Failed to create design"))
            .design_done();

        Ok(())
    }
}

pub fn graph_scenario_group() -> Box<dyn ScenarioGroup> {
    Box::new(ScenarioGroupImpl::new(
        "graphs",
        vec![Box::new(CorrectGraph), Box::new(InvalidGraph)],
        vec![],
    ))
}

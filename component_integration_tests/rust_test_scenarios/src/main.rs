mod internals;
mod tests;

use test_scenarios_rust::cli::run_cli_app;
use test_scenarios_rust::test_context::TestContext;

use crate::tests::root_scenario_group;

fn main() -> Result<(), String> {
    let raw_arguments: Vec<String> = std::env::args().collect();

    // Root group.
    let root_group = root_scenario_group();

    // Run.
    let test_context = TestContext::new(root_group);
    run_cli_app(&raw_arguments, &test_context)
}

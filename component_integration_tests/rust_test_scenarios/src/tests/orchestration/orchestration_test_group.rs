use super::orchestration_concurrency::{MultipleConcurrencyTest, NestedConcurrencyTest, SingleConcurrencyTest};
use super::orchestration_sequence::{AwaitSequenceTest, NestedSequenceTest, SingleSequenceTest};
use crate::internals::test_case::{TestGroup, TestGroupImpl};

pub struct OrchestrationTestGroup {
    test_group: TestGroupImpl,
}

impl OrchestrationTestGroup {
    pub fn new() -> Self {
        OrchestrationTestGroup {
            test_group: TestGroupImpl::new("orchestration"),
        }
    }
}

impl TestGroup for OrchestrationTestGroup {
    fn get_test_group_impl(&mut self) -> &mut TestGroupImpl {
        &mut self.test_group
    }

    fn init(&mut self) -> () {
        // Sequence tests
        self.test_group.add_test_case(Box::new(SingleSequenceTest));
        self.test_group.add_test_case(Box::new(NestedSequenceTest));
        self.test_group.add_test_case(Box::new(AwaitSequenceTest));
        // Concurrency tests
        self.test_group.add_test_case(Box::new(SingleConcurrencyTest));
        self.test_group.add_test_case(Box::new(MultipleConcurrencyTest));
        self.test_group.add_test_case(Box::new(NestedConcurrencyTest));
    }
}

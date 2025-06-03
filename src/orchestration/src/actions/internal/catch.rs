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

#[cfg(feature = "runtime-api-mock")]
use async_runtime::testing::mock::*;

#[cfg(not(feature = "runtime-api-mock"))]
use async_runtime::*;

#[allow(dead_code)]
fn example() {
    spawn(async {});
}

#[cfg(test)]
#[cfg(not(loom))]
mod test {

    use super::*;
    use testing_macros::ensure_clear_mock_runtime;

    #[test]
    #[ensure_clear_mock_runtime]
    fn test_catch() {
        example();

        let _x = async_runtime::testing::mock::runtime_instance(|runtime| {
            assert!(runtime.remaining_tasks() > 0);
            runtime.advance_tasks();

            assert_eq!(runtime.remaining_tasks(), 0);
        });
    }
}

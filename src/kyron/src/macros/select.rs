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

/// Awaits on multiple futures concurrently until any one finishes.
///
/// The macro is an .await call on an internal future, and thus needs to be used within async code.
/// It will await by polling all futures in order specified in the macro call until one of the futures
/// is ready and matches its pattern. At that point the future's block will be executed and its result
/// will be the result of the `select!` macro. All blocks need to return the same type.
///
/// The macro accepts one or more cases with the following structure:
///
/// ```text
/// <match pattern> = <future> => <block>
/// ```
///
/// # Return value
///
/// Returns the result of the block associated with the first (as specified in [Fairness](#fairness)) ready and matched future.
///
/// # Examples
///
/// Example of pattern matching:
///
/// ```text
/// select! {
///     var1 = fut1 => {
///         println!("First future finished {}.", var1);
///     }
///     _ = fut2 => {
///         println!("Second future finished. The result was ignored.");
///     }
///     Err(var3) = fut3 => {
///         println!("Third future finished with error {}", var3);
///     }
/// }
/// ```
///
/// Example of `select!` returning a value. Note that all the blocks need to return the same type.
///
/// ```text
/// //
/// let result = select! {
///     var1 = fut1 => {
///         println!("First future finished {}.", var1);
///         var1
///     }
///     _ = fut2 => {
///         println!("Second future finished. The result was ignored.");
///         0
///     }
///     Err(var3) = fut3 => {
///         println!("Third future finished with error {}", var3);
///         var3
///     }
/// };
/// ```
///
/// # Fairness
///
/// Each time the awaited future created by `select!` is polled by the runtime, it will poll each
/// future in the order specified in the macro call. Once a future reports [`Poll::Ready`](core::task::Poll::Ready), it will not be polled again.
/// The macro stops polling all futures once a future reports [`Poll::Ready`](core::task::Poll::Ready) with a result that matches the associated pattern.
///
/// # Panics
///
/// `select!` currently doesn't panic on its own.
#[macro_export]
macro_rules! select {
    // Notes:
    //
    // The @ symbol at the beginning is a common pattern in Rust macros to hide a rule from the user.

    // Entry. This is the part that matches the user input. It starts the recursion by parsing the first case, and passing the remainder.
    ($v:pat = $f:expr => $b:block $($remainder:tt)*) => {
        select!(@ {();} $v = $f => $b $($remainder)*)
    };

    // Recursion. The recursion is necessary to store the index of each case, and to calculate the number of cases.
    //
    // For an example input:
    //
    // select! {
    //     v1 = f1 => { ... }
    //     v2 = f2 => { ... }
    //     v3 = f3 => { ... }
    // }
    //
    // the output of the recursion will be:
    //
    // (_ _ _); () v1 = f1 => { ... }, (_) v2 = f2 => { ... }, (_ _) v3 = f3 => { ... }
    //
    // This output can then be matched with the main logic. The underscore within parenthesis are used for counting, where () is 0, (_) is 1, and so on.
    // The underscores can be matched to a number using a macro. They can also be used to skip values when destructuring a tuple.
    // The previous underscores and the accumulated cases are passed through the recursion within {} brackets.
    (@ {($($prev_underscores:tt)*); $($case_accum:tt)*} $v:pat = $f:expr => $b:block $($remainder:tt)+ ) => {
        select!(@{ ($($prev_underscores)* _); $($case_accum)* ($($prev_underscores)*) $v = $f => $b,} $($remainder)+)
    };

    // Recursion termination.
    (@ {($($prev_underscores:tt)*); $($case_accum:tt)*} $v:pat = $f:expr => $b:block) => {
        select!(@ ($($prev_underscores)* _); $($case_accum)* ($($prev_underscores)*) $v = $f => $b,)
    };

    // Main logic. This is matched at the end of the recursion.
    (@ ($($max_index_underscores:tt)*); $(($($index_underscores:tt)*) $v:pat = $f:expr => $b:block,)+) => {{
        use core::future::Future;
        use core::pin::pin;

        // If a bit at case's index is set to 1, the case is disabled.
        let mut disabled_cases = 0_usize;

        // The pin! macro is necessary to pin futures that don't implement that Unpin trait, for example async functions and closures.
        // The use of the pin! macro is safe because select! awaits on the poll_fn immediately instad of returning the future.
        // If select was to return the future, the wrapping block would need to be async to capture the local variable crated by pin!.
        let mut pins = ($(pin!($f),)+);

        core::future::poll_fn(|context| {
            // Create a block for each case and poll its future.
            // TODO: This will have to be a for loop over all futures if randomization is needed.
            $({
                let case_index = $crate::underscore_count!($($index_underscores)*);
                let case_mask = 1 << case_index;

                if (disabled_cases & case_mask) == 0 {
                    let ($($index_underscores,)* pin, ..) = &mut pins;

                    match pin.as_mut().poll(context) {
                        core::task::Poll::Ready(result) => {
                            #[allow(unused)] // In case $v isn't used within the block
                            #[allow(unreachable_patterns)] // $v could be _
                            match result {
                                $v => {
                                    return core::task::Poll::Ready($b);
                                },
                                _ => {
                                    disabled_cases |= case_mask;
                                }
                            }
                        },
                        core::task::Poll::Pending => {}
                    }
                }
            })+

            core::task::Poll::Pending
        }).await
    }};
}

#[macro_export]
macro_rules! underscore_count {
    () => {
        0_usize
    };
    (_) => {
        1_usize
    };
    (_ _) => {
        2_usize
    };
    (_ _ _) => {
        3_usize
    };
    (_ _ _ _) => {
        4_usize
    };
    (_ _ _ _ _) => {
        5_usize
    };
    (_ _ _ _ _ _) => {
        6_usize
    };
    (_ _ _ _ _ _ _) => {
        7_usize
    };
    (_ _ _ _ _ _ _ _) => {
        8_usize
    };
    (_ _ _ _ _ _ _ _ _) => {
        9_usize
    };
    (_ _ _ _ _ _ _ _ _ _) => {
        10_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _) => {
        11_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _) => {
        12_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _) => {
        13_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        14_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        15_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        16_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        17_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        18_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        19_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        20_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        21_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        22_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        23_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        24_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        25_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        26_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        27_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        28_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        29_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        30_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        31_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        32_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        33_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        34_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        35_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        36_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        37_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        38_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        39_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        40_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        41_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        42_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        43_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        44_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        45_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        46_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        47_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        48_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        49_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        50_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        51_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        52_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        53_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        54_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        55_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        56_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        57_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        58_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        59_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        60_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        61_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        62_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        63_usize
    };
    (_ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _) => {
        64_usize
    };
}

#[cfg(test)]
#[cfg(not(loom))]
mod tests {
    use core::future::Future;
    use core::{future, task};

    #[test]
    fn one_case() {
        async fn test() -> i32 {
            let fut1 = future::ready(111);
            let mut result = 0;

            select! {
                var1 = fut1 => {
                    result = var1;
                }
            }

            result
        }

        let mut context = task::Context::from_waker(task::Waker::noop());
        let mut future = Box::pin(test());
        let result = future.as_mut().poll(&mut context);
        assert_eq!(result, task::Poll::Ready(111));
    }

    #[test]
    fn case_1_first() {
        async fn test() -> i32 {
            let fut1 = future::ready(111);
            let fut2 = future::pending::<i32>();
            let fut3 = future::pending::<i32>();
            let mut result = 0;

            select! {
                var1 = fut1 => {
                    result = var1;
                }
                _ = fut2 => {
                    result = 222;
                }
                _ = fut3 => {
                    result = 333;
                }
            }

            result
        }

        let mut context = task::Context::from_waker(task::Waker::noop());
        let mut future = Box::pin(test());
        let result = future.as_mut().poll(&mut context);
        assert_eq!(result, task::Poll::Ready(111));
    }

    #[test]
    fn case_2_first() {
        async fn test() -> i32 {
            let fut1 = future::pending::<i32>();
            let fut2 = future::ready(222);
            let fut3 = future::pending::<i32>();
            let mut result = 0;

            select! {
                _ = fut1 => {
                    result = 111;
                }
                var2 = fut2 => {
                    result = var2;
                }
                _ = fut3 => {
                    result = 333;
                }
            }

            result
        }

        let mut context = task::Context::from_waker(task::Waker::noop());
        let mut future = Box::pin(test());
        let result = future.as_mut().poll(&mut context);
        assert_eq!(result, task::Poll::Ready(222));
    }

    #[test]
    fn case_3_first() {
        async fn test() -> i32 {
            let fut1 = future::pending::<i32>();
            let fut2 = future::pending::<i32>();
            let fut3 = future::ready(333);
            let mut result = 0;

            select! {
                _ = fut1 => {
                    result = 111;
                }
                _ = fut2 => {
                    result = 222;
                }
                var3 = fut3 => {
                    result = var3;
                }
            }

            result
        }

        let mut context = task::Context::from_waker(task::Waker::noop());
        let mut future = Box::pin(test());
        let result = future.as_mut().poll(&mut context);
        assert_eq!(result, task::Poll::Ready(333));
    }

    #[test]
    fn case_2_and_3_ready() {
        async fn test() -> i32 {
            let fut1 = future::pending::<i32>();
            // Both futures are ready, but since there's no randomization, the 2nd will return first.
            let fut2 = future::ready(222);
            let fut3 = future::ready(333);
            let mut result = 0;

            select! {
                _ = fut1 => {
                    result = 111;
                }
                var2 = fut2 => {
                    result = var2;
                }
                var3 = fut3 => {
                    result = var3;
                }
            }

            result
        }

        let mut context = task::Context::from_waker(task::Waker::noop());
        let mut future = Box::pin(test());
        let result = future.as_mut().poll(&mut context);
        assert_eq!(result, task::Poll::Ready(222));
    }

    #[test]
    fn optional_case_matches() {
        async fn test() -> i32 {
            let fut1 = future::pending::<i32>();
            let fut2: future::Ready<Option<i32>> = future::ready(Some(222));
            let fut3 = future::ready(333);
            let fut4 = future::pending::<i32>();
            let mut result = 0;

            select! {
                _ = fut1 => {
                    result = 111;
                }
                Some(var2) = fut2 => {
                    result = var2;
                }
                var3 = fut3 => {
                    result = var3;
                }
                _ = fut4 => {
                    result = 444;
                }
            }

            result
        }

        let mut context = task::Context::from_waker(task::Waker::noop());
        let mut future = Box::pin(test());
        let result = future.as_mut().poll(&mut context);
        assert_eq!(result, task::Poll::Ready(222));
    }

    #[test]
    fn optional_case_fails_to_match() {
        async fn test() -> i32 {
            let fut1 = future::pending::<i32>();
            let fut2: future::Ready<Option<i32>> = future::ready(None);
            let fut3 = future::ready(333);
            let fut4 = future::pending::<i32>();
            let mut result = 0;

            select! {
                _ = fut1 => {
                    result = 111;
                }
                Some(var2) = fut2 => {
                    result = var2;
                }
                var3 = fut3 => {
                    result = var3;
                }
                _ = fut4 => {
                    result = 444;
                }
            }

            result
        }

        let mut context = task::Context::from_waker(task::Waker::noop());
        let mut future = Box::pin(test());
        let result = future.as_mut().poll(&mut context);
        assert_eq!(result, task::Poll::Ready(333));
    }

    #[test]
    fn all_cases_pending() {
        async fn test() -> i32 {
            let fut1 = future::pending::<i32>();
            let fut2 = future::pending::<i32>();
            let fut3 = future::pending::<i32>();
            let mut result = 0;

            select! {
                _ = fut1 => {
                    result = 111;
                }
                _ = fut2 => {
                    result = 222;
                }
                _ = fut3 => {
                    result = 333;
                }
            }

            result
        }

        let mut context = task::Context::from_waker(task::Waker::noop());
        let mut future = Box::pin(test());

        for _ in 0..10 {
            let result = future.as_mut().poll(&mut context);
            assert_eq!(result, task::Poll::Pending);
        }
    }

    #[test]
    fn select_returns_result() {
        async fn test() -> i32 {
            let fut1 = future::ready(111);
            let fut2 = future::pending::<i32>();
            let fut3 = future::pending::<i32>();

            select! {
                var1 = fut1 => {
                    var1
                }
                _ = fut2 => {
                    222
                }
                _ = fut3 => {
                    333
                }
            }
        }

        let mut context = task::Context::from_waker(task::Waker::noop());
        let mut future = Box::pin(test());
        let result = future.as_mut().poll(&mut context);
        assert_eq!(result, task::Poll::Ready(111));
    }

    #[test]
    fn select_called_with_async_function() {
        async fn test() -> i32 {
            async fn first() -> i32 {
                111
            }

            let fut1 = first();
            let fut2 = future::pending::<i32>();
            let fut3 = future::pending::<i32>();

            select! {
                var1 = fut1 => {
                    var1
                }
                _ = fut2 => {
                    222
                }
                _ = fut3 => {
                    333
                }
            }
        }

        let mut context = task::Context::from_waker(task::Waker::noop());
        let mut future = Box::pin(test());
        let result = future.as_mut().poll(&mut context);
        assert_eq!(result, task::Poll::Ready(111));
    }

    #[test]
    fn select_called_with_async_closure() {
        async fn test() -> i32 {
            let first = async || 111;

            let fut1 = first();
            let fut2 = future::pending::<i32>();
            let fut3 = future::pending::<i32>();

            select! {
                var1 = fut1 => {
                    var1
                }
                _ = fut2 => {
                    222
                }
                _ = fut3 => {
                    333
                }
            }
        }

        let mut context = task::Context::from_waker(task::Waker::noop());
        let mut future = Box::pin(test());
        let result = future.as_mut().poll(&mut context);
        assert_eq!(result, task::Poll::Ready(111));
    }
}

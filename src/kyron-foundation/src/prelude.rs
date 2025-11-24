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

pub use crate::sync::foundation_atomic::*;
pub use iceoryx2_bb_elementary::scope_guard::*;
pub use iceoryx2_bb_lock_free::*;

pub use crate::containers::*;
pub use crate::types::*;
pub use iceoryx2_bb_memory::pool_allocator::*;

#[cfg(not(any(feature = "tracing", feature = "log", feature = "score-log")))]
compile_error!("At least one of features 'tracing', 'log' or 'score-log' must be enabled!");

#[cfg(feature = "tracing")]
pub use tracing::{debug, error, info, span, trace, warn, Level};

#[cfg(feature = "log")]
pub use log::{debug, error, info, trace, warn, Level};

#[cfg(feature = "score-log")]
compile_error!("Not ready yet!");

#[cfg(feature = "log")]
#[macro_export]
macro_rules! tracing_adapter {
    // Case 1: key = ?value, then more
    ($key:ident = ?$val:expr, $($rest:tt)*) => {
        $crate::tracing_adapter!(@accum concat!(stringify!($key), "={:?}, "), $val; $($rest)*);
    };

    // Case 2: ?value, then more
    (?$val:expr, $($rest:tt)*) => {
        $crate::tracing_adapter!(@accum "{:?}, ", $val; $($rest)*);
    };

    // Message only case
    ($msg:literal) => {
        $crate::prelude::trace!($msg);
    };

    // Internal accumulator: finish
    (@accum $fmt:expr, $($vals:expr),*; $msg:literal) => {
        $crate::prelude::trace!(concat!($fmt, $msg), $($vals),*);
    };

    // Continue accumulating: key = ?value
    (@accum $fmt:expr, $($vals:expr),*; $key:ident = ?$val:expr, $($rest:tt)*) => {
        $crate::tracing_adapter!(
            @accum concat!($fmt, stringify!($key), "={:?}, "),
            $($vals,)* $val;
            $($rest)*
        );
    };

    // Continue accumulating: ?value
    (@accum $fmt:expr, $($vals:expr),*; ?$val:expr, $($rest:tt)*) => {
        $crate::tracing_adapter!(
            @accum concat!($fmt, "{:?}, "),
            $($vals,)* $val;
            $($rest)*
        );
    };
}

#[cfg(feature = "tracing")]
#[macro_export]
macro_rules! tracing_adapter {
     ($($t:tt)*) => {
         $crate::prelude::trace!($($t)*);
    };
}

/// Proxy for `trace!` macro, works with `tracing` and `log` features.
/// `tracing` key-value syntax (`name =? value`) is supported for `log` derived crates.
/// NOTE: provided variables cannot be interpolated into the message literal directly.
pub use tracing_adapter;

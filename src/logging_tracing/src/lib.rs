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

pub mod prelude;

#[cfg(feature = "tracing")]
mod tracing;
#[cfg(feature = "tracing")]
type LogTraceLibrary = tracing::TracingLibraryImpl;

#[cfg(feature = "log")]
mod log;
#[cfg(feature = "log")]
type LogTraceLibrary = log::LogLibraryImpl;

#[cfg(feature = "score-log")]
compile_error!("Not yet supported");

#[derive(Debug, Clone, Copy)]
pub enum Level {
    FATAL = 0,
    ERROR = 1,
    WARN = 2,
    INFO = 3,
    DEBUG = 4,
    TRACE = 5,
}

#[derive(Debug, Clone, Copy)]
pub enum LogMode {
    Logging,
    Tracing,
}

#[derive(Debug, Clone, Copy)]
pub enum TraceScope {
    ///
    /// Logs events from App and if `traced` is running also logs system events (if not, no kernel traces will be there)
    ///
    AppScope,

    ///
    /// Logs app event to `traced` and the events need to be dumped by `perfetto` tool. This is useful once you want to trace multiple apps
    ///
    SystemScope,
}

pub struct LogAndTraceBuilder {
    log_level: Level,
    enable_tracing: Option<TraceScope>,
    enable_logging: bool,
}

impl Default for LogAndTraceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl LogAndTraceBuilder {
    pub fn new() -> Self {
        Self {
            log_level: Level::INFO,
            enable_tracing: None,
            enable_logging: false,
        }
    }

    pub fn global_log_level(mut self, level: Level) -> Self {
        self.log_level = level;
        self
    }

    ///
    /// Enables tracing in given mode. Not supported on QNX target now.
    ///
    pub fn enable_tracing(mut self, scope: TraceScope) -> Self {
        self.enable_tracing = Some(scope);
        self
    }

    ///
    /// Enables logging
    ///
    pub fn enable_logging(mut self, enable: bool) -> Self {
        self.enable_logging = enable;
        self
    }

    ///
    /// Builds and starts log and trace backend
    ///
    #[allow(clippy::result_unit_err)]
    pub fn build(self) -> Result<LogTraceLibrary, ()> {
        LogTraceLibrary::try_from(self)
    }
}

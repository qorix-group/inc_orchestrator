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
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::format;
use tracing_subscriber::prelude::*;
use tracing_subscriber::Layer;

use crate::LogAndTraceBuilder;

pub struct TracingLibraryImpl {
    #[cfg(feature = "perfetto")]
    local_tracer_guard: Option<internal_perfetto::WorkerGuard>,
}

impl TryFrom<LogAndTraceBuilder> for TracingLibraryImpl {
    type Error = ();

    fn try_from(builder: LogAndTraceBuilder) -> Result<Self, Self::Error> {
        let registry = tracing_subscriber::Registry::default();

        let mut layers = None;

        let fmt_layer = fmt::layer()
            .with_writer(std::io::stdout)
            .event_format(format::Format::default().with_thread_ids(true))
            .with_span_events(format::FmtSpan::FULL)
            .with_filter(LevelFilter::from_level(builder.log_level.into()));

        if builder.enable_logging {
            layers = Some(fmt_layer.boxed());
        }

        #[cfg(feature = "perfetto")]
        let mut local_tracer_guard = None;

        #[cfg(feature = "perfetto")]
        if let Some(tracing_mode) = self.enable_tracing {
            use internal_perfetto::*;

            match tracing_mode {
                TraceScope::AppScope => {
                    // Initialize tracing
                    let file = File::create(self.get_trace_filename()).expect("Unable to create tracing file");
                    let (nb, guard) = tracing_appender::non_blocking(file);
                    local_tracer_guard = Some(guard);

                    let perfetto_layer = NativeLayer::from_config(local_trace_config(), nb)
                        .with_enable_system(true)
                        .with_enable_in_process(true)
                        .build()
                        .unwrap();

                    layers = Some(match layers {
                        Some(l) => l.and_then(perfetto_layer).boxed(),
                        None => perfetto_layer.boxed(),
                    })
                }
                TraceScope::SystemScope => {
                    let perfetto_layer = layer::SdkLayer::from_config(system_trace_config(), None)
                        .with_enable_system(true)
                        .build()
                        .unwrap();

                    layers = Some(match layers {
                        Some(l) => l.and_then(perfetto_layer).boxed(),
                        None => perfetto_layer.boxed(),
                    })
                }
            }
        }

        if let Some(layer) = layers {
            tracing::subscriber::set_global_default(registry.with(layer)).unwrap();
        }

        internal_iceoryx2::set_log_level(builder.log_level);

        Ok(Self {
            #[cfg(feature = "perfetto")]
            local_tracer_guard,
        })
    }
}

impl From<crate::Level> for tracing::Level {
    fn from(level: crate::Level) -> Self {
        match level {
            crate::Level::FATAL => tracing::Level::ERROR,
            crate::Level::ERROR => tracing::Level::ERROR,
            crate::Level::WARN => tracing::Level::WARN,
            crate::Level::INFO => tracing::Level::INFO,
            crate::Level::DEBUG => tracing::Level::DEBUG,
            crate::Level::TRACE => tracing::Level::TRACE,
        }
    }
}

mod internal_iceoryx2 {
    pub use iceoryx2::prelude::LogLevel;
    pub fn set_log_level(level: crate::Level) {
        let iceoryx_level = match level {
            crate::Level::FATAL => LogLevel::Error,
            crate::Level::ERROR => LogLevel::Error,
            crate::Level::WARN => LogLevel::Warn,
            crate::Level::INFO => LogLevel::Info,
            crate::Level::DEBUG => LogLevel::Debug,
            crate::Level::TRACE => LogLevel::Trace,
        };
        iceoryx2::prelude::set_log_level(iceoryx_level);
    }
}

#[cfg(feature = "perfetto")]
mod internal_perfetto {
    pub use ::core::fmt::Write;
    pub use std::{
        fs::File,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
        {env, fs},
    };

    pub use tracing_appender::non_blocking::WorkerGuard;

    pub use tracing_perfetto_sdk_layer::{self as layer, NativeLayer};
    pub use tracing_perfetto_sdk_schema as schema;
    pub use tracing_perfetto_sdk_schema::trace_config;
    pub const TRACE_OUTDIR_ENV_VAR: &str = "TRACE_OUTDIR";

    fn system_trace_config() -> schema::TraceConfig {
        schema::TraceConfig {
            buffers: vec![trace_config::BufferConfig {
                size_kb: Some(20480),
                ..Default::default()
            }],
            data_sources: vec![trace_config::DataSource {
                config: Some(schema::DataSourceConfig {
                    name: Some("rust_tracing".into()),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn local_trace_config() -> schema::TraceConfig {
        const FTRACE_EVENTS: [&str; 3] = ["sched_switch", "sched_wakeup", "sched_waking"];
        let ftrace = schema::FtraceConfig {
            ftrace_events: FTRACE_EVENTS.iter().map(|&s| s.to_string()).collect(),
            ..Default::default()
        };

        schema::TraceConfig {
            buffers: vec![trace_config::BufferConfig {
                size_kb: Some(20480),
                ..Default::default()
            }],
            data_sources: vec![
                trace_config::DataSource {
                    config: Some(schema::DataSourceConfig {
                        name: Some("rust_tracing".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                trace_config::DataSource {
                    config: Some(schema::DataSourceConfig {
                        name: Some("linux.process_stats".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                trace_config::DataSource {
                    config: Some(schema::DataSourceConfig {
                        name: Some("linux.perf".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                trace_config::DataSource {
                    config: Some(schema::DataSourceConfig {
                        name: Some("linux.ftrace".into()),
                        ftrace_config: Some(ftrace),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    ///
    /// This API is used to create a file name for the tracing file.
    ///
    fn get_trace_filename(&self) -> PathBuf {
        // Get the current process name
        let process_name = env::current_exe()
            .ok()
            .and_then(|path| path.file_name().map(|name| name.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "unknown_process".to_string());

        // Get the current timestamp
        let start = SystemTime::now();
        let duration = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
        // Format timestamp into seconds
        let seconds = duration.as_secs();
        let date_time = self.format_timestamp(seconds);

        // Set the trace output directory if specified by the env var or /tmp otherwise
        let out_dir = match env::var(TRACE_OUTDIR_ENV_VAR) {
            Ok(dir) => PathBuf::from(dir),
            Err(_) => env::temp_dir(),
        };

        // Create the output directory if not yet exist
        if !out_dir.exists() {
            fs::create_dir(&out_dir).unwrap();
        }

        // Generate the filename
        let filename = format!("{}/trace_{}_{}.pftrace", out_dir.display(), process_name, date_time);
        PathBuf::from(filename)
    }

    ///
    /// Formats timestamp as YYYY-MM-DD_HH-MM-SS". This is used for naming the tracing file.
    ///
    fn format_timestamp(seconds: u64) -> String {
        // Break into h/m/s first
        let mut secs = seconds;
        let s = secs % 60;
        secs /= 60;
        let m = secs % 60;
        secs /= 60;
        let h = secs % 24;
        let mut days = secs / 24;

        // Compute year
        let mut year = 1970;
        loop {
            let days_in_year = if is_leap(year) { 366 } else { 365 };
            if days < days_in_year {
                break;
            }
            days -= days_in_year;
            year += 1;
        }

        // Compute month
        const DAYS_IN_MONTH: [[u32; 12]; 2] = [
            // non-leap-year
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
            // leap-year
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
        ];

        let leap = if is_leap(year) { 1 } else { 0 };

        let mut month = 0;
        while days >= DAYS_IN_MONTH[leap][month] as u64 {
            days -= DAYS_IN_MONTH[leap][month] as u64;
            month += 1;
        }

        let day = days + 1;

        format!("{:04}-{:02}-{:02}_{:02}-{:02}-{:02}", year, month + 1, day, h, m, s)
    }

    fn is_leap(year: u64) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }
}

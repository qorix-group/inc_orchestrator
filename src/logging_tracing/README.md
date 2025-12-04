## USAGE

1. In your code , include the APIs from tracing as a prelude;

```rust
use logging_tracing::prelude::*;
```

2. Tracing Initialization

In your main function, initialize tracing by using the below API. This will create a pftrace file in
the /tmp folder with a timestamp which will store the traces.

 2.1. Logging Mode

 For using logging mode where the log levels Debug, Info, Warn and Error are available to use and
 the logging statements are logged to the console. The 2nd parameter is insignificant here, but
 needed as of now.
```rust
    let logger = LogAndTraceBuilder::new()
        .global_log_level(Level::INFO)
        .enable_logging(true)
        .build();
    logger.init_log_trace();
```

 2.2. Application Tracing Mode

 For using tracing mode where the log level Trace is available, which is similar to verbose and
 captures all the other log levels. The application traces are captured and can be visualized in
 Perfetto UI. If you run `traced`, You will also get kernel trace included. If you need tune config,
 please look into `local_trace_config()`
```rust
    let tracer = LogAndTraceBuilder::new()
        .global_log_level(Level::TRACE)
        .enable_tracing(TraceScope::AppScope)
        .enable_logging(true)
        .build();
```

2.3. System Tracing Mode

For using tracing mode where the log level Trace is available, which is similar to verbose and
captures all the other log levels. The system-level traces along with application traces are
captured by using the daemons from Perfetto and can be visualized in Perfetto UI.
```rust
    let tracer = LogAndTraceBuilder::new()
        .global_log_level(Level::TRACE)
        .enable_tracing(TraceScope::SystemScope)
        .build();
```

Steps to dump logs in system mode (refer to [this](#perfetto) for details and `repo_root/scripts`
for small automation ):
- make sure `traced` and `trace_probes` is up and running with config you did or with example config
  in `config/` folder
- before starting your program/programs run `perfetto --txt -c CONFIG -o output_file` to let it dump
  trace buffers.
- run apps
- close `perfetto` once you want to finish dump



4. Create the tracing events using the below API, Pass the required data.

```rust
    trace!("Trace level test");
    debug!("Trace level test");
    info!("{:?}:inside test_function!",thread::current().id());
    warn!("Trace level test");
    error!("Trace level test");
```

## PERFETTO

### Building
When building for perfetto you shall use `--feature perfetto` passed to build command to enable it.
This is only available currently for `cargo` builds.
```

### Prerequisites for Perfetto:
1.  If using ubuntu virtual machine, make sure that the VM runs in hardware virtualization mode and
    not software virtualization. In case it's in software virtualization mode, indicated by green
    turtle in right corner of VM do the following to make changes (when using VirtualBox).
```
    1.1. Launch Regedit Editor as the admin, in the path:
    "Computer\HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\DeviceGuard\"
    Scenarios\***\Enabled – change Enabled to “0” everywhere. NOTE: After Windows update you might
    need to do it again! 1.2. run cmd as admin, and run bcdedit /set hypervisorlaunchtype off 1.3.
    in BIOS change virtualization to off , Security → Virtualization → ensure that “VTd and/or Intel
    Virtualization Technology is disabled.
```
2. Before cloning and building Perfetto , make sure that curl, Clang,protobuf-compiler, g++ ,
   python3 is installed and the path is set in the Env variables.

First you need to obtain perfetto, please use
[Precompiled binaries](https://github.com/google/perfetto/releases)
[Build yourself](https://perfetto.dev/docs/contributing/build-instructions)

To fast start on your local PC use
[https://perfetto.dev/docs/quickstart/linux-tracing#capturing-a-trace](https://perfetto.dev/docs/quickstart/linux-tracing#capturing-a-trace)
On embeeded HW like RPi You need to do same things as in local PC use case. The only thing is that
you can make `traced` and `traced_probes` always run on startup so you dont need to worry about them

## READ THE LOGS/TRACE

1. When used in logging mode , the log statements will be visible on the console where theexecutable
   runs.
2. When used in tracing mode with App Scope, the user can see the trace data using the
   [ perfetto ui](https://ui.perfetto.dev/).
   1. Open the webpage .
   2. Click on the link to "Open trace file"
   3. Select the generate trace file from the folder set by `TRACE_OUTDIR` environment variable (or
      `/tmp` if unset) similar to the naming convention like
      "trace_hello_tracing_2025-04-11_17-46-38.pftrace"
3. When used in trracing mode with System Scope,
   1. YET TO IMPLEMENT.

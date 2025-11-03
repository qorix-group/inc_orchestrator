# Features
* Actions
    * `sequence` - run multiple actions in sequence
    * `concurrency` - run multiple actions in paraller
    * `invoke` - call user functions
    * `catch` - error handling
    * `select` - run multiple actions in first win fashion
    * `sync` - receive notification
    * `trigger` - send notification in process or across process
    * `local_graph` - model dependencies as Direct Acyclic Graph

* Configuration:
    * Full decouple of application logic (defined flow) from it's deployment
        * configure events mapping (local, global, timer)
        * configure in which worker user functions shall run
        * others

* C++ support
    * Ability to call C++ code from Orchestration using `Invoke` action
    * C++ macros that create Rust binding for the user  (no hand writing)
    * Rust macros that creates `FFI` layer for the user (no hand writing)

* OSes
    * Linux support (x86_64 & aarch64)
    * QNX support (aarch64), 7.1 & 8.0

* Testing:
    * Coverage by component tests
    * Coverage by unit tests

* Examples
    * rich pool of examples

# Known issues
* ...

# Planned features
* `mw_com` support

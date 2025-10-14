# Features
* Runtime configurability
    * configurable async workers (regarding their thread settings)
    * configurable dedicated workers (regarding their thread settings)
    * ability to spawn tasks in dedicated workers only using `spawn_on_dedicated`
    * configurable `safety` worker along with `spawn_safety` to ensure reaction handling
* Time module
    * async sleep along with SW Timers infrastructure
* Futures:
    * `select!` macro support
* IO:
    * basic net support (UDP, TCP) with port of OSS hyper http server
* Channels:
    * `spsc` channel
    * `spmc_broadcast` channel

* IPC:
    * initial support for `async` API for IPCs - currently for iceoryx2.


* OSes
    * Linux support (x86_64 & aarch64)
    * QNX support (aarch64), 7.1 & 8.0

* Testing:
    * Coverage by component tests
    * Coverage by unit tests
* Misc
    * manual instantiation of runtime from plain sync functions or async main decorator

* Third party
    * Internal port of http framework [hyper](https://github.com/hyperium/hyper)

# Known issues
* Setting affinity on dedicated_workers does not work in some cases
* Documentation may be not detailed in some places

# Planned features
* Async API for all messaging patterns for `iceoryx2`
* More common building blocks in `Timer` module (timeout, interval, etc)
* Remove maitenance allocations at runtime
* ...

# `inc_orchestrator`

Incubation repo for orchestration

## Setup

### System dependencies

```bash
sudo apt-get update
sudo apt-get install -y curl build-essential protobuf-compiler libclang-dev git python3-dev python-is-python3 python3-venv
```

### Rust installation

[Install Rust using rustup](https://www.rust-lang.org/tools/install)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
```

### Bazel installation

[Install Bazel using Bazelisk](https://bazel.build/install/bazelisk)

```bash
curl --proto '=https' -sSfOL https://github.com/bazelbuild/bazelisk/releases/download/v1.26.0/bazelisk-amd64.deb
dpkg -i bazelisk-amd64.deb
rm bazelisk-amd64.deb
```

Correct Bazel version will be installed on first run, based on `bazelversion` file.

## Build

### Using Cargo

You can either use manually usually `cargo` commands or use `xtask` approach

```bash
cargo xtask - print usage
```

The `xtask` has advantage that it builds using separate build dirs, so when building `test` and `target`, there is no need to rebuild each time.

#### Run some specific

Use regular commands but prefer `cargo xtask`

```bash
cargo xtask run --example basic
```

### Using Bazel

```bash
bazel run //orchestration:basic
```

### Bazel

#### Targets

##### Tests

Each component has defined test target as `component:tests` so You can run them via `bazel`:

```txt
bazel test //PATH_TO_COMPONENT:tests
```

You can also run all tests via:

```txt
bazel test //...
```

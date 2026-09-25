# Contributing to CUV (C-Ultra-Velocity) ⚡

Thank you for your interest in contributing to **CUV**! We are building the next-generation package manager, build driver, and toolchain orchestrator for C and C++ — the `uv` and `bun` for the systems world.

---

## 🚀 Quickstart: Local Development

### Prerequisites
- [Rust toolchain](https://rustup.rs/) (1.75+ recommended)
- A local C/C++ compiler (`clang++`/`clang` or `g++`/`gcc`)

### 1. Clone & Build
```bash
git clone https://github.com/araskova/cuv.git
cd cuv
cargo build
```

### 2. Run Tests
Ensure all unit and integration tests pass:
```bash
cargo test
```

### 3. Install Locally for Testing
To test the binary directly in your terminal:
```bash
cargo install --path .
cuv info
```

---

## 📦 The 3-Minute Contribution: Add a Library to CUV

One of the easiest and highest-impact ways to contribute is adding your favorite C++ libraries to the built-in CUV registry!

Open [`src/package/registry.rs`](src/package/registry.rs) and add your library to `get_known_registry()`:

```rust
m.insert(
    "your_library",
    KnownPackage {
        name: "your_library",
        repo: "owner/repo",
        include_subpath: "include", // Subdirectory containing headers (or "" if root)
        default_tag: "v1.0.0",      // Latest stable tag or version
        description: "High-performance linear algebra library",
        homepage: "https://yourlibrary.org",
        license: "MIT",
    },
);
```

Then test it locally:
```bash
cargo test
./target/debug/cuv add your_library
```

Commit your change and submit a Pull Request! 🎉

---

## 🏛️ Codebase Architecture

CUV is organized into clean, domain-specific modules:

| Directory | Purpose |
| :--- | :--- |
| [`src/commands/`](src/commands/) | CLI command handlers (`init`, `add`, `sync`, `build`, `run`, `test`, `export`, `clean`, `info`) |
| [`src/compiler/`](src/compiler/) | Parallel compilation engine, `-MMD` dependency parsing, and static/shared/executable linkers |
| [`src/core/`](src/core/) | Manifest parser (`cuv.toml`), project metadata, and target configuration |
| [`src/package/`](src/package/) | Package registry, GitHub tarball resolver, and content-addressable cache |
| [`src/test_runner/`](src/test_runner/) | Test discovery (`tests/*.cpp`), project object linking, and test suite execution |
| [`src/toolchain/`](src/toolchain/) | Host compiler probing (`clang++`, `g++`, `clang`, `gcc`, `ar`), version, and triple detection |
| [`src/interop/`](src/interop/) | CMake compatibility layer (`CMakeLists.txt` export and `cuv.cmake` provider) |
| [`src/ui/`](src/ui/) | High-tech terminal animations, spinners, progress bars, and status formatting |

---

## 🛠️ Pull Request Guidelines

1. **Keep it focused**: Each PR should address a single feature, bug fix, or package addition.
2. **Add tests**: If adding new functionality, add a corresponding test in [`tests/`](tests/).
3. **Format & Lint**:
   ```bash
   cargo fmt --check
   cargo clippy --all-targets -- -D warnings
   cargo test
   ```
4. **Descriptive PR Title**: Use conventional commits style:
   - `feat(compiler): add support for precompiled headers`
   - `fix(resolver): handle release tags without leading v`
   - `pkg: add raylib to registry`
   - `docs: update quickstart guide`

---

## 💬 Community & Questions

- **Issues**: Use our [GitHub Issue Templates](https://github.com/araskova/cuv/issues/new/choose) for bugs, feature ideas, and library requests.
- **Discussions**: Share ideas, show off projects built with CUV, and connect with the community.
- **Code of Conduct**: All participants are expected to adhere to our [Code of Conduct](CODE_OF_CONDUCT.md).

Thank you for helping make C and C++ fast, modern, and delightful again! ⚡

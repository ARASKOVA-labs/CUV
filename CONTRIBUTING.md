# Contributing to CUV (C-Ultra-Velocity) ⚡

Thank you for your interest in contributing to **CUV**! We are building the next-generation package manager, build driver, and toolchain orchestrator for C and C++ — the `uv` and `bun` for the systems world.

---

## 🚀 Quickstart: Local Development

### Prerequisites
- [Rust toolchain](https://rustup.rs/) (1.75+ recommended)
- A local C/C++ compiler (`clang++`/`clang` or `g++`/`gcc`)

### 1. Clone & Build
```bash
git clone https://github.com/ARASKOVA-labs/CUV.git
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

## 🛠️ Pull Request Guidelines & CI Protection

All pull requests undergo strict automated validation and security screening:

1. **Automated CI Matrix**: Every PR must pass formatting (`cargo fmt --check`), strict linting (`cargo clippy --all-targets -- -D warnings`), multi-target builds, and all tests across macOS, Linux, and Windows.
2. **Security & Supply Chain Auditing**: Every PR is automatically screened by `cargo-audit` (known CVEs), `gitleaks` (secret/token detection), and dependency review.
3. **Semantic PR Titles**: Titles must adhere to Conventional Commits (e.g. `feat:`, `fix:`, `pkg:`, `docs:`, `chore:`, `perf:`).
4. **Code Quality**: Clean idiomatic Rust, zero clippy warnings, and no unnecessary comments.
5. **Code Owner Sign-off**: Pull requests modifying core compiler, package manager, or CI workflows require maintainer review from `@ARASKOVA-labs`.

---

## 💬 Community & Questions

- **Issues**: Use our [GitHub Issue Templates](https://github.com/ARASKOVA-labs/CUV/issues/new/choose) for bugs, feature ideas, and library requests.
- **Discussions**: Share ideas, show off projects built with CUV, and connect with the community.
- **Code of Conduct**: All participants are expected to adhere to our [Code of Conduct](CODE_OF_CONDUCT.md).

Thank you for helping make C and C++ fast, modern, and delightful again! ⚡

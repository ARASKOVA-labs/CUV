# CUV (C-Ultra-Velocity) ⚡
> Extremely fast, zero-dependency package manager, build driver, and toolchain orchestrator for C and C++.
> An **Araskova Systems** Open Source Initiative. Inspired by `uv` (Python) and `bun` (JavaScript).

```
   ______  __  ___    __
  / ____/ / / / / |  / /
 / /     / / / /| | / / 
/ /___  / /_/ / | |/ /  
\____/  \____/  |___/   
```

[![CI](https://github.com/araskova/cuv/actions/workflows/ci.yml/badge.svg)](https://github.com/araskova/cuv/actions)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org)
[![Platform](https://img.shields.io/badge/Platform-macOS%20%7C%20Linux%20%7C%20Windows-lightgrey.svg)]()
[![Status](https://img.shields.io/badge/Status-Alpha%20Active-emerald.svg)]()

---

## The Problem with Modern C++

In 2026, systems programming still suffers from severe tooling fragmentation:
- **Rust** has `cargo` — unified, fast, standardized dependency resolution and compilation.
- **Python** has `uv` — single-binary Rust toolchain running 10–100x faster than pip.
- **JavaScript** has `bun` — single-binary instant runtime, bundler, and package manager.
- **C and C++** are still stuck with 200-line `CMakeLists.txt` files, slow Python-based Conan recipes, and minutes of rebuild time. Over **47% of developers** in the ISO C++ survey cite dependency management as their #1 frustration.

**CUV** is designed to fix this: **A single, standalone Rust binary that provides an instant, zero-config package manager, build driver, and toolchain orchestrator for C and C++.**

---

## ⚡ Key Highlights

- ⚡ **Blazing Fast**: Cold boot in `< 5ms`, sub-second parallel incremental builds, zero external dependencies (no Python, no CMake, no Ninja).
- 📦 **Header-Only Fast Lane**: Fetch, unpack, and link 50%+ of modern C++ libraries (`fmt`, `nlohmann_json`, `spdlog`, `catch2`, `glm`, `taskflow`) in **`< 100ms`**.
- 🛠️ **Zero-Config Developer Experience**: `cuv init` and `cuv run` compile and launch modern C++20 applications instantly with animated terminal feedback.
- 🎯 **Full Target Kinds**: First-class support for `executable`, `static-lib` (`ar rcs`), and `shared-lib` (`.dylib` / `.so`).
- 🔍 **Instant IDE & LSP Integration**: Automatically emits `compile_commands.json` on every build for zero-config autocomplete and diagnostics with `clangd` in VS Code, Neovim, and CLion.
- 🧪 **Project-Linked Test Runner**: `cuv test` discovers and executes test suites in parallel, automatically linking project object files.
- 🤝 **Non-Hostile CMake Interop**: Use `cuv export cmake` or drop in `cuv.cmake` to resolve dependencies inside legacy enterprise CMake codebases.

---

## 🚀 Installation

### Option 1: Standalone Installer (Fastest & Recommended)
Install CUV directly with a single shell command without needing any external dependencies:

**macOS & Linux:**
```bash
curl -LsSf https://raw.githubusercontent.com/araskova/cuv/master/scripts/install.sh | sh
```

**Windows (PowerShell):**
```powershell
powershell -c "irm https://raw.githubusercontent.com/araskova/cuv/master/scripts/install.ps1 | iex"
```

### Option 2: Homebrew (macOS & Linux)
```bash
brew install araskova/tap/cuv
```

### Option 3: Via Cargo & Crates.io
```bash
# Instant pre-compiled binary via cargo-binstall
cargo binstall cuv

# Or compile from source
cargo install --git https://github.com/araskova/cuv.git
```

Verify your installation:
```bash
cuv info
```

*(For enterprise CI/CD integration and deployment instructions, see [DEPLOYMENT_GUIDE.md](DEPLOYMENT_GUIDE.md) and [PITCH_DECK.md](PITCH_DECK.md).)*

---

## 🏁 60-Second Quickstart

### 1. Initialize a new project
```bash
cuv init my_app --std c++20
cd my_app
```

This creates a clean, minimal project structure:
```
my_app/
├── cuv.toml              # Clean declarative manifest
├── include/              # Public headers
├── src/
│   └── main.cpp          # Modern C++20 entrypoint
├── tests/
│   └── test_basic.cpp    # Unit test suite
└── .gitignore
```

### 2. Build and run in a single command
```bash
cuv run
```
```text
⚡ my_app v0.1.0 (clang++ [arm64-apple-darwin27.0.0])

⚡ Built `my_app` (executable) in 388ms
   ├── Objects: 1 (1 recompiled, 0 cached)
   ├── Profile: debug (c++20)
   └── Output:  target/debug/my_app (57.2 KB)

🚀 `target/debug/my_app`
────────────────────────────────────────────────────
Hello from CUV (C-Ultra-Velocity)!
Engine: High-Performance Modern C++ Toolchain by Araskova
────────────────────────────────────────────────────
✨ Process exited with code 0 in 12ms
```

### 3. Add dependencies at warp speed
```bash
# Add from built-in registry aliases
cuv add fmt
cuv add nlohmann_json

# Or add any GitHub repository directly
cuv add github:gabime/spdlog --version v1.13.0
```

Headers are globally cached in `~/.cuv/cache/` and mounted into `.cuv/include/` in **under 200ms**.

### 4. Run test suites
```bash
cuv test
```
```text
🧪 Running 1 test suite(s)...
  ✔ test_basic (4ms)
────────────────────────────────────────────────────
✨ All test suites passed! 1 passed in 5ms
```

### 5. Build for production
```bash
cuv build --release
```
Compiles with multi-core parallelism, `-O3`, `-DNDEBUG`, and full optimization into `target/release/`.

### 6. Inspect & manage global ABI cache
```bash
cuv cache info
cuv cache clean
```

### 7. Team remote cache (CUV Cloud)
```bash
cuv login
cuv cloud status
```

---

## 📄 The `cuv.toml` Manifest

```toml
[project]
name = "my_service"
version = "0.1.0"
standard = "c++20"
kind = "executable" # "executable", "static-lib", "shared-lib"

[dependencies]
fmt = "10.2.1"
nlohmann_json = "v3.11.3"
spdlog = "v1.13.0"

[target.macos]
frameworks = ["CoreGraphics"]
links = ["sqlite3"]

[target.linux]
links = ["pthread", "dl"]
```

---

## 📊 Comparison Matrix

| Metric | CUV (Araskova) | CMake + vcpkg | Conan 2.0 | XMake |
| :--- | :---: | :---: | :---: | :---: |
| **Startup / Invocation Speed** | **< 5ms (Rust)** | ~2–5s (CMake) | ~1–3s (Python) | ~20ms (C/Lua) |
| **Runtime Dependencies** | **None (Single binary)**| Git, Python, MSBuild | Python 3, Pip | None (Lua embedded) |
| **Manifest Readability** | **`cuv.toml` (Declarative)** | Procedural CMake syntax | `conanfile.py` (Python) | `xmake.lua` (Lua DSL) |
| **Header-Only Install Speed** | **< 100ms** | 1–3 min (port build) | 30s–1 min | 5–15s |
| **IDE LSP Configuration** | **Automatic 0-config** | Manual setup required | Requires CMake presets | Generates compile_commands |
| **CMake Compatibility** | **Drop-in `cuv.cmake`** | Native | CMake generator | Can generate CMake |

---

## 🤝 Contributing & Community

We welcome contributors of all experience levels! Check out our **[Contributing Guide](CONTRIBUTING.md)** to get started.

### 🌟 3-Minute Contribution: Add a Library
Adding a C++ library to the CUV registry takes just 3 lines of Rust in [`src/package/registry.rs`](src/package/registry.rs)! See the [3-Minute Guide](CONTRIBUTING.md#-the-3-minute-contribution-add-a-library-to-cuv).

- **Good First Issues**: Check out issues labeled [`good-first-issue`](https://github.com/araskova/cuv/labels/good-first-issue).
- **Request a Library**: Open a [Library Request](https://github.com/araskova/cuv/issues/new?template=library_request.yml).
- **Business Model & Strategy**: Read [`BUSINESS_MODEL.md`](BUSINESS_MODEL.md) for our commercialization and growth plan.
- **Architectural Specification**: Read [`SPECIFICATION.md`](SPECIFICATION.md).
- **Roadmap & Phases**: Read [`DEVELOPMENT_PLAN.md`](DEVELOPMENT_PLAN.md).

---

## 📜 License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

---
© 2026 **Araskova**. Built for the next era of systems engineering.

# CUV (C-Ultra-Velocity) — Master Product Development Plan
### Next-Generation C/C++ Toolchain Manager & Package Driver
**An Araskova Systems Initiative**

---

## 1. Product Vision & Value Proposition

CUV is a single, zero-dependency, statically linked Rust binary that acts as the modern developer toolchain for C and C++ (the "uv" and "bun" for the C++ world).

### Core Pillars
1. **Zero Runtime Dependencies**: No Python, no CMake, no Ninja, no Perl required.
2. **Sub-Second Feedback Loop**: Instant `< 5ms` startup time, incremental parallel builds, and auto-generated `compile_commands.json` for IDE autocompletion (`clangd`).
3. **The "Header-Only Fast Lane"**: Install and link 50%+ of modern C++ libraries (`nlohmann_json`, `fmt`, `spdlog`, `catch2`, `glm`, `stb`) in `< 100ms`.
4. **Global Content-Addressable ABI Cache**: Zero recompilation of unchanged packages across projects on the same machine.
5. **Non-Hostile CMake Interoperability**: Provide `cuv.cmake` and `cuv export cmake` so legacy enterprise codebases adopt CUV without rewrite friction.

---

## 2. System Architecture & Module Breakdown

```
┌────────────────────────────────────────────────────────────────────────┐
│                        CUV CLI (Single Rust Binary)                    │
│   cuv init │ cuv add │ cuv run │ cuv build │ cuv test │ cuv export    │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
       ┌────────────────────────────┼────────────────────────────┐
       ▼                            ▼                            ▼
┌───────────────┐           ┌───────────────┐           ┌────────────────┐
│  manifest.rs  │           │  resolver.rs  │           │  toolchain.rs  │
│  Parses and   │           │ Resolves git, │           │ Detects Clang, │
│  writes       │           │ tags, registry│           │ GCC, MSVC,     │
│  cuv.toml     │           │ & tarballs    │           │ stdlib & triple│
└───────┬───────┘           └───────┬───────┘           └────────┬───────┘
        │                           │                            │
        └───────────────────────────┼────────────────────────────┘
                                    ▼
       ┌─────────────────────────────────────────────────────────┐
       │                       compiler.rs                       │
       │  Parallel compiler driver, flag synthesiser, include    │
       │  mapper, and compile_commands.json generator            │
       └────────────────────────────┬────────────────────────────┘
                                    │
       ┌────────────────────────────┴────────────────────────────┐
       ▼                                                         ▼
┌───────────────┐                                       ┌────────────────┐
│   cache.rs    │                                       │    cmake.rs    │
│ Content-Addr  │                                       │ CMakeLists.txt │
│ ABI Hashes &  │                                       │ export and     │
│ Hardlinks     │                                       │ cuv.cmake drop │
└───────────────┘                                       └────────────────┘
```

### Module Responsibilities
- [`src/manifest.rs`](./src/manifest.rs): Type-safe TOML deserialization and serialization for `cuv.toml` and legacy `cxx.toml`.
- [`src/toolchain.rs`](./src/toolchain.rs): Auto-probes local host compilers (`clang++`, `g++`), detects version, target triple (`arm64-apple-darwin`, `x86_64-unknown-linux-gnu`), and standard library.
- [`src/cache.rs`](./src/cache.rs): Manages `~/.cuv/cache/` content-addressable storage, hardlink tree linking, and ABI fingerprint computation.
- [`src/resolver.rs`](./src/resolver.rs): High-speed package acquisition (built-in registry, GitHub tags, branches, tarball decompression).
- [`src/compiler.rs`](./src/compiler.rs): Discovers sources, sets up flags (`-O3`, `-std=c++20`, `-I`), generates `compile_commands.json`, and links binaries.
- [`src/test_runner.rs`](./src/test_runner.rs): Discovers and runs test suites in parallel (`tests/*.cpp`, `test/*.cpp`).
- [`src/cmake.rs`](./src/cmake.rs): Generates `cuv.cmake` wrapper and exports clean `CMakeLists.txt` for legacy projects.

---

## 3. Six-Phase Engineering Roadmap

### Phase 1: MVP Core CLI & Parallel Compilation (Status: COMPLETED)
- [x] Rust binary project setup (`Cargo.toml` with `clap`, `tokio`, `serde`, `which`).
- [x] Declarative `cuv.toml` specification and parser (`src/manifest.rs`).
- [x] Host compiler & archiver detection (`clang++`, `g++`, `clang`, `gcc`, `ar`) (`src/toolchain.rs`).
- [x] High-concurrency parallel compilation engine with Tokio worker pool (`src/compiler.rs`).
- [x] Incremental build engine with Makefile-style header dependency tracking (`-MMD -MF .d`).
- [x] Complete target kinds: `executable`, `static-lib` (`ar rcs`), and `shared-lib` (`-dynamiclib`/`-shared`).
- [x] Project-linked test runner (`cuv test`) discovering and executing tests in parallel (`src/test_runner.rs`).
- [x] Automated `compile_commands.json` generation for zero-config LSP autocompletion.

### Phase 2: Dependency Resolution & Header-Only Ingestion (Status: COMPLETED)
- [x] Built-in registry aliases (`fmt`, `nlohmann_json`, `spdlog`, `catch2`, `glm`, `stb`, `cxxopts`, `magic_enum`).
- [x] GitHub tarball resolver with robust tag/branch fallback and GZ unpacker (`src/resolver.rs`).
- [x] Automatic project mount into `.cuv/include/` via hardlinks.
- [x] Zero-recompile header-only installation in `< 200ms`.
- [x] Automatic dependency restoration on `cuv build`, `cuv run`, `cuv test`, plus dedicated `cuv sync` (`cuv install`).
- [x] CMake export bridge (`cuv export cmake`, `cuv export provider`).

### Phase 3: Content-Addressable ABI Cache & Composite Hasher (Status: COMPLETED)
- [x] Implement strict composite ABI fingerprinting:
  $$\text{Hash} = \text{SHA256}(\text{SourceBytes} + \text{LocalHeadersBytes} + \text{CompilerVersion} + \text{TargetTriple} + \text{Standard} + \text{Optimization} + \text{Flags} + \text{Defines})$$
- [x] Implement machine-wide object file caching (`.o` caching in `~/.cuv/cache/obj/<hash>.o`).
- [x] Sub-millisecond zero-copy hardlink mounting with project-local `.d` dependency restoration.
- [x] Global cache management CLI (`cuv cache info`, `cuv cache clean [--obj-only]`, `cuv cache size`).
- [x] Full integration testing in `tests/test_cache.rs` (clean rebuild hits, cross-project sharing, eviction metrics).

### Phase 4: Pre-Compiled Binary Artifact Registry (Status: COMPLETED)
- [x] PackageKind metadata (`HeaderOnly` vs `StaticLibrary`) in `src/package/registry.rs`.
- [x] Target triple normalization: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`.
- [x] Pre-compiled static archive (`.a`) mounting into `.cuv/lib/` and header mounting into `.cuv/include/`.
- [x] Automatic linker detection and `-L.cuv/lib -l<name>` synthesis for executables, shared libraries, and test suites.
- [x] Cryptographic SHA-256 checksum verification on binary artifact ingestion.
- [x] Complete integration test suite in `tests/test_binary_artifacts.rs`.

### Phase 5: C++20 Modules First-Class DAG Engine (Status: COMPLETED)
- [x] Direct scanning of `export module <name>;`, `module <name>;`, and `import <name>;` across `.cppm`, `.ixx`, `.cxxm`, `.ccm`, and `.cpp`.
- [x] Topological DAG dependency sorter generating linear batch stages for parallel module building.
- [x] Cyclic module dependency detection (`A -> B -> A`) returning structured error diagnostics.
- [x] Upstream Clang module compilation pipeline (`--precompile`, `-fprebuilt-module-path`, `-fmodule-file`).
- [x] Complete integration test suite in `tests/test_modules.rs`.

### Phase 6: Enterprise Remote Cache & Cloud Sync (Status: COMPLETED)
- [x] CUV Cloud authentication & credentials engine (`~/.cuv/credentials.toml`, `CUV_API_TOKEN`).
- [x] Remote cache HTTP/S3/R2 client (`src/cloud/client.rs`) with token bearer auth, object retrieval, and push.
- [x] CLI authentication commands: `cuv login`, `cuv logout`, `cuv whoami`.
- [x] Cloud remote cache inspection command: `cuv cloud status` reporting connection, latency, and organization.
- [x] Complete integration test suite in `tests/test_cloud.rs`.

---

## 4. Competitive Differentiation Matrix

| Metric | CUV (Araskova) | CMake + vcpkg | Conan 2.0 | XMake |
| :--- | :---: | :---: | :---: | :---: |
| **Startup / Invocation Speed** | **< 5ms (Rust)** | ~2–5s (CMake) | ~1–3s (Python) | ~20ms (C/Lua) |
| **Runtime Dependencies** | **None (Single binary)**| Git, Python, MSBuild | Python 3, Pip | None (Lua embedded) |
| **Manifest Readability** | **`cuv.toml` (Declarative)** | Procedural CMake syntax | `conanfile.py` (Python) | `xmake.lua` (Lua DSL) |
| **Header-Only Install Speed** | **< 100ms** | 1–3 min (port build) | 30s–1 min | 5–15s |
| **IDE LSP Configuration** | **Automatic 0-config** | Manual setup required | Requires CMake presets | Generates compile_commands |
| **CMake Compatibility** | **Drop-in `cuv.cmake`** | Native | CMake generator | Can generate CMake |

---

## 5. Quality Assurance & Testing Strategy

1. **Unit Testing**:
   - Manifest parsing and edge-case validation (`tests/test_manifest.rs`).
   - ABI hash collision resistance tests (`tests/test_abi_hash.rs`).
2. **Integration Verification**:
   - Hermetic build verification across clean sandbox directories.
   - Cross-platform compiler validation (Apple Clang on macOS, Clang/GCC on Linux).
3. **Dogfooding**:
   - Use CUV to compile and manage native C++ components in sister Araskova projects (e.g. `oculus_native`).

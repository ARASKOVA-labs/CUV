# CUV (C-Ultra-Velocity) — Master Deployment & Distribution Guide

This document outlines the optimal deployment architecture and distribution channels to ensure frictionless global adoption for CUV.

---

## 1. The 4 Golden Distribution Channels

To achieve viral adoption similar to `uv`, `bun`, and `rustup`, CUV provides four distinct distribution tiers:

```
┌────────────────────────────────────────────────────────────────────────┐
│                        USER INSTALLATION TIERS                         │
├────────────────────────┬───────────────────────┬───────────────────────┤
│ Tier 1: One-Line Curl  │ Tier 2: Homebrew      │ Tier 3: Cargo Ecosystem
│ (macOS/Linux/Windows)  │ (macOS & Linux)       │ (Systems Engineers)   │
│ curl ... | sh          │ brew install cuv      │ cargo binstall cuv    │
└────────────────────────┴───────────────────────┴───────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│             GitHub Releases Automated Cross-Compilation Matrix         │
│  • aarch64-apple-darwin        • x86_64-apple-darwin                   │
│  • x86_64-unknown-linux-gnu    • aarch64-unknown-linux-gnu             │
│  • x86_64-pc-windows-msvc      • x86_64-unknown-linux-musl             │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Channel 1: The One-Line Installer (Recommended)

This is the highest-conversion channel for 90%+ of developers. It downloads the pre-built native binary for the user's OS and architecture, verifies it, installs it into `~/.cuv/bin/`, and adds it to their shell profile in under 2 seconds.

### macOS & Linux
```bash
curl -LsSf https://raw.githubusercontent.com/araskova/cuv/master/scripts/install.sh | sh
```
*(Or with custom domain once DNS is mapped: `curl -LsSf https://cuv.araskova.com/install.sh | sh`)*

### Windows (PowerShell)
```powershell
powershell -ExecutionPolicy ByPass -c "irm https://raw.githubusercontent.com/araskova/cuv/master/scripts/install.ps1 | iex"
```

---

## 3. Channel 2: Homebrew (macOS & Linux)

Homebrew is the primary package manager for macOS developers.

### Setting up the Tap
1. Create a public repository on GitHub: `araskova/homebrew-tap`.
2. Add the formula at `Formula/cuv.rb` (provided in this repository).
3. Developers install via:
   ```bash
   brew install araskova/tap/cuv
   ```
4. Once CUV achieves 75+ stars and notable adoption, submit a PR to `homebrew/core` for `brew install cuv` direct availability!

---

## 4. Channel 3: Cargo & Crate Distribution

For Rust developers and systems engineers with `cargo`:

### Instant Pre-Compiled Binary Download (`cargo-binstall`)
```bash
cargo binstall cuv
```

### Build from Source (`crates.io`)
```bash
cargo install cuv
```

---

## 5. Automated CI/CD Release Pipeline

The pipeline is pre-configured in [`.github/workflows/release.yml`](.github/workflows/release.yml).

### How to Trigger a Production Release
1. Update `version = "0.1.0"` in `Cargo.toml`.
2. Commit and tag the release:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
3. GitHub Actions automatically executes the multi-architecture matrix:
   - Builds `--release` binaries with optimizations and LTO.
   - Packages them into `.tar.gz` and `.zip` archives.
   - Computes SHA256 checksums.
   - Creates a GitHub Release with downloadable binary assets:
     - `cuv-aarch64-apple-darwin.tar.gz`
     - `cuv-x86_64-apple-darwin.tar.gz`
     - `cuv-x86_64-unknown-linux-gnu.tar.gz`
     - `cuv-aarch64-unknown-linux-gnu.tar.gz`
     - `cuv-x86_64-pc-windows-msvc.zip`
     - `SHA256SUMS.txt`

---

## 6. GitHub Actions Action for Enterprise CI/CD

To allow teams to use CUV in their own GitHub Actions workflows (e.g. `uses: araskova/setup-cuv@v1`):

Create an `action.yml` in `araskova/setup-cuv`:
```yaml
name: 'Setup CUV'
description: 'Install and configure CUV (C-Ultra-Velocity)'
inputs:
  version:
    description: 'Version of CUV to install'
    default: 'latest'
runs:
  using: 'composite'
  steps:
    - shell: bash
      run: |
        curl -LsSf https://raw.githubusercontent.com/araskova/cuv/master/scripts/install.sh | sh
        echo "$HOME/.cuv/bin" >> $GITHUB_PATH
```

---

## 7. Publishing to Crates.io

When ready to publish to crates.io:
1. Log in with your crates.io API token:
   ```bash
   cargo login
   ```
2. Verify package contents:
   ```bash
   cargo package --list
   ```
3. Publish:
   ```bash
   cargo publish
   ```

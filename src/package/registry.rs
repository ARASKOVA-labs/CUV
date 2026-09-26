use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageKind {
    HeaderOnly,
    StaticLibrary { lib_name: &'static str },
}

#[derive(Debug, Clone)]
pub struct BinaryArtifact {
    pub target_triple: &'static str,
    pub url: &'static str,
    pub sha256: &'static str,
    pub lib_filename: &'static str,
}

#[derive(Debug, Clone)]
pub struct KnownPackage {
    pub name: &'static str,
    pub repo: &'static str,
    pub include_subpath: &'static str,
    pub default_tag: &'static str,
    pub description: &'static str,
    pub homepage: &'static str,
    pub license: &'static str,
    pub kind: PackageKind,
    pub artifacts: &'static [BinaryArtifact],
}

pub fn normalize_target_triple(triple: &str) -> &'static str {
    let lower = triple.to_lowercase();
    if lower.contains("darwin") || lower.contains("apple") || lower.contains("macos") {
        if lower.contains("arm64") || lower.contains("aarch64") {
            "aarch64-apple-darwin"
        } else {
            "x86_64-apple-darwin"
        }
    } else if lower.contains("linux") {
        if lower.contains("arm64") || lower.contains("aarch64") {
            "aarch64-unknown-linux-gnu"
        } else {
            "x86_64-unknown-linux-gnu"
        }
    } else if lower.contains("windows") {
        "x86_64-pc-windows-msvc"
    } else {
        "unknown"
    }
}

pub fn get_known_registry() -> HashMap<&'static str, KnownPackage> {
    let mut m = HashMap::new();

    m.insert(
        "fmt",
        KnownPackage {
            name: "fmt",
            repo: "fmtlib/fmt",
            include_subpath: "include",
            default_tag: "10.2.1",
            description: "Modern formatting library for C++",
            homepage: "https://fmt.dev",
            license: "MIT",
            kind: PackageKind::HeaderOnly,
            artifacts: &[],
        },
    );

    m.insert(
        "nlohmann_json",
        KnownPackage {
            name: "nlohmann_json",
            repo: "nlohmann/json",
            include_subpath: "include",
            default_tag: "v3.11.3",
            description: "JSON for Modern C++",
            homepage: "https://json.nlohmann.me",
            license: "MIT",
            kind: PackageKind::HeaderOnly,
            artifacts: &[],
        },
    );

    m.insert(
        "json",
        KnownPackage {
            name: "json",
            repo: "nlohmann/json",
            include_subpath: "include",
            default_tag: "v3.11.3",
            description: "JSON for Modern C++ (alias for nlohmann_json)",
            homepage: "https://json.nlohmann.me",
            license: "MIT",
            kind: PackageKind::HeaderOnly,
            artifacts: &[],
        },
    );

    m.insert(
        "spdlog",
        KnownPackage {
            name: "spdlog",
            repo: "gabime/spdlog",
            include_subpath: "include",
            default_tag: "v1.13.0",
            description: "Fast C++ logging library",
            homepage: "https://github.com/gabime/spdlog",
            license: "MIT",
            kind: PackageKind::HeaderOnly,
            artifacts: &[],
        },
    );

    m.insert(
        "catch2",
        KnownPackage {
            name: "catch2",
            repo: "catchorg/Catch2",
            include_subpath: "src/catch2",
            default_tag: "v3.5.3",
            description: "Modern, C++-native test framework",
            homepage: "https://github.com/catchorg/Catch2",
            license: "BSL-1.0",
            kind: PackageKind::HeaderOnly,
            artifacts: &[],
        },
    );

    m.insert(
        "doctest",
        KnownPackage {
            name: "doctest",
            repo: "doctest/doctest",
            include_subpath: "doctest",
            default_tag: "v2.4.11",
            description: "The fastest feature-rich C++ testing framework",
            homepage: "https://github.com/doctest/doctest",
            license: "MIT",
            kind: PackageKind::HeaderOnly,
            artifacts: &[],
        },
    );

    m.insert(
        "cxxopts",
        KnownPackage {
            name: "cxxopts",
            repo: "jarro2783/cxxopts",
            include_subpath: "include",
            default_tag: "v3.2.0",
            description: "Lightweight C++ command line option parser",
            homepage: "https://github.com/jarro2783/cxxopts",
            license: "MIT",
            kind: PackageKind::HeaderOnly,
            artifacts: &[],
        },
    );

    m.insert(
        "magic_enum",
        KnownPackage {
            name: "magic_enum",
            repo: "Neargye/magic_enum",
            include_subpath: "include",
            default_tag: "v0.9.5",
            description: "Static reflection for enums in C++17/20",
            homepage: "https://github.com/Neargye/magic_enum",
            license: "MIT",
            kind: PackageKind::HeaderOnly,
            artifacts: &[],
        },
    );

    m.insert(
        "stb",
        KnownPackage {
            name: "stb",
            repo: "nothings/stb",
            include_subpath: "",
            default_tag: "master",
            description: "Single-file public domain libraries for C/C++",
            homepage: "https://github.com/nothings/stb",
            license: "MIT OR Public Domain",
            kind: PackageKind::HeaderOnly,
            artifacts: &[],
        },
    );

    m.insert(
        "glm",
        KnownPackage {
            name: "glm",
            repo: "g-truc/glm",
            include_subpath: "glm",
            default_tag: "1.0.1",
            description: "OpenGL Mathematics (GLM) header-only library",
            homepage: "https://github.com/g-truc/glm",
            license: "Happy Bunny OR MIT",
            kind: PackageKind::HeaderOnly,
            artifacts: &[],
        },
    );

    m.insert(
        "expected",
        KnownPackage {
            name: "expected",
            repo: "TartanLlama/expected",
            include_subpath: "include",
            default_tag: "v1.1.0",
            description: "C++11/14/17 std::expected implementation",
            homepage: "https://github.com/TartanLlama/expected",
            license: "CC0-1.0",
            kind: PackageKind::HeaderOnly,
            artifacts: &[],
        },
    );

    m.insert(
        "taskflow",
        KnownPackage {
            name: "taskflow",
            repo: "taskflow/taskflow",
            include_subpath: "taskflow",
            default_tag: "v3.7.0",
            description: "General-purpose parallel and heterogeneous task programming in C++",
            homepage: "https://taskflow.github.io",
            license: "MIT",
            kind: PackageKind::HeaderOnly,
            artifacts: &[],
        },
    );

    m.insert(
        "sqlite3",
        KnownPackage {
            name: "sqlite3",
            repo: "sqlite/sqlite",
            include_subpath: "",
            default_tag: "version-3.45.1",
            description: "Self-contained, serverless SQL database engine",
            homepage: "https://www.sqlite.org",
            license: "Public Domain",
            kind: PackageKind::StaticLibrary { lib_name: "sqlite3" },
            artifacts: &[
                BinaryArtifact {
                    target_triple: "aarch64-apple-darwin",
                    url: "https://github.com/araskova/cuv-binaries/releases/download/v0.1.0/sqlite3-3.45.1-aarch64-apple-darwin.tar.gz",
                    sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                    lib_filename: "libsqlite3.a",
                },
                BinaryArtifact {
                    target_triple: "x86_64-unknown-linux-gnu",
                    url: "https://github.com/araskova/cuv-binaries/releases/download/v0.1.0/sqlite3-3.45.1-x86_64-unknown-linux-gnu.tar.gz",
                    sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                    lib_filename: "libsqlite3.a",
                },
            ],
        },
    );

    m.insert(
        "zlib",
        KnownPackage {
            name: "zlib",
            repo: "madler/zlib",
            include_subpath: "",
            default_tag: "v1.3.1",
            description: "Massively spiffy yet delicately unobtrusive compression library",
            homepage: "https://zlib.net",
            license: "Zlib",
            kind: PackageKind::StaticLibrary { lib_name: "z" },
            artifacts: &[
                BinaryArtifact {
                    target_triple: "aarch64-apple-darwin",
                    url: "https://github.com/araskova/cuv-binaries/releases/download/v0.1.0/zlib-1.3.1-aarch64-apple-darwin.tar.gz",
                    sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                    lib_filename: "libz.a",
                },
            ],
        },
    );

    m.insert(
        "raylib",
        KnownPackage {
            name: "raylib",
            repo: "raysan5/raylib",
            include_subpath: "src",
            default_tag: "5.0",
            description: "A simple and easy-to-use library to enjoy videogames programming",
            homepage: "https://www.raylib.com",
            license: "Zlib",
            kind: PackageKind::StaticLibrary { lib_name: "raylib" },
            artifacts: &[
                BinaryArtifact {
                    target_triple: "aarch64-apple-darwin",
                    url: "https://github.com/araskova/cuv-binaries/releases/download/v0.1.0/raylib-5.0-aarch64-apple-darwin.tar.gz",
                    sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                    lib_filename: "libraylib.a",
                },
            ],
        },
    );

    m
}

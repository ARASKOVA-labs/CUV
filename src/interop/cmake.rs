use crate::core::manifest::CuvManifest;
use anyhow::Result;
use std::path::Path;

pub fn export_cmake_lists(manifest: &CuvManifest, output_path: &Path) -> Result<()> {
    let mut cmake = String::new();
    cmake.push_str("# Generated automatically by CUV (C-Ultra-Velocity)\n");
    cmake.push_str("cmake_minimum_required(VERSION 3.20)\n");
    cmake.push_str(&format!("project({} VERSION {})\n\n", manifest.project.name, manifest.project.version));

    let standard_num = manifest.project.standard.trim_start_matches("c++");
    cmake.push_str(&format!("set(CMAKE_CXX_STANDARD {})\n", standard_num));
    cmake.push_str("set(CMAKE_CXX_STANDARD_REQUIRED ON)\n\n");

    cmake.push_str("file(GLOB_RECURSE SOURCES \"src/*.cpp\" \"src/*.cc\" \"src/*.cxx\" \"src/*.c\")\n\n");

    if manifest.project.kind == "static-lib" {
        cmake.push_str(&format!("add_library({} STATIC ${{SOURCES}})\n", manifest.project.name));
    } else if manifest.project.kind == "shared-lib" {
        cmake.push_str(&format!("add_library({} SHARED ${{SOURCES}})\n", manifest.project.name));
    } else {
        cmake.push_str(&format!("add_executable({} ${{SOURCES}})\n", manifest.project.name));
    }

    cmake.push_str(&format!("target_include_directories({} PUBLIC ${{CMAKE_CURRENT_SOURCE_DIR}}/include ${{CMAKE_CURRENT_SOURCE_DIR}}/.cuv/include)\n\n", manifest.project.name));

    if cfg!(target_os = "macos") {
        if let Some(target_cfg) = manifest.target.get("macos") {
            for fw in &target_cfg.frameworks {
                cmake.push_str(&format!("find_library({}_FW {} REQUIRED)\n", fw.to_uppercase(), fw));
                cmake.push_str(&format!("target_link_libraries({} PUBLIC ${{{}_FW}})\n", manifest.project.name, fw.to_uppercase()));
            }
            for l in &target_cfg.links {
                cmake.push_str(&format!("target_link_libraries({} PUBLIC {})\n", manifest.project.name, l));
            }
        }
    }

    std::fs::write(output_path, cmake)?;
    Ok(())
}

pub fn generate_cmake_provider(output_path: &Path) -> Result<()> {
    let provider = r#"# CUV (C-Ultra-Velocity) CMake Dependency Provider
# Allows any existing CMakeLists.txt to consume dependencies resolved by CUV

macro(cuv_import)
    foreach(PKG ${ARGN})
        message(STATUS "[CUV] Resolving dependency '${PKG}' via CUV CLI...")
        execute_process(
            COMMAND cuv add ${PKG}
            WORKING_DIRECTORY ${CMAKE_CURRENT_SOURCE_DIR}
            RESULT_VARIABLE CUV_RES
        )
        if(NOT CUV_RES EQUAL 0)
            message(WARNING "[CUV] Failed to resolve '${PKG}' via cuv CLI.")
        else()
            include_directories(SYSTEM ${CMAKE_CURRENT_SOURCE_DIR}/.cuv/include)
        endif()
    endforeach()
endmacro()
"#;
    std::fs::write(output_path, provider)?;
    Ok(())
}

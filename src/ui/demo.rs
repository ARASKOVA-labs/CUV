use crate::ui::animation::{
    create_progress_bar, create_spinner, print_build_summary, print_test_suite_result,
    print_test_summary,
};
use crate::ui::banner::{play_animated_shimmer, render_card};
use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;
use std::time::Duration;

pub async fn run_demo() -> Result<()> {
    play_animated_shimmer().await;

    let toolchain_info = vec![
        (
            "Engine",
            "CUV Ultra-Velocity Compiler Driver".bold().to_string(),
        ),
        (
            "Compiler",
            "clang++ 21.0.0 (Apple Silicon / aarch64)"
                .bright_cyan()
                .to_string(),
        ),
        (
            "C++ Standard",
            "C++20 Modules & Concepts enabled".green().to_string(),
        ),
        (
            "Cache Mode",
            "Global Machine-Wide ABI Store (~/.cuv/cache)"
                .dimmed()
                .to_string(),
        ),
        (
            "Parallelism",
            format!(
                "{} worker threads active",
                std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(8)
            )
            .yellow()
            .to_string(),
        ),
    ];
    render_card("SYSTEM TOOLCHAIN HUD", &toolchain_info);
    println!();

    let spinner = create_spinner("Scanning C++20 module dependency graph...");
    tokio::time::sleep(Duration::from_millis(300)).await;
    spinner.finish_with_message(format!(
        "{} Resolved 14 module DAG nodes in 1.4ms (zero cycles)",
        "✔".green().bold()
    ));

    println!();

    println!(
        "{} {} v0.1.0 (clang++ [aarch64-apple-darwin])",
        "⚡".yellow(),
        "nebula_engine".bold()
    );

    let total_units = 16;
    let pb = create_progress_bar(total_units, "Compiling");

    let sample_sources = [
        "src/core/simd_math.cpp [⚡cache hit]",
        "src/render/vulkan_pipeline.cpp",
        "src/ecs/world_scheduler.cpp [⚡cache hit]",
        "src/audio/spatial_dsp.cpp",
        "src/physics/collision_bvh.cpp [⚡cache hit]",
        "src/net/quic_transport.cpp",
        "src/scene/camera_controller.cpp [⚡cache hit]",
        "src/render/mesh_opt.cpp",
    ];

    for (i, src) in sample_sources.iter().enumerate() {
        pb.set_message(src.to_string());
        pb.set_position((i + 1) as u64 * 2);
        tokio::time::sleep(Duration::from_millis(90)).await;
    }

    pb.finish_and_clear();

    print_build_summary(
        "nebula_engine",
        "executable",
        "c++20",
        "release",
        &PathBuf::from("target/release/nebula_engine"),
        16,
        6,
        10,
        Duration::from_millis(342),
    );

    println!("\n{} Running engine test suites...", "🧪".magenta().bold());
    tokio::time::sleep(Duration::from_millis(150)).await;
    print_test_suite_result(
        "test_simd_vector_ops",
        true,
        Duration::from_millis(12),
        None,
    );
    tokio::time::sleep(Duration::from_millis(120)).await;
    print_test_suite_result(
        "test_lockless_memory_arena",
        true,
        Duration::from_millis(8),
        None,
    );
    tokio::time::sleep(Duration::from_millis(100)).await;
    print_test_suite_result(
        "test_cxx20_coroutines_scheduler",
        true,
        Duration::from_millis(15),
        None,
    );
    print_test_summary(3, 0, Duration::from_millis(35));

    let cloud_info = vec![
        (
            "Remote Cache",
            "https://cache.cuv.dev".bright_cyan().to_string(),
        ),
        (
            "Status",
            "🟢 Connected (Roundtrip: 14ms)".green().bold().to_string(),
        ),
        (
            "Organization",
            "Araskova Enterprise Demo".bold().to_string(),
        ),
        (
            "CI Acceleration",
            "92.4% build time reduction active"
                .bright_yellow()
                .to_string(),
        ),
    ];
    render_card("CUV CLOUD ENTERPRISE", &cloud_info);

    println!(
        "\n{} {} Experience the future of C/C++ systems engineering.\n",
        "✨".bright_yellow(),
        "CUV Demo Complete!".green().bold()
    );

    Ok(())
}

pub mod add;
pub mod build;
pub mod cache;
pub mod clean;
pub mod cloud;
pub mod export;
pub mod info;
pub mod init;
pub mod run;
pub mod sync;
pub mod test;

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand, Debug, Clone)]
pub enum CloudAction {
    #[command(about = "Check remote cache connectivity and latency")]
    Status,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Initialize a new modern C++ project")]
    Init {
        #[arg(default_value = ".", help = "Project directory or name")]
        name: String,
        #[arg(
            long,
            default_value = "c++20",
            help = "C++ standard to use (c++17, c++20, c++23)"
        )]
        std: String,
        #[arg(long, help = "Initialize as a library instead of executable")]
        lib: bool,
    },
    #[command(about = "Add a dependency (e.g. fmt, nlohmann_json, github:owner/repo)")]
    Add {
        #[arg(
            help = "Package name, alias, or GitHub repo (e.g., \"fmt\", \"github:nlohmann/json\")"
        )]
        package: String,
        #[arg(short, long, help = "Specific version or tag")]
        version: Option<String>,
    },
    #[command(about = "Compile the C/C++ project")]
    Build {
        #[arg(
            short,
            long,
            help = "Build with optimizations in release mode (-O3, -DNDEBUG)"
        )]
        release: bool,
        #[arg(short, long, help = "Verbose compiler commands")]
        verbose: bool,
    },
    #[command(about = "Build and run the project executable")]
    Run {
        #[arg(short, long, help = "Run in release mode")]
        release: bool,
        #[arg(short, long, help = "Verbose output")]
        verbose: bool,
        #[arg(
            last = true,
            help = "Arguments passed directly to the compiled executable"
        )]
        args: Vec<String>,
    },
    #[command(about = "Discover and run test suites in tests/ or test/")]
    Test,
    #[command(about = "Export project configurations (e.g. CMakeLists.txt or cuv.cmake)")]
    Export {
        #[arg(
            default_value = "cmake",
            help = "Format to export: \"cmake\" or \"provider\""
        )]
        format: String,
    },
    #[command(
        alias = "install",
        about = "Sync and restore all dependencies declared in cuv.toml"
    )]
    Sync,
    #[command(about = "Remove build artifacts (target/)")]
    Clean,
    #[command(about = "Inspect or clear the machine-wide global cache")]
    Cache {
        #[command(subcommand)]
        action: cache::CacheAction,
    },
    #[command(about = "Authenticate with CUV Cloud team cache")]
    Login {
        #[arg(short, long, help = "API token for authentication")]
        token: Option<String>,
        #[arg(short, long, help = "Organization name")]
        org: Option<String>,
    },
    #[command(about = "Clear CUV Cloud authentication credentials")]
    Logout,
    #[command(about = "Show current CUV Cloud authentication identity")]
    Whoami,
    #[command(about = "Inspect CUV Cloud remote cache status and latency")]
    Cloud {
        #[command(subcommand)]
        action: Option<CloudAction>,
    },
    #[command(about = "Display detected C++ toolchain and environment info")]
    Info,
    #[command(about = "Play interactive live visual demo of CUV animations and velocity")]
    Demo,
    #[command(about = "Display the CUV stylized ASCII banner")]
    Banner,
}

pub async fn dispatch(command: Commands) -> Result<()> {
    match command {
        Commands::Init { name, std, lib } => init::handle_init(&name, &std, lib),
        Commands::Add { package, version } => add::handle_add(&package, version).await,
        Commands::Sync => sync::handle_sync().await,
        Commands::Build { release, verbose } => build::handle_build(release, verbose).await,
        Commands::Run {
            release,
            verbose,
            args,
        } => run::handle_run(release, verbose, args).await,
        Commands::Test => test::handle_test().await,
        Commands::Export { format } => export::handle_export(&format),
        Commands::Clean => clean::handle_clean(),
        Commands::Cache { action } => cache::handle_cache(action),
        Commands::Login { token, org } => cloud::handle_login(token, org).await,
        Commands::Logout => cloud::handle_logout(),
        Commands::Whoami => cloud::handle_whoami(),
        Commands::Cloud { action: _ } => cloud::handle_cloud_status().await,
        Commands::Info => info::handle_info(),
        Commands::Demo => crate::ui::demo::run_demo().await,
        Commands::Banner => {
            crate::ui::banner::print_static_banner();
            Ok(())
        }
    }
}

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
    /// Check remote cache connectivity and latency
    Status,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new modern C++ project
    Init {
        /// Project directory or name
        #[arg(default_value = ".")]
        name: String,
        /// C++ standard to use (c++17, c++20, c++23)
        #[arg(long, default_value = "c++20")]
        std: String,
        /// Initialize as a library instead of executable
        #[arg(long)]
        lib: bool,
    },
    /// Add a dependency (e.g. fmt, nlohmann_json, github:owner/repo)
    Add {
        /// Package name, alias, or GitHub repo (e.g., "fmt", "github:nlohmann/json")
        package: String,
        /// Specific version or tag
        #[arg(short, long)]
        version: Option<String>,
    },
    /// Compile the C/C++ project
    Build {
        /// Build with optimizations in release mode (-O3, -DNDEBUG)
        #[arg(short, long)]
        release: bool,
        /// Verbose compiler commands
        #[arg(short, long)]
        verbose: bool,
    },
    /// Build and run the project executable
    Run {
        /// Run in release mode
        #[arg(short, long)]
        release: bool,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
        /// Arguments passed directly to the compiled executable
        #[arg(last = true)]
        args: Vec<String>,
    },
    /// Discover and run test suites in tests/ or test/
    Test,
    /// Export project configurations (e.g. CMakeLists.txt or cuv.cmake)
    Export {
        /// Format to export: "cmake" or "provider"
        #[arg(default_value = "cmake")]
        format: String,
    },
    /// Sync and restore all dependencies declared in cuv.toml
    #[command(alias = "install")]
    Sync,
    /// Remove build artifacts (target/)
    Clean,
    /// Inspect or clear the machine-wide global cache
    Cache {
        #[command(subcommand)]
        action: cache::CacheAction,
    },
    /// Authenticate with CUV Cloud team cache
    Login {
        /// API token for authentication
        #[arg(short, long)]
        token: Option<String>,
        /// Organization name
        #[arg(short, long)]
        org: Option<String>,
    },
    /// Clear CUV Cloud authentication credentials
    Logout,
    /// Show current CUV Cloud authentication identity
    Whoami,
    /// Inspect CUV Cloud remote cache status and latency
    Cloud {
        #[command(subcommand)]
        action: Option<CloudAction>,
    },
    /// Display detected C++ toolchain and environment info
    Info,
    /// Play interactive live visual demo of CUV animations and velocity
    Demo,
    /// Display the CUV stylized ASCII banner
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

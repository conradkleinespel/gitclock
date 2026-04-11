mod config;
mod git;
mod spawn_async;
mod timeslot;
mod commands {
    pub mod commit;
    pub mod config;
    pub mod info;
    pub mod pre_commit_hook;
    pub mod pre_push_hook;
    pub mod pre_rebase_hook;
    pub mod push;
    pub mod rebase;
    pub mod rewrite_history;
    pub mod timeslot;
}

use crate::commands::config::ConfigOptions;
use crate::commands::timeslot::TimeslotOptions;
use crate::config::Config;
use chrono::Utc;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gitclock")]
#[command(about = "A CLI to schedule Git commits", version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// View the location of your config file
    Info,
    /// Set configuration options
    Config {
        /// Open the configuration file in your default editor
        #[arg(long, conflicts_with_all = &["timezone", "allow_push_outside_timeslot"])]
        edit: bool,
        /// Set a specific, fixed, timezone to prevent leaking your system timezone
        #[arg(long)]
        timezone: Option<String>,
        /// Allow push command outside timeslots, may trigger CI runs
        #[arg(long)]
        allow_push_outside_timeslot: Option<bool>,
    },
    /// Manage timeslots in which to commit
    Timeslot {
        /// Add a timeslot, defined by --days, --start and --end
        #[arg(short, long)]
        add: bool,
        /// Days this timeslot applies to, eg 1-5 for Monday through Friday or 6-7 for Saturday and Sunday
        #[arg(long)]
        days: Option<String>,
        /// Start time, eg 0900 for 9am or 1730 for 5:30pm
        #[arg(long)]
        start: Option<String>,
        /// End time, eg 0900 for 9am or 1730 for 5:30pm
        #[arg(long)]
        end: Option<String>,
        /// List timeslots
        #[arg(short, long)]
        list: bool,
    },
    /// Run git commit with modified times
    Commit {
        /// Arguments to pass to git commit
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run git push ensuring no commits are in the future
    Push {
        /// Arguments to pass to git push
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run git rebase with the pre-commit hook
    Rebase {
        /// Arguments to pass to git rebase
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Rewrite git history with dates within timeslots
    RewriteHistory,
    /// Prevents mistakenly committing outside timeslots
    PreCommitHook,
    /// Prevents mistakenly pushing outside timeslots
    PrePushHook,
    /// Prevents mistakenly rebasing outside timeslots
    PreRebaseHook,
}

fn main() {
    let mut config = match Config::create_from_conf() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    if let Err(e) = config.check_config() {
        eprintln!("Configuration error: {}", e);
        println!();
        println!("To fix your configuration, edit:");
        match config.get_file_path() {
            Ok(path) => println!("  {}", path.display()),
            Err(_) => println!("  (Could not determine config path)"),
        }
        std::process::exit(2);
    }

    let cli = Cli::parse();

    let now = Utc::now();
    let exit_code = match cli.command {
        Commands::Info => commands::info::run_info_command(&config),
        Commands::Config {
            edit,
            timezone,
            allow_push_outside_timeslot,
        } => commands::config::run_config_command(
            &mut config,
            ConfigOptions {
                edit,
                allow_push_outside_timeslot,
                timezone,
            },
        ),
        Commands::Timeslot {
            add,
            days,
            start,
            end,
            list,
        } => commands::timeslot::run_timeslot_command(
            TimeslotOptions {
                add,
                list,
                days,
                start,
                end,
            },
            &mut config,
        ),
        Commands::Commit { args } => commands::commit::run_commit_command(now, &args, &config),
        Commands::Push { args } => commands::push::run_push_command(now, &args, &config),
        Commands::Rebase { args } => commands::rebase::run_rebase_command(now, &args, &config),
        Commands::RewriteHistory => {
            commands::rewrite_history::run_rewrite_history_command(now, &config)
        }
        Commands::PreCommitHook => {
            commands::pre_commit_hook::run_pre_commit_hook_command(now, &config)
        }
        Commands::PrePushHook => commands::pre_push_hook::run_pre_push_hook_command(now, &config),
        Commands::PreRebaseHook => {
            commands::pre_rebase_hook::run_pre_rebase_hook_command(now, &config)
        }
    };

    std::process::exit(exit_code);
}

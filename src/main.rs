use clap::{Parser, Subcommand};

mod commands;
mod error;
mod id;
mod index;
mod storage;
mod types;

#[derive(Parser)]
#[command(name = "st")]
#[command(about = "Sterna - Git-native issue tracker")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize Sterna in the current repository
    Init,

    /// Create a new issue
    Create {
        /// Issue title
        title: String,

        /// Issue description
        #[arg(short, long)]
        description: Option<String>,

        /// Priority: critical, high, medium, low, backlog
        #[arg(short, long)]
        priority: Option<String>,

        /// Type: epic, task, bug, feature, chore
        #[arg(short = 't', long = "type")]
        issue_type: Option<String>,

        /// Labels (can be specified multiple times)
        #[arg(short, long)]
        label: Vec<String>,
    },

    /// List issues
    List {
        /// Filter by status: open, in_progress, closed
        #[arg(long)]
        status: Option<String>,

        /// Filter by type: epic, task, bug, feature, chore
        #[arg(short = 't', long = "type")]
        issue_type: Option<String>,
    },

    /// Get issue details
    Get {
        /// Issue ID or prefix
        id: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Init => commands::init::run(),
        Commands::Create {
            title,
            description,
            priority,
            issue_type,
            label,
        } => commands::create::run(title, description, priority, issue_type, label),
        Commands::List { status, issue_type } => commands::list::run(status, issue_type),
        Commands::Get { id } => commands::get::run(id),
    };

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

use clap::{Parser, Subcommand};

mod commands;
mod dag;
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

    /// Claim an issue to work on
    Claim {
        /// Issue ID or prefix
        id: String,

        /// Context for the claim (e.g., branch name, PR)
        #[arg(short, long)]
        context: Option<String>,
    },

    /// Release a claimed issue back to open
    Release {
        /// Issue ID or prefix
        id: String,

        /// Reason for releasing
        #[arg(short, long)]
        reason: Option<String>,
    },

    /// Close an issue
    Close {
        /// Issue ID or prefix
        id: String,

        /// Reason for closing
        #[arg(short, long)]
        reason: Option<String>,
    },

    /// Reopen a closed issue
    Reopen {
        /// Issue ID or prefix
        id: String,

        /// Reason for reopening
        #[arg(short, long)]
        reason: Option<String>,
    },

    /// Show issues ready for work (open and unclaimed)
    Ready,

    /// Update an issue
    Update {
        /// Issue ID or prefix
        id: String,

        /// New title
        #[arg(long)]
        title: Option<String>,

        /// New description
        #[arg(short, long)]
        description: Option<String>,

        /// New priority
        #[arg(short, long)]
        priority: Option<String>,

        /// New type
        #[arg(short = 't', long = "type")]
        issue_type: Option<String>,

        /// New labels (replaces existing)
        #[arg(short, long)]
        label: Option<Vec<String>>,
    },

    /// Create a dependency between issues
    Depend {
        /// Source issue ID
        source: String,

        /// Target issue that source depends on (needs done first)
        #[arg(long)]
        needs: Option<String>,

        /// Target issue that source blocks
        #[arg(long)]
        blocks: Option<String>,

        /// Target issue that source relates to
        #[arg(long)]
        relates_to: Option<String>,

        /// Target issue that is parent of source
        #[arg(long)]
        parent: Option<String>,

        /// Target issue that source duplicates
        #[arg(long)]
        duplicates: Option<String>,
    },

    /// Export all issues and edges to JSON
    Export {
        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Import issues and edges from JSON
    Import {
        /// Input file
        file: String,
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
        Commands::Claim { id, context } => commands::claim::run(id, context),
        Commands::Release { id, reason } => commands::release::run(id, reason),
        Commands::Close { id, reason } => commands::close::run(id, reason),
        Commands::Reopen { id, reason } => commands::reopen::run(id, reason),
        Commands::Ready => commands::ready::run(),
        Commands::Update {
            id,
            title,
            description,
            priority,
            issue_type,
            label,
        } => commands::update::run(id, title, description, priority, issue_type, label),
        Commands::Depend {
            source,
            needs,
            blocks,
            relates_to,
            parent,
            duplicates,
        } => commands::depend::run(source, needs, blocks, relates_to, parent, duplicates),
        Commands::Export { output } => commands::export::run(output),
        Commands::Import { file } => commands::import::run(file),
    };

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

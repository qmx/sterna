use clap::{Parser, Subcommand};

mod commands;
mod dag;
mod error;
mod id;
mod snapshot;
mod storage;
mod types;

#[derive(Parser)]
#[command(name = "st", version = env!("VERGEN_GIT_DESCRIBE"))]
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

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Get issue details
    Get {
        /// Issue ID or prefix
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
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
    Ready {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

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

    /// Manage dependencies between issues
    #[command(subcommand)]
    Dep(DepCommands),

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

    /// Remove all Sterna data from this repository
    Purge {
        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },

    /// Push snapshot to remote
    Push {
        /// Remote name (default: origin)
        remote: Option<String>,
    },

    /// Pull and merge snapshot from remote
    Pull {
        /// Remote name (default: origin)
        remote: Option<String>,
    },

    /// Pull then push (convenience command)
    Sync {
        /// Remote name (default: origin)
        remote: Option<String>,
    },

    /// Show onboarding info for agents
    Onboard {
        /// Export default content to stdout (for customization)
        #[arg(long)]
        export: bool,
    },

    /// Show full command reference
    Prime {
        /// Export default content to stdout (for customization)
        #[arg(long)]
        export: bool,
    },
}

#[derive(Subcommand)]
enum DepCommands {
    /// Add a dependency between issues
    Add {
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

    /// Remove a dependency between issues
    Remove {
        /// Source issue ID
        source: String,

        /// Target issue that source depends on
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
        Commands::List {
            status,
            issue_type,
            json,
        } => commands::list::run(status, issue_type, json),
        Commands::Get { id, json } => commands::get::run(id, json),
        Commands::Claim { id, context } => commands::claim::run(id, context),
        Commands::Release { id, reason } => commands::release::run(id, reason),
        Commands::Close { id, reason } => commands::close::run(id, reason),
        Commands::Reopen { id, reason } => commands::reopen::run(id, reason),
        Commands::Ready { json } => commands::ready::run(json),
        Commands::Update {
            id,
            title,
            description,
            priority,
            issue_type,
            label,
        } => commands::update::run(id, title, description, priority, issue_type, label),
        Commands::Dep(cmd) => match cmd {
            DepCommands::Add {
                source,
                needs,
                blocks,
                relates_to,
                parent,
                duplicates,
            } => commands::dep::add(source, needs, blocks, relates_to, parent, duplicates),
            DepCommands::Remove {
                source,
                needs,
                blocks,
                relates_to,
                parent,
                duplicates,
            } => commands::dep::remove(source, needs, blocks, relates_to, parent, duplicates),
        },
        Commands::Export { output } => commands::export::run(output),
        Commands::Import { file } => commands::import::run(file),
        Commands::Purge { yes } => commands::purge::run(yes),
        Commands::Push { remote } => commands::push::run(remote),
        Commands::Pull { remote } => commands::pull::run(remote),
        Commands::Sync { remote } => commands::sync::run(remote),
        Commands::Onboard { export } => commands::onboard::run(export),
        Commands::Prime { export } => commands::prime::run(export),
    };

    if let Err(e) = result {
        eprintln!("error: {}", e);
        std::process::exit(1);
    }
}

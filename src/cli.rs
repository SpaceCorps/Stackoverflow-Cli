//! The command tree. One variant per command; `commands` does the work.

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "stackoverflow",
    version,
    about = "CLI for searching and scraping StackOverflow questions and answers via Apify — YAML-first output optimized for LLM agents",
    after_help = "An LLM agent should start with: stackoverflow agent-readme",
    propagate_version = true,
    disable_help_subcommand = true
)]
pub struct Cli {
    /// Print raw JSON instead of YAML, for scripting
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Args, Clone, Default, Debug)]
pub struct AuthArgs {
    /// Account to run against (see 'stackoverflow accounts list')
    #[arg(short = 'a', long, value_name = "ACCOUNT", global = true)]
    pub account: Option<String>,

    /// Apify API token (or set APIFY_TOKEN env var)
    #[arg(long, value_name = "KEY", env = "APIFY_TOKEN", global = true)]
    pub api_key: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Search StackOverflow for questions matching a query
    Search(SearchArgs),

    /// Scrape a StackOverflow URL (question, tag page, or comma-separated tags)
    Scrape(ScrapeArgs),

    /// Log in with an Apify API token (opens browser to copy token)
    Login(LoginArgs),

    /// Manage accounts and their stored API tokens
    #[command(subcommand)]
    Accounts(AccountsCommand),

    /// Print the operating manual for an LLM agent driving this CLI
    #[command(name = "agent-readme")]
    AgentReadme,
}

// ---------------------------------------------------------------------------------------------
// search

#[derive(Args, Clone, Debug)]
pub struct SearchArgs {
    /// Search query keywords
    #[arg(value_name = "QUERY")]
    pub query: String,

    /// Filter by tags (comma-separated, e.g. 'python,machine-learning')
    #[arg(long = "tagged", value_name = "TAGS")]
    pub tagged: Option<String>,

    /// Include top answers for each question
    #[arg(long)]
    pub answers: bool,

    /// Target Stack Exchange site (default: "stackoverflow")
    #[arg(long, value_name = "SITE")]
    pub site: Option<String>,

    /// Maximum number of questions to return
    #[arg(long = "max", value_name = "COUNT")]
    pub max_results: Option<u32>,

    /// Sort order (e.g. relevance, newest, votes, activity)
    #[arg(long, value_name = "SORT")]
    pub sort: Option<String>,

    #[command(flatten)]
    pub auth: AuthArgs,
}

// ---------------------------------------------------------------------------------------------
// scrape

#[derive(Args, Clone, Debug)]
pub struct ScrapeArgs {
    /// Comma-separated tags to scrape (e.g. 'ai-agent,llm') or StackOverflow URL
    #[arg(value_name = "TARGET")]
    pub target: String,

    /// Include top answers for each question (default: true)
    #[arg(long, conflicts_with = "no_answers")]
    pub answers: bool,

    /// Do not include answers
    #[arg(long = "no-answers", conflicts_with = "answers")]
    pub no_answers: bool,

    /// Target Stack Exchange site (default: "stackoverflow")
    #[arg(long, value_name = "SITE")]
    pub site: Option<String>,

    /// Maximum number of questions to return
    #[arg(long = "max", value_name = "COUNT")]
    pub max_results: Option<u32>,

    /// Sort order (e.g. newest, votes, activity)
    #[arg(long, value_name = "SORT")]
    pub sort: Option<String>,

    #[command(flatten)]
    pub auth: AuthArgs,
}

// ---------------------------------------------------------------------------------------------
// login

#[derive(Args, Clone, Debug)]
pub struct LoginArgs {
    /// Account name to store (default: "default")
    #[arg(value_name = "NAME", default_value = "default")]
    pub name: String,

    /// Apify API token (prompted for securely if omitted)
    #[arg(long, value_name = "KEY", conflicts_with = "api_key_stdin")]
    pub api_key: Option<String>,

    /// Read the API token from stdin, e.g. `pbpaste | stackoverflow login --api-key-stdin`
    #[arg(long)]
    pub api_key_stdin: bool,

    /// Do not open the browser to the API tokens page automatically
    #[arg(long)]
    pub no_browser: bool,

    /// Replace the key on an account that already exists
    #[arg(long)]
    pub force: bool,

    /// Store the key without calling the API to check it first
    #[arg(long)]
    pub no_verify: bool,
}

// ---------------------------------------------------------------------------------------------
// accounts

#[derive(Subcommand, Debug)]
pub enum AccountsCommand {
    /// Add an account and store its API token in the OS keystore
    Add {
        /// Short name for this account, used as --account elsewhere
        name: String,
        /// Apify API token (prompted for, without echo, if omitted)
        #[arg(long, value_name = "KEY", conflicts_with = "api_key_stdin")]
        api_key: Option<String>,
        /// Read the API token from stdin, e.g. `pbpaste | stackoverflow accounts add work --api-key-stdin`
        #[arg(long)]
        api_key_stdin: bool,
        /// Replace the key on an account that already exists
        #[arg(long)]
        force: bool,
        /// Store the key without calling the API to check it first
        #[arg(long)]
        no_verify: bool,
    },
    /// List configured accounts
    List {
        /// Call the API once per account to check validity
        #[arg(long)]
        check: bool,
    },
    /// Check that an account's stored key still works
    Test {
        /// Account name
        name: String,
    },
    /// Remove an account and delete its stored key
    Remove {
        /// Account name
        name: String,
        /// Skip the confirmation prompt
        #[arg(long)]
        yes: bool,
    },
}

#[cfg(test)]
mod tests {
    #[test]
    fn command_tree_is_valid() {
        use clap::CommandFactory;
        super::Cli::command().debug_assert();
    }
}

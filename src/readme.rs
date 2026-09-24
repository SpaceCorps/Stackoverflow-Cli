//! The manual an agent reads before its first call. Markdown by default so it can be pasted
//! into a system prompt or a prompt context; `--json` gives the same rules as structured data.

use crate::{obj, output};

pub fn print() {
    if output::json() {
        output::write(&obj! {
            "tool" => "stackoverflow",
            "apiVersion" => API_VERSION,
            "rules" => RULES,
            "exitCodes" => obj! {
                "0" => "ok",
                "1" => "error - unclassified, report and stop",
                "2" => "network - retry once, then stop",
                "3" => "auth_required - stop, surface the remediation to a human",
                "4" => "not_found - do not retry",
                "5" => "rate_limited - back off before retrying",
                "6" => "invalid_input - fix the call arguments",
                "7" => "no_account - run stackoverflow accounts list or set APIFY_TOKEN",
            },
        });
        return;
    }
    println!("{README}");
}

/// The API version targeted by this CLI build.
pub const API_VERSION: &str = "1.0.0";

const RULES: &[&str] = &[
    "Pass --account or set APIFY_TOKEN / --api-key. Multi-account setups should always pass --account.",
    "Run 'stackoverflow accounts list' if you do not know which accounts exist.",
    "On code auth_required, stop and surface the remediation string. Do not retry.",
    "Use --json when you are going to parse the output in scripts or agent loops.",
    "Always check exit codes: non-zero indicates an error envelope on stderr.",
    "Queries or tag scrapes may take 15-45 seconds to complete via the Apify scraper actor.",
];

const README: &str = r#"# stackoverflow - agent operating manual

A CLI for searching and scraping StackOverflow questions and answers via Apify: search questions,
filter by tags, include top answers, and scrape tag pages or question URLs. Results are YAML on stdout,
errors are YAML on stderr, and `--json` switches both to JSON.

## Authentication & Accounts

An API key (Apify token) can be supplied in three ways:
1. `--account <name>` (short `-a <name>`): Uses a saved account from the OS keystore.
2. `--api-key <key>`: Explicit API token on the command line.
3. `APIFY_TOKEN` environment variable.

### Managing accounts

    stackoverflow login [<name>] [--api-key <key>]  # opens Apify console to copy token
    stackoverflow accounts add <name> --api-key <key> [--force] [--no-verify]
    printf %s "$TOKEN" | stackoverflow accounts add <name> --api-key-stdin
    stackoverflow accounts list [--check]
    stackoverflow accounts test <name>
    stackoverflow accounts remove <name> --yes

`add` tests the token before saving it unless `--no-verify` is passed. The key is stored
securely in your OS keystore (macOS Keychain, Windows DPAPI, Linux Secret Service).
`list --check` tests stored tokens against the Apify API.

## Core Commands

### Search

Search StackOverflow for questions matching a natural language query:

    stackoverflow search "AI coding agent orchestration"
    stackoverflow search "multi-agent workflow" --sort newest --max 20
    stackoverflow search "code review automation" --tagged "python,ai" --answers
    stackoverflow search "memory leak debugging" --site "stackoverflow"

Options:
- `<QUERY>`: Search keywords or query string
- `--tagged <TAGS>`: Filter by comma-separated tags (e.g. 'python,machine-learning')
- `--answers`: Include top answers for each question
- `--site <SITE>`: Stack Exchange site (default: "stackoverflow")
- `--max <COUNT>`: Maximum number of questions to return
- `--sort <SORT>`: Sort order (relevance, newest, votes, activity)

### Scrape

Scrape questions from StackOverflow tags or a specific URL:

    stackoverflow scrape "ai-agent,llm"
    stackoverflow scrape "https://stackoverflow.com/questions/tagged/ai-agent" --max 20
    stackoverflow scrape "https://stackoverflow.com/questions/12345678/some-question"
    stackoverflow scrape "rust" --answers

Options:
- `<TARGET>`: Comma-separated tags (e.g. 'ai-agent,llm') or StackOverflow URL
- `--answers`: Include top answers for each question (default: true)
- `--no-answers`: Omit answers to speed up scraping
- `--site <SITE>`: Stack Exchange site (default: "stackoverflow")
- `--max <COUNT>`: Maximum number of questions to return
- `--sort <SORT>`: Sort order (newest, votes, activity)

## Output Format & Error Codes

Output is YAML on stdout by default; pass `--json` to receive structured JSON.
Errors are printed to stderr as an envelope with a stable exit code:

    0  ok
    1  error          unclassified - report it and stop
    2  network        retry once, then stop
    3  auth_required  stop; give the human the `remediation` string verbatim
    4  not_found      the resource does not exist; do not retry
    5  rate_limited   back off before trying again
    6  invalid_input  fix the call arguments
    7  no_account     run `stackoverflow accounts list` or set APIFY_TOKEN
"#;

# Stackoverflow CLI (`stackoverflow`)

[![CI](https://github.com/SpaceCorps/Stackoverflow-Cli/actions/workflows/ci.yml/badge.svg)](https://github.com/SpaceCorps/Stackoverflow-Cli/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Release](https://img.shields.io/github/v/release/SpaceCorps/Stackoverflow-Cli?color=brightgreen)](https://github.com/SpaceCorps/Stackoverflow-Cli/releases)

A blazing-fast native command-line tool and agent interface for searching and scraping StackOverflow questions and answers via Apify, written in Rust. Features YAML-first output optimized for terminal and LLM readability, raw JSON via `--json`, secure OS keystore credential storage, and multi-account support.

Ported from the original C# prototype [`Stackoverflow.Console`](https://github.com/nielsbosma/Stackoverflow.Console) by Niels Bosma to high-performance native Rust under the [SpaceCorps](https://github.com/SpaceCorps) organization.

---

## Features

- ⚡ **Native Speed & Zero Dependencies:** Standalone static binary with 1–3 ms cold start times.
- 🔐 **OS Keystore Integration:** API tokens are stored in macOS Keychain, Windows DPAPI, or Linux Secret Service (`secret-tool`).
- 🤖 **Agentic Protocol:** Clean YAML on stdout by default; structured JSON with `--json`; deterministic error envelopes on stderr with stable exit codes.
- 👥 **Multi-Account Safety:** Configure multiple named accounts (`stackoverflow accounts add work`) and target them explicitly with `--account work` (`-a work`).
- 📖 **Self-Documenting:** Embedded `stackoverflow agent-readme [--json]` delivers complete machine-readable rules and documentation directly to LLM agents.

---

## Installation

### Precompiled Binaries
Download prebuilt standalone binaries for macOS, Linux, and Windows from [GitHub Releases](https://github.com/SpaceCorps/Stackoverflow-Cli/releases/latest).

### From Source via Cargo
```bash
cargo install --git https://github.com/SpaceCorps/Stackoverflow-Cli --locked
```

---

## Authentication

You can authenticate in three ways:

### 1. Interactive Login (Recommended)
```bash
stackoverflow login [account_name]
```
Opens your browser to `https://console.apify.com/account/integrations`, securely prompts for your token without terminal echo, verifies it against the API, and stores it in your OS keystore.

### 2. Multi-Account Management
```bash
# Add a named account (prompted securely)
stackoverflow accounts add work

# Or pass via stdin in CI/CD pipelines
printf %s "$APIFY_TOKEN" | stackoverflow accounts add ci --api-key-stdin

# List and verify stored accounts
stackoverflow accounts list --check

# Test account connectivity
stackoverflow accounts test work
```

### 3. Environment Variable or Flag
Set `APIFY_TOKEN` in your environment or pass `--api-key <key>` on any command:
```bash
export APIFY_TOKEN="your-apify-token"
stackoverflow search "Rust memory leaks"
```

---

## Commands

### `search`
Search StackOverflow for questions matching a query.

```bash
# Basic keyword search
stackoverflow search "AI coding agent orchestration"

# Filter by tags and include top answers
stackoverflow search "code review automation" --tagged "python,ai" --answers

# Limit results and sort by newest
stackoverflow search "multi-agent workflow" --sort newest --max 20

# Structured JSON output for agent pipelines
stackoverflow search "memory leak debugging" --json
```

**Options:**
- `<QUERY>`: Search query keywords (required)
- `--tagged <TAGS>`: Filter by comma-separated tags (e.g. `'python,machine-learning'`)
- `--answers`: Include top answers for each question
- `--site <SITE>`: Stack Exchange site (default: "stackoverflow")
- `--max <COUNT>`: Maximum number of questions to return
- `--sort <SORT>`: Sort order (`relevance`, `newest`, `votes`, `activity`)
- `--account <ACCOUNT>` / `-a <ACCOUNT>`: Account name to resolve from keystore
- `--api-key <KEY>`: Direct Apify API token
- `--json`: Output as structured JSON

### `scrape`
Scrape questions from StackOverflow tags or a specific URL.

```bash
# Scrape questions by tags
stackoverflow scrape "ai-agent,llm"

# Scrape a specific tag page with limit
stackoverflow scrape "https://stackoverflow.com/questions/tagged/ai-agent" --max 20

# Scrape a specific question URL
stackoverflow scrape "https://stackoverflow.com/questions/12345678/some-question"

# Fast scrape without answer bodies
stackoverflow scrape "rust" --no-answers
```

**Options:**
- `<TARGET>`: Comma-separated tags (e.g. `'ai-agent,llm'`) or StackOverflow URL (required)
- `--answers`: Include top answers for each question (default: true)
- `--no-answers`: Fast scrape without answer bodies
- `--site <SITE>`: Stack Exchange site (default: "stackoverflow")
- `--max <COUNT>`: Maximum number of questions to return
- `--sort <SORT>`: Sort order (`newest`, `votes`, `activity`)
- `--account <ACCOUNT>` / `-a <ACCOUNT>`: Account name to resolve from keystore
- `--api-key <KEY>`: Direct Apify API token
- `--json`: Output as structured JSON

### `agent-readme`
Outputs the embedded operating manual for AI agents driving this CLI:

```bash
# Print markdown manual
stackoverflow agent-readme

# Print structured JSON specifications
stackoverflow agent-readme --json
```

---

## Agentic & Scripting Protocol

Pass `--json` to any command to receive raw JSON instead of YAML:

```bash
stackoverflow search "quantum computing" --json | jq '.[].title'
```

### Structured Error Envelope
Errors are printed to `stderr` with machine-readable exit codes matching the `code` field:

```json
{
  "error": "The Apify API token was rejected.",
  "code": "auth_required",
  "detail": "HTTP 401: Unauthorized",
  "remediation": "Set APIFY_TOKEN, use --api-key <key>, or run: stackoverflow login"
}
```

| Exit Code | `code:` | Meaning | Agent Action |
| :--- | :--- | :--- | :--- |
| `0` | `ok` | Success | Continue |
| `1` | `error` | General unclassified error | Report error |
| `2` | `network` | Network timeout or 5xx error | Retry once with backoff |
| `3` | `auth_required` | Invalid or rejected API token | Surface remediation to user; do NOT retry |
| `4` | `not_found` | Resource not found | Do not retry |
| `5` | `rate_limited` | Upstream rate limit reached (HTTP 429) | Back off before retrying |
| `6` | `invalid_input` | Missing arguments or invalid parameters | Correct arguments |
| `7` | `no_account` | No account specified and no key available | Prompt user to configure account |

---

## Documentation

Full documentation, guides, and agent discovery manifests are available at:
**[https://spacecorps.github.io/Stackoverflow-Cli/](https://spacecorps.github.io/Stackoverflow-Cli/)**

- [Authentication Guide](https://spacecorps.github.io/Stackoverflow-Cli/auth.md)
- [Agent Manual (llms.txt)](https://spacecorps.github.io/Stackoverflow-Cli/llms.txt)
- [Full Agent Specification (llms-full.txt)](https://spacecorps.github.io/Stackoverflow-Cli/llms-full.txt)
- [A2A Agent Card](https://spacecorps.github.io/Stackoverflow-Cli/.well-known/agent-card.json)
- [AgentSkills Manifest](https://spacecorps.github.io/Stackoverflow-Cli/.well-known/agent-skills/index.json)

---

## License

MIT License. See [LICENSE](LICENSE) for details.
Original prototype created by [Niels Bosma](https://github.com/nielsbosma).
Maintained by [SpaceCorps](https://github.com/SpaceCorps).

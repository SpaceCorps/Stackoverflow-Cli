# Stackoverflow CLI (`stackoverflow`)

> A blazing fast native command-line tool and agent interface for searching and scraping StackOverflow questions and answers via Apify, written in Rust.

Canonical URL: [https://spacecorps.github.io/Stackoverflow-Cli/](https://spacecorps.github.io/Stackoverflow-Cli/)

## Core Advantages

- **1–3 ms Startup:** A single static binary that executes in milliseconds, eliminating .NET runtime overhead.
- **OS Keystore Security:** API tokens are stored in macOS Keychain, Windows DPAPI, or Linux Secret Service (`secret-tool`).
- **Agentic Protocol:** Clean YAML on stdout by default; structured JSON with `--json`; deterministic error envelopes on stderr with stable exit codes.
- **Multi-Account Safety:** Manage separate work, personal, and CI accounts with explicit `--account` targeting.
- **Self-Documenting:** Embedded `stackoverflow agent-readme [--json]` delivers complete machine-readable rules directly to LLM agents.

## Quickstart

```bash
# 1. Install
cargo install --git https://github.com/SpaceCorps/Stackoverflow-Cli --locked

# 2. Login (interactively opens Apify console, prompts for token, verifies with API)
stackoverflow login

# 3. Search StackOverflow
stackoverflow search "AI coding agent orchestration"
```

## Commands

### `search`
Search StackOverflow for questions matching a query:
```bash
stackoverflow search "AI coding agent orchestration"
stackoverflow search "code review automation" --tagged "python,ai" --answers
stackoverflow search "multi-agent workflow" --sort newest --max 20
```

### `scrape`
Scrape tags, tagged URLs, or question URLs:
```bash
stackoverflow scrape "ai-agent,llm"
stackoverflow scrape "https://stackoverflow.com/questions/tagged/ai-agent" --max 20
stackoverflow scrape "https://stackoverflow.com/questions/12345678/some-question"
stackoverflow scrape "rust" --no-answers
```

### `accounts`
Manage multiple accounts and credentials:
```bash
stackoverflow login [name]
stackoverflow accounts add work
stackoverflow accounts list --check
stackoverflow accounts test work
stackoverflow accounts remove work --yes
```

### `agent-readme`
View embedded agent instructions and rules:
```bash
stackoverflow agent-readme
stackoverflow agent-readme --json
```

## Agentic Interface & Manifests

- [LLMs Overview (llms.txt)](https://spacecorps.github.io/Stackoverflow-Cli/llms.txt)
- [Full Agent Manual (llms-full.txt)](https://spacecorps.github.io/Stackoverflow-Cli/llms-full.txt)
- [A2A Agent Card](https://spacecorps.github.io/Stackoverflow-Cli/.well-known/agent-card.json)
- [AgentSkills Manifest](https://spacecorps.github.io/Stackoverflow-Cli/.well-known/agent-skills/index.json)
- [Authentication Guide](https://spacecorps.github.io/Stackoverflow-Cli/auth.md)
- [Pricing & Licensing](https://spacecorps.github.io/Stackoverflow-Cli/pricing.md)

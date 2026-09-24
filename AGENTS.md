# AGENTS.md

Notes for whoever extends this next.

`stackoverflow` is a native Rust CLI for searching and scraping StackOverflow questions and answers via Apify, built to be driven by an LLM agent or human developer. It replaces the .NET global tool `Stackoverflow.Console` by Niels Bosma and keeps full interface compatibility: the same commands (`search`, `scrape`), flags, options, and YAML-first output, while adding native multi-account management, OS keystores, deterministic error envelopes, and self-documenting agent capabilities.

For the manual the *agent* reads, run `stackoverflow agent-readme` (or `stackoverflow agent-readme --json`) - that text lives in `src/readme.rs` and is the tool's interface for its agentic audience. This file is for the human editing the source.

## Commands

```bash
cargo build --release              # target/release/stackoverflow
cargo test                         # unit tests + tests/cli.rs against an in-process mock API
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --check
cargo install --path . --locked    # install to ~/.cargo/bin/stackoverflow
```

Use a throwaway config directory when testing so you never touch real credentials:

```bash
export STACKOVERFLOW_CONFIG_DIR=$(mktemp -d) STACKOVERFLOW_SECRET_STORE=plaintext
```

| Variable | Effect |
| --- | --- |
| `STACKOVERFLOW_CONFIG_DIR` | Overrides the config and secrets storage directory |
| `STACKOVERFLOW_SECRET_STORE` | Forces a keystore backend: `dpapi`, `keychain`, `libsecret`, `plaintext` |
| `STACKOVERFLOW_ALLOW_PLAINTEXT_STORE=1` | Permits the plaintext fallback when no OS keystore is available |
| `APIFY_API_URL` | Overrides the API base URL - how `tests/cli.rs` points at its in-process mock |
| `APIFY_TOKEN` | Environment variable for direct Apify API token authentication |

## Layout

```
src/
  main.rs          argument parsing, --json pre-scan, clap error formatting
  cli.rs           clap derive command hierarchy and help text
  commands/
    mod.rs         dispatcher
    search.rs      search command implementation
    scrape.rs      scrape command implementation (tags, tagged URLs, question URLs)
    accounts.rs    accounts add|list|test|remove
    login.rs       interactive/scripted browser-assisted login
  client.rs        blocking HTTP (ureq + rustls), status -> ErrorCode mapping
  error.rs         ErrorCode (= exit code) and Error {code, message, detail, remediation}
  output.rs        YAML default (serde_norway), JSON (--json), error envelopes, obj! macro
  account.rs       multi-account resolution and API key management
  config.rs        config.yaml, paths, atomic file writes, 0600 permissions, cross-process lock
  secrets.rs       Keychain (/usr/bin/security), libsecret (secret-tool), DPAPI, plaintext fallback
  readme.rs        agent-readme text and embedded API rules
tests/
  cli.rs           drives the binary against an in-process mock TCP server
```

## Why it is built this way

**Blocking HTTP, no async runtime.** A CLI makes one to a few requests. Tokio would cost more in startup than it saves. Cold starts execute in 1–3 ms.

**The Keychain goes through `/usr/bin/security`, not the Security framework.** Reading through `/usr/bin/security` avoids code-signing prompt loops on rebuilt unsigned binaries.

**Responses stay `serde_json::Value`.** The upstream Apify scraper actor returns dataset items whose fields may evolve over time. This CLI prints whatever comes back cleanly in YAML or JSON.

**Multi-account safety:** Accounts are stored with unique names (`stackoverflow accounts add <name> --api-key <key>`) and referenced via `--account <name>` (`-a <name>`). Direct `--api-key <key>` and `APIFY_TOKEN` are also supported for single-key workflows.

**Deterministic exit codes:** Exit codes map directly to the `error::ErrorCode` enum values (1..=7), matching the `code:` field in stderr error envelopes.

## Releasing

CI (`.github/workflows/ci.yml`) runs fmt, clippy and tests on Linux, macOS and Windows for every push and pull request. Publishing a GitHub Release builds pre-compiled binaries for:
- macOS Apple Silicon (`aarch64-apple-darwin`)
- macOS Intel (`x86_64-apple-darwin`)
- Linux x86_64 static musl (`x86_64-unknown-linux-musl`)
- Linux ARM64 static musl (`aarch64-unknown-linux-musl`)
- Windows x64 (`x86_64-pc-windows-msvc`)

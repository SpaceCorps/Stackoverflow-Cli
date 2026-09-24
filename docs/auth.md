---
title: "Authentication Guide"
description: "Authentication methods, credential storage, and error handling for developers and AI agents using the Stackoverflow CLI."
author: "SpaceCorps"
date: "2026-09-24"
---

# Authentication Guide for Stackoverflow CLI

This document outlines authentication methods, credential storage, and error handling for developers and AI agents using the Stackoverflow CLI.

## Overview
The Stackoverflow CLI interfaces directly with the Apify REST API (`https://api.apify.com/v2/`) to execute the StackOverflow scraper actor (`sheshinmcfly~stackoverflow-scraper`). Authentication requires an API token issued through the Apify console. Tokens can be stored in the host operating system's native keystore or supplied directly via environment variables and standard input.

## Prerequisites
- An Apify account ([apify.com](https://apify.com))
- An API token generated from the integrations console (`https://console.apify.com/account/integrations`)
- Stackoverflow CLI installed (`cargo install --git https://github.com/SpaceCorps/Stackoverflow-Cli --locked`)

## Authentication Methods

### 1. Interactive Browser Login (`stackoverflow login`)
The recommended flow for local developer machines:
```bash
stackoverflow login [account_name]
```
1. The CLI launches your system browser to `https://console.apify.com/account/integrations`.
2. You copy or generate an API token.
3. Paste the token into the CLI prompt (input characters are masked).
4. The CLI validates the token with a test query to `GET /v2/users/me`.
5. Upon confirmation, the token is securely saved to the native OS keyring under the account name (defaults to `default`).

### 2. Scripted & Headless Pipelines
For headless CI/CD environments, Docker containers, or autonomous agent runners:
```bash
printf %s "$APIFY_TOKEN" | stackoverflow accounts add ci --api-key-stdin
```
Or pass the token directly as a CLI flag:
```bash
stackoverflow accounts add ci --api-key "$APIFY_TOKEN"
```

### 3. Environment Variable
When no keychain account is specified, the CLI automatically checks for:
- `APIFY_TOKEN`: API token used if no `--account` or `--api-key` is explicitly supplied.

### 4. Direct Command Override
Every API command supports `--api-key`:
```bash
stackoverflow search "Rust lifetimes" --api-key "$APIFY_TOKEN"
```

## Multi-Account Management
Switch or verify accounts using:
```bash
stackoverflow accounts list --check
stackoverflow accounts test [account_name]
stackoverflow accounts remove [account_name] --yes
```

## Error Handling
When authentication fails, commands exit with non-zero exit codes and output standardized JSON error payloads:
- `auth_required` (exit code 3): No token provided or token rejected by upstream API.
- `no_account` (exit code 7): Specified account does not exist in keystore.
- `rate_limited` (exit code 5): Apify API rate limits reached.

## Security Best Practices
1. **Never Commit Keys**: Keep `.env` or plaintext token files out of git repositories.
2. **Use Native OS Keystore**: The CLI automatically utilizes macOS Keychain, Windows DPAPI, or Linux Secret Service (`secret-tool`).
3. **Machine Verification**: When writing agent automation scripts, always pass `--json` to reliably capture machine-readable error codes.

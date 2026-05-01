# Stackoverflow.Console

CLI for searching and scraping StackOverflow questions and answers via Apify — YAML-first output optimized for LLM agent consumption.

## Install

```bash
dotnet tool install -g Stackoverflow.Console
```

## Usage

```bash
# Set API key
export APIFY_TOKEN=your-token-here

# Search questions
stackoverflow search "AI coding agent orchestration"
stackoverflow search "multi-agent workflow" --sort newest --max 20
stackoverflow search "code review automation" --tagged "python,ai"

# Scrape a specific question
stackoverflow scrape "https://stackoverflow.com/questions/12345678/some-question"

# Scrape a tag page
stackoverflow scrape "https://stackoverflow.com/questions/tagged/ai-agent" --max 20
```

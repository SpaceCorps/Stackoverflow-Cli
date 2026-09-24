//! `stackoverflow scrape` command implementation.

use crate::account;
use crate::cli::ScrapeArgs;
use crate::client::ScrapeInput;
use crate::error::Result;
use crate::output;

pub fn run(args: ScrapeArgs) -> Result<()> {
    let resolved = account::resolve(args.auth.account.as_deref(), args.auth.api_key.as_deref())?;
    let client = resolved.client();

    let target = args.target.trim();
    let (tags, keywords, mode) = parse_target(target);

    // Default include_answers to true unless --no-answers was passed
    let include_answers = !args.no_answers;

    let input = ScrapeInput {
        keywords,
        tags,
        site: args.site.unwrap_or_else(|| "stackoverflow".to_string()),
        include_answers,
        mode,
        max_results: args.max_results,
        sort: args.sort,
    };

    let result = client.scrape(&input)?;
    output::write(&result);
    Ok(())
}

fn parse_target(target: &str) -> (Option<Vec<String>>, Option<String>, String) {
    if let Some(idx) = target.find("/questions/tagged/") {
        let after = &target[idx + "/questions/tagged/".len()..];
        let tag_part = after.split(['?', '#', '/']).next().unwrap_or("");
        let tags: Vec<String> =
            tag_part.split(['+', ',']).map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();
        if !tags.is_empty() {
            return (Some(tags), None, "tags".to_string());
        }
    }

    if target.starts_with("http://") || target.starts_with("https://") {
        // If it's a specific question URL or other page URL
        if let Some(idx) = target.find("/questions/") {
            let after = &target[idx + "/questions/".len()..];
            let parts: Vec<&str> = after.split('/').filter(|s| !s.is_empty()).collect();
            if parts.len() >= 2 {
                // e.g. 12345/some-question-slug -> turn slug into keywords
                let title = parts[1].replace('-', " ");
                return (None, Some(title), "search".to_string());
            }
        }
        return (None, Some(target.to_string()), "search".to_string());
    }

    // Comma-separated tags or single tag
    let tags: Vec<String> = target.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect();

    if !tags.is_empty() {
        (Some(tags), None, "tags".to_string())
    } else {
        (None, Some(target.to_string()), "search".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tag_list() {
        let (tags, kw, mode) = parse_target("ai-agent, llm");
        assert_eq!(tags, Some(vec!["ai-agent".into(), "llm".into()]));
        assert_eq!(kw, None);
        assert_eq!(mode, "tags");
    }

    #[test]
    fn parses_tagged_url() {
        let (tags, kw, mode) = parse_target("https://stackoverflow.com/questions/tagged/rust+wasm?tab=newest");
        assert_eq!(tags, Some(vec!["rust".into(), "wasm".into()]));
        assert_eq!(kw, None);
        assert_eq!(mode, "tags");
    }

    #[test]
    fn parses_question_url() {
        let (tags, kw, mode) = parse_target("https://stackoverflow.com/questions/12345/how-to-fix-borrow-checker");
        assert_eq!(tags, None);
        assert_eq!(kw, Some("how to fix borrow checker".into()));
        assert_eq!(mode, "search");
    }
}

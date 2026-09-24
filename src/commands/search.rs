//! `stackoverflow search` command implementation.

use crate::account;
use crate::cli::SearchArgs;
use crate::client::SearchInput;
use crate::error::Result;
use crate::output;

pub fn run(args: SearchArgs) -> Result<()> {
    let resolved = account::resolve(args.auth.account.as_deref(), args.auth.api_key.as_deref())?;
    let client = resolved.client();

    let tags = args.tagged.as_deref().map(split_tags).filter(|t| !t.is_empty());

    let input = SearchInput {
        keywords: Some(args.query),
        tags,
        site: args.site.unwrap_or_else(|| "stackoverflow".to_string()),
        include_answers: args.answers,
        mode: "search".to_string(),
        max_results: args.max_results,
        sort: args.sort,
    };

    let result = client.search(&input)?;
    output::write(&result);
    Ok(())
}

fn split_tags(tags: &str) -> Vec<String> {
    tags.split(',').map(str::trim).filter(|s| !s.is_empty()).map(str::to_string).collect()
}

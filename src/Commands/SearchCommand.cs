using System.ComponentModel;
using Stackoverflow.Console.Infrastructure;
using Spectre.Console;
using Spectre.Console.Cli;

namespace Stackoverflow.Console.Commands;

public sealed class SearchCommand : AsyncCommand<SearchCommand.Settings>
{
    public sealed class Settings : GlobalSettings
    {
        [CommandArgument(0, "<QUERY>")]
        [Description("Search query")]
        public required string Query { get; init; }

        [CommandOption("--max <N>")]
        [Description("Maximum results to return")]
        [DefaultValue(10)]
        public int Max { get; init; } = 10;

        [CommandOption("--sort <SORT>")]
        [Description("Sort by: relevance, newest, votes, active")]
        [DefaultValue("relevance")]
        public string Sort { get; init; } = "relevance";

        [CommandOption("--tagged <TAGS>")]
        [Description("Filter by tags (comma-separated, e.g. 'python,machine-learning')")]
        public string? Tagged { get; init; }
    }

    protected override async Task<int> ExecuteAsync(CommandContext context, Settings settings, CancellationToken cancellation)
    {
        AnsiConsole.MarkupLine("[grey]Searching StackOverflow (this may take 30–60s)...[/]");

        using var client = settings.CreateClient();
        var doc = await client.SearchAsync(new SearchInput
        {
            SearchTerms = [settings.Query],
            MaxResults = settings.Max,
            Sort = settings.Sort,
            Tagged = settings.Tagged
        });

        YamlOutput.Write(doc);
        return 0;
    }
}

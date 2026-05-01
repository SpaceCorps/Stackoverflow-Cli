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

        [CommandOption("--tagged <TAGS>")]
        [Description("Filter by tags (comma-separated, e.g. 'python,machine-learning')")]
        public string? Tagged { get; init; }

        [CommandOption("--answers")]
        [Description("Include top answers for each question")]
        public bool Answers { get; init; }
    }

    protected override async Task<int> ExecuteAsync(CommandContext context, Settings settings, CancellationToken cancellation)
    {
        AnsiConsole.MarkupLine("[grey]Searching StackOverflow (this may take 30–60s)...[/]");

        using var client = settings.CreateClient();
        var doc = await client.SearchAsync(new SearchInput
        {
            Keywords = settings.Query,
            Tags = settings.Tagged?.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries),
            IncludeAnswers = settings.Answers
        });

        YamlOutput.Write(doc);
        return 0;
    }
}

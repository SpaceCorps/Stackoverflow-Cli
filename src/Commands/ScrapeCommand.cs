using System.ComponentModel;
using Stackoverflow.Console.Infrastructure;
using Spectre.Console;
using Spectre.Console.Cli;

namespace Stackoverflow.Console.Commands;

public sealed class ScrapeCommand : AsyncCommand<ScrapeCommand.Settings>
{
    public sealed class Settings : GlobalSettings
    {
        [CommandArgument(0, "<URL>")]
        [Description("StackOverflow URL to scrape (question, tag page, or user profile)")]
        public required string Url { get; init; }

        [CommandOption("--max <N>")]
        [Description("Maximum results to return")]
        [DefaultValue(10)]
        public int Max { get; init; } = 10;
    }

    protected override async Task<int> ExecuteAsync(CommandContext context, Settings settings, CancellationToken cancellation)
    {
        AnsiConsole.MarkupLine("[grey]Scraping StackOverflow (this may take 30–60s)...[/]");

        using var client = settings.CreateClient();
        var doc = await client.ScrapeAsync(new ScrapeInput
        {
            StartUrls = [new { url = settings.Url }],
            MaxResults = settings.Max
        });

        YamlOutput.Write(doc);
        return 0;
    }
}

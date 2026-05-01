using System.ComponentModel;
using Stackoverflow.Console.Infrastructure;
using Spectre.Console;
using Spectre.Console.Cli;

namespace Stackoverflow.Console.Commands;

public sealed class ScrapeCommand : AsyncCommand<ScrapeCommand.Settings>
{
    public sealed class Settings : GlobalSettings
    {
        [CommandArgument(0, "<TAGS>")]
        [Description("Comma-separated StackOverflow tags to scrape (e.g. 'ai-agent,llm')")]
        public required string Tags { get; init; }

        [CommandOption("--answers")]
        [Description("Include top answers for each question")]
        public bool Answers { get; init; }
    }

    protected override async Task<int> ExecuteAsync(CommandContext context, Settings settings, CancellationToken cancellation)
    {
        AnsiConsole.MarkupLine("[grey]Scraping StackOverflow (this may take 30–60s)...[/]");

        using var client = settings.CreateClient();
        var doc = await client.ScrapeAsync(new ScrapeInput
        {
            Tags = settings.Tags.Split(',', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries),
            IncludeAnswers = settings.Answers
        });

        YamlOutput.Write(doc);
        return 0;
    }
}

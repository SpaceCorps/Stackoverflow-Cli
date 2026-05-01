using System.ComponentModel;
using Spectre.Console.Cli;

namespace Stackoverflow.Console.Infrastructure;

public class GlobalSettings : CommandSettings
{
    [CommandOption("--api-key <KEY>")]
    [Description("Apify API token (or set APIFY_TOKEN env var)")]
    public string? ApiKey { get; init; }

    public ApifyClient CreateClient()
    {
        var key = ApiKey ?? Environment.GetEnvironmentVariable("APIFY_TOKEN")
            ?? throw new InvalidOperationException(
                "API key required. Use --api-key or set APIFY_TOKEN.");
        return new ApifyClient(key);
    }
}

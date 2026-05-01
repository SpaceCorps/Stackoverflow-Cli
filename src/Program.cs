using Stackoverflow.Console.Commands;
using Spectre.Console.Cli;

var app = new CommandApp();

app.Configure(config =>
{
    config.SetApplicationName("stackoverflow");

    config.AddCommand<SearchCommand>("search")
        .WithDescription("Search StackOverflow for questions matching a query");

    config.AddCommand<ScrapeCommand>("scrape")
        .WithDescription("Scrape a StackOverflow URL (question, tag page, or user profile)");
});

return app.Run(args);

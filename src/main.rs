use dotenvy::dotenv;
use std::sync::Arc;
use subprocess::{self, NullFile, Redirection};
use sysinfo::{ProcessorExt, System, SystemExt};
use tracing::info;

use poise::serenity_prelude as serenity;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(prefix_command, track_edits)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"] command: Option<String>,
) -> Result<(), Error> {
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: &format!(
            "\
Type {}help command for more info on a command.
You can edit your message to the bot and the bot will edit its response.",
            ctx.prefix()
        ),
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}

#[poise::command(prefix_command, track_edits)]
async fn echo(ctx: Context<'_>, #[rest] msg: String) -> Result<(), Error> {
    ctx.say(msg).await?;
    Ok(())
}

#[poise::command(prefix_command, track_edits)]
async fn about(ctx: Context<'_>) -> Result<(), Error> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu = sys.get_global_processor_info().get_cpu_usage();
    let used_mem = sys.get_used_memory();
    let total_mem = sys.get_total_memory();

    let about_str: &String = &format!(
        "```
      Guilds: {}
     Members: {}
      Memory: {}
   CPU Usage: {}
 API Version: {}
Rust Version: {}
    Platform: {}
```",
        ctx.cache().guild_count(),
        ctx.cache().user_count(),
        &format!(
            "{:.2} GB used out of {:.2} GB",
            used_mem as f64 / (1024. * 1024.),
            total_mem as f64 / (1024. * 1024.)
        ),
        &format!("{:.2}%", cpu),
        serenity::GATEWAY_VERSION,
        "rust",
        "os",
    );
    ctx.say(about_str).await?;
    Ok(())
}

#[poise::command(prefix_command, owners_only)]
async fn shutdown(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Shutting down...").await?;
    ctx.framework().shard_manager().shutdown_all().await;
    Ok(())
}

#[poise::command(prefix_command, track_edits, owners_only)]
async fn shell(ctx: Context<'_>, #[rest] cmd: String) -> Result<(), Error> {
    let otpt = subprocess::Exec::shell(cmd)
        .stdin(NullFile)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Merge)
        .capture()?;
    ctx.say(&format!("```{}```", otpt.stdout_str())).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenv().ok();

    let bot_token = std::env::var("TOKEN");
    let bot_prefix = std::env::var("PREFIX");

    let token = bot_token.expect("fatal: missing discord bot token");
    let intents = serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::GUILD_MESSAGES;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            commands: vec![help(), echo(), about(), shutdown(), shell()],
            skip_checks_for_owners: false,
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(bot_prefix.expect("fatal: missing bot prefix")),
                mention_as_prefix: true,
                ignore_bots: true,
                edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                    std::time::Duration::from_secs(3600),
                ))),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|_context, _ready, _framework| Box::pin(async move { Ok(Data {}) }))
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    client.unwrap().start().await.unwrap();
}

async fn event_handler(
    _ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            info!(
                "Logged in as {} ({})",
                data_about_bot.user.name, data_about_bot.user.id
            )
        }
        _ => {}
    }
    Ok(())
}

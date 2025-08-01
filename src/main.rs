use poise::serenity_prelude as serenity;
use shuttle_runtime::SecretStore;
use shuttle_serenity::ShuttleSerenity;
use std::sync::Arc;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

mod admin;
mod events;
mod latex;
mod misc;

#[shuttle_runtime::main]
async fn main(#[shuttle_runtime::Secrets] secret_store: SecretStore) -> ShuttleSerenity {
    let prefix = "-".to_owned();

    let token = secret_store
        .get("TOKEN")
        .expect("fatal: missing discord bot token");

    let intents = serenity::GatewayIntents::MESSAGE_CONTENT
        | serenity::GatewayIntents::DIRECT_MESSAGES
        | serenity::GatewayIntents::GUILD_MESSAGES
        | serenity::GatewayIntents::GUILD_MEMBERS
        | serenity::GatewayIntents::GUILDS;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            event_handler: |ctx, event, framework, data| {
                Box::pin(crate::events::event_handler(ctx, event, framework, data))
            },
            post_command: |ctx| {
                Box::pin(async move {
                    tracing::info!(
                        "User `{}' ({}) ran command `{}' in {}: {}",
                        ctx.author().name,
                        ctx.author().id,
                        ctx.command().qualified_name,
                        if ctx.guild().is_some() {
                            format!(
                                "guild `{}' ({})",
                                ctx.guild().unwrap().name,
                                ctx.guild().unwrap().id
                            )
                        } else {
                            "Direct Messages".to_owned()
                        },
                        ctx.invocation_string()
                    )
                })
            },
            commands: vec![
                crate::admin::shell(),
                crate::admin::shutdown(),
                crate::misc::echo(),
                crate::misc::help(),
                crate::misc::about(),
                crate::latex::tex(),
            ],
            skip_checks_for_owners: false,
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some(prefix.clone()),
                mention_as_prefix: true,
                ignore_bots: true,
                edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                    std::time::Duration::from_secs(3600),
                ))),
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, _framework| {
            Box::pin(async move {
                ctx.set_activity(Some(serenity::ActivityData::listening(format!(
                    "{prefix}help"
                ))));
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .map_err(shuttle_runtime::CustomError::new)?;

    Ok(client.into())
}

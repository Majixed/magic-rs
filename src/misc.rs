use sysinfo::System;

use crate::{Context, Error, serenity};

/// Displays the bot's help page
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

/// Echoes back your message
#[poise::command(prefix_command, track_edits)]
pub async fn echo(ctx: Context<'_>, #[rest] msg: String) -> Result<(), Error> {
    ctx.say(msg).await?;
    Ok(())
}

/// Shows information about the bot
#[poise::command(prefix_command, track_edits)]
pub async fn about(ctx: Context<'_>) -> Result<(), Error> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu = sys.global_cpu_usage();
    let used_mem = sys.used_memory();
    let total_mem = sys.total_memory();

    let about_str: &String = &format!(
        "```
      Guilds: {}
     Members: {}
      Memory: {}
   CPU Usage: {}
 API Version: {}
    Platform: {}
```",
        ctx.cache().guild_count(),
        ctx.cache().user_count(),
        &format!(
            "{:.2} GB used out of {:.2} GB",
            used_mem as f64 / (1024. * 1024. * 1024.),
            total_mem as f64 / (1024. * 1024. * 1024.)
        ),
        &format!("{:.2}%", cpu),
        serenity::GATEWAY_VERSION,
        System::kernel_long_version(),
    );
    ctx.say(about_str).await?;
    Ok(())
}

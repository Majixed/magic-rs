use subprocess::{self, NullFile, Redirection};

use crate::{Context, Error};

/// Disconnects and shuts down the bot
#[poise::command(prefix_command, owners_only)]
pub async fn shutdown(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Shutting down...").await?;
    ctx.framework().shard_manager().shutdown_all().await;
    Ok(())
}

/// Runs system commands in the operating environment
#[poise::command(prefix_command, track_edits, owners_only)]
pub async fn shell(ctx: Context<'_>, #[rest] cmd: String) -> Result<(), Error> {
    let output = subprocess::Exec::shell(cmd)
        .stdin(NullFile)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Merge)
        .capture()?;
    let mut out_str = output.stdout_str();

    if out_str.is_empty() {
        ctx.say("No output.").await?;
    } else {
        out_str.truncate(1992);
        ctx.say(&format!("```\n{}\n```", out_str)).await?;
    }
    Ok(())
}

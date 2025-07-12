use tracing::info;

use crate::{Data, Error, serenity};

pub async fn event_handler(
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

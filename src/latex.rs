use image::codecs::png::{CompressionType, FilterType};
use pdf2image_alt::{PdfInfo, RenderOptionsBuilder, render_pdf_single_page};
use poise::CreateReply;
use reqwest::multipart;

use crate::{Context, Error, serenity};

/// Compiles simple LaTeX expressions
#[poise::command(prefix_command, track_edits)]
pub async fn tex(ctx: Context<'_>, #[rest] code: String) -> Result<(), Error> {
    let form = multipart::Form::new().text("filename[]", "document.tex");

    let contents = multipart::Part::text(code);
    let returns = multipart::Part::text("pdf");

    let form = form.part("filecontents[]", contents);
    let form = form.part("return", returns);

    let client = reqwest::Client::new();
    let resp = client
        .post("https://texlive.net/cgi-bin/latexcgi")
        .multipart(form)
        .send()
        .await
        .unwrap()
        .bytes()
        .await?;

    if resp.starts_with("%".as_bytes()) {
        let pdf = PdfInfo::read(&resp).await.unwrap();
        let opts = RenderOptionsBuilder::default()
            .pdftocairo(true)
            .resolution(pdf2image_alt::DPI::Uniform(700))
            .build()?;

        let img = render_pdf_single_page(&resp, &pdf, 1, &opts).await.unwrap();

        let mut png_bytes = Vec::new();

        let encoder = image::codecs::png::PngEncoder::new_with_quality(
            &mut png_bytes,
            CompressionType::Best,
            FilterType::NoFilter,
        );

        img.write_with_encoder(encoder).unwrap();

        let attach =
            serenity::CreateAttachment::bytes(png_bytes, &format!("{}.png", ctx.author().id));
        let reply = CreateReply::default().attachment(attach);
        ctx.send(reply).await?;
    } else {
        ctx.say("Error occurred.").await?;
    }
    Ok(())
}

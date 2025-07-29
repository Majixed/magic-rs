use grep::{printer::StandardBuilder, regex::RegexMatcher, searcher::SearcherBuilder};
use image::{
    EncodableLayout,
    codecs::png::{CompressionType, FilterType, PngEncoder},
};
use pdf2image_alt::{PdfInfo, RenderOptionsBuilder, render_pdf_single_page};
use poise::CreateReply;
use reqwest::multipart;
use tokio::task;

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
        .await?
        .bytes()
        .await?;

    let bytes = resp.as_bytes();

    if bytes.starts_with(b"%PDF") {
        let pdf = PdfInfo::read(bytes).await?;
        let opts = RenderOptionsBuilder::default()
            .pdftocairo(true)
            .resolution(pdf2image_alt::DPI::Uniform(700))
            .build()?;

        let content = render_pdf_single_page(bytes, &pdf, 1, &opts)
            .await?
            .into_rgba8();

        let ht = content.height();
        let wd = if content.width() > 1000 {
            content.width()
        } else {
            1000
        };

        let pixels = image::Rgba([0, 0, 0, 0]);

        let mut img = image::ImageBuffer::from_pixel(wd, ht, pixels);

        image::imageops::overlay(&mut img, &content, 0, 0);

        let png_bytes = task::spawn_blocking(move || {
            let mut png_bytes = Vec::new();

            let encoder = PngEncoder::new_with_quality(
                &mut png_bytes,
                CompressionType::Best,
                FilterType::NoFilter,
            );

            img.write_with_encoder(encoder).unwrap();

            png_bytes
        })
        .await?;

        let attach =
            serenity::CreateAttachment::bytes(png_bytes, format!("{}.png", ctx.author().id));
        let reply = CreateReply::default().attachment(attach);
        ctx.send(reply).await?;
    } else {
        let re = RegexMatcher::new(r"^!")?;
        let mut searcher = SearcherBuilder::new()
            .after_context(10)
            .line_number(false)
            .build();
        let mut printer = StandardBuilder::new()
            .max_matches(Some(1))
            .build_no_color(vec![]);
        searcher.search_slice(&re, bytes, printer.sink(&re))?;

        let output = String::from_utf8(printer.into_inner().into_inner())?;

        if output.is_empty() {
            ctx.say("No output.").await?;
        } else {
            ctx.say(format!("```\n{output}\n```")).await?;
        }
    }
    Ok(())
}

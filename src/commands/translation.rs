use bottomify;
use futures::AsyncReadExt;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::Colour;

use uwuifier::uwu_ify_sse;

const LEN: usize = 1 << 16;

#[command]
#[usage = "<command> <text>"]
#[description("Bottom Commands baby")]
async fn bottom(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let subc = args.single::<String>().unwrap();
    let lefts = args.rest().to_string();
    let mut desc = "".to_string();
    let mut out = "".to_string();
    let mut okay = true;
    if subc == "encode".to_string() {
        out = bottomify::bottom::encode_string(&lefts);
        desc = format!("**Text**\n{}\n**Bottom**\n{}", lefts, out);
    } else {
        if subc == "decode".to_string() {
            out = match bottomify::bottom::decode_string(&lefts) {
                Ok(d) => d,
                Err(e) => format!("Error Decoding: {:?}", e),
            };
            desc = format!("**Bottom**\n{}\n**Text**\n{}", lefts, out);
        } else {
            okay = false;
        }
    }

    if okay {
        if out.chars().count() > 2000 {
            msg.channel_id
            .send_message(&ctx.http, |f| {
                f.content("Your text was too big to encode/decode and send. Please send a shorter string.")
            })
            .await
            .unwrap();
        } else {
            msg.channel_id
                .send_message(&ctx.http, |f| {
                    f.embed(|e| {
                        e.description(desc);
                        e.title(format!("Bottom {}", subc));
                        e.color(Colour::from_rgb(255, 255, 51));
                        e
                    })
                })
                .await
                .unwrap();
        }
    } else {
        msg.channel_id
            .send_message(&ctx.http, |f| {
                f.content(format!(
                    "`{}` is not a valid subcommand. Chose either encode or decode",
                    subc
                ))
            })
            .await
            .unwrap();
    };
    Ok(())
}

#[command]
#[usage = "<text>"]
#[description("Uwuify your text")]
async fn uwu(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let mut bytes = vec![0u8; LEN];
    let input = args.rest().to_string();
    input.as_bytes().read(&mut bytes).await.unwrap();
    let len = bytes.len();
    let mut temp_bytes1 = vec![0u8; LEN * 16];
    let mut temp_bytes2 = vec![0u8; LEN * 16];
    let mut uwu_text = String::new();
    uwu_ify_sse(&bytes, len, &mut temp_bytes1, &mut temp_bytes2)
        .read_to_string(&mut uwu_text)
        .await
        .unwrap();
    let ftext = uwu_text.trim_matches(char::from(0));
    msg.channel_id
        .send_message(&ctx.http, |f| {
            f.embed(|e| {
                e.description(format!("**Title**\n{}\n**Uwu**\n{}", input, ftext));
                e.title("UWUified");
                e.color(Colour::FABLED_PINK);
                e
            })
        })
        .await
        .unwrap();
    Ok(())
}

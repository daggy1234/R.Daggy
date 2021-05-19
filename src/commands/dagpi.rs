use crate::utils::client;
use dagpirs;
use serde::{Deserialize, Serialize};
use serde_json;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::Colour;
use serenity_utils::prompt::reaction_prompt;
use std::time::Instant;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
#[derive(Serialize, Deserialize)]
pub struct Joke {
    pub id: String,
    pub joke: String,
}

#[command]
#[bucket("dagpi")]
#[description("get Dagpi Status")]
async fn status(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let cliet = data.get::<client::ClientKey>().expect("No Client");
    let now = Instant::now();
    let resp = cliet.get("https://api.dagpi.xyz", "Nope").await;
    let new_now = Instant::now();
    let diff = new_now.duration_since(now);
    match resp {
        Ok(_r) => {
            msg.channel_id
                .say(&ctx, format!("API is online. Took `{:?}` to ping", diff))
                .await
                .unwrap();
        }
        Err(s) => {
            msg.channel_id
                .say(
                    &ctx,
                    format!(
                        "API is online.\nReturned Status Code `{}`\nTook `{:?}`s",
                        s, diff
                    ),
                )
                .await
                .unwrap();
        }
    }
    Ok(())
}

#[command]
#[bucket("dagpi")]
#[description("Getting a random roast")]
async fn roast(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let c = data.get::<dagpirs::Client>().expect("Data");
    let roast: Result<dagpirs::models::Roast, String> = c.data.roast().await.unwrap();
    match roast {
        Ok(r) => {
            msg.channel_id.say(&ctx, r.roast).await.unwrap();
        }
        Err(s) => {
            msg.channel_id
                .say(&ctx, format!("API error\n{:?}", s))
                .await
                .unwrap();
        }
    };
    Ok(())
}

#[command]
#[bucket("dagpi")]
#[description("A fun joke idk")]
async fn joke(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let cliet = data.get::<client::ClientKey>().expect("No Client");
    let tok = std::env::var("DAGPI_TOKEN").expect("No token");
    let now = Instant::now();
    let resp = cliet.get("https://api.dagpi.xyz/data/joke", &tok).await;
    let new_now = Instant::now();
    let diff = new_now.duration_since(now);
    match resp {
        Ok(r) => {
            let js = r.json::<Joke>().await.unwrap();
            msg.channel_id.say(&ctx, js.joke).await.unwrap();
        }
        Err(s) => {
            msg.channel_id
                .say(
                    &ctx,
                    format!(
                        "API is online.\nReturned Status Code `{}`\nTook `{:?}`s",
                        s, diff
                    ),
                )
                .await
                .unwrap();
        }
    };
    Ok(())
}

#[command]
#[bucket("dagpi")]
#[description("Play guess the headline")]
async fn headline(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let c = data.get::<dagpirs::Client>().expect("Data");
    let hl: Result<dagpirs::models::Headline, String> = c.data.headline().await.unwrap();

    match hl {
        Ok(h) => {
            let emojis = [
                ReactionType::Custom {
                    animated: true,
                    id: EmojiId(734746863340748892),
                    name: Some("giftick".to_string()),
                },
                ReactionType::Custom {
                    animated: true,
                    id: EmojiId(734746864280404018),
                    name: Some("gifcross".to_string()),
                },
            ];
            let ch = msg.channel_id;
            let mut prompt_msg = ch
                .send_message(&ctx.http, |f| {
                    f.embed(|e| {
                        e.title("Is this headline real or not?");
                        e.description(&h.text);
                        e.color(Colour::ORANGE);
                        e
                    })
                })
                .await?;
            println!("Lols");
            let (idx, _) = reaction_prompt(ctx, &prompt_msg, &msg.author, &emojis, 30.0).await?;
            println!("got idx");
            let mut right = "incorrect";
            if idx == 1 && h.fake {
                right = "correct";
            } else {
                if idx == 0 && !h.fake {
                    right = "correct"
                }
            }
            println!("Lols");
            prompt_msg
                .edit(&ctx.http, |f| {
                    f.embed(|e| {
                        e.title(format!("Your were {}", right));
                        e.color(Colour::ORANGE);
                        let mut guess = "is fake";
                        if idx == 0 {
                            guess = "is real"
                        };

                        e.description(format!(
                            "Headline: `{}`\n\nFake: `{:?}`\n\nYour Guess: `{}`",
                            &h.text, &h.fake, guess
                        ))
                    })
                })
                .await?;
        }
        Err(s) => {
            msg.channel_id
                .say(&ctx, format!("API error\n{:?}", s))
                .await
                .unwrap();
        }
    };
    Ok(())
}

#[command]
#[num_args(2)]
#[bucket("dagpi")]
#[usage = "<@member> <flag>"]
#[description("Pride filter for an Image!")]
async fn pride(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let typing = msg.channel_id.start_typing(&ctx.http).unwrap();
    let member = args.single::<id::UserId>().unwrap();
    let cached_guild = msg
        .guild_id
        .unwrap()
        .to_guild_cached(&ctx.cache)
        .await
        .unwrap();
    let cliet = data.get::<client::ClientKey>().expect("No Client");
    if let Some(user) = &cached_guild.members.get(&member) {
        let av = user
            .user
            .static_avatar_url()
            .unwrap_or(user.user.default_avatar_url())
            .replace("webp", "png");
        let flag = args.single::<String>().unwrap();
        let now = Instant::now();
        let tok = std::env::var("DAGPI_TOKEN").expect("No token");
        let resp = cliet
            .get(
                &format!(
                    "https://api.dagpi.xyz/image/pride/?url={}&flag={}",
                    av, flag
                ),
                &tok,
            )
            .await;
        let new_now = Instant::now();
        let diff = new_now.duration_since(now);
        match resp {
            Ok(r) => {
                let byt = r.bytes().await.unwrap();
                let mut f = File::create("pride.png").await.unwrap();
                let slice = byt.to_vec();
                f.write_all(&slice).await.unwrap();
                let files = vec!["pride.png"];
                &msg.channel_id
                    .send_files(&ctx, files, |f| {
                        f.content(format!("Powerd by Dagpi. Took {:?} seconds", diff))
                    })
                    .await
                    .unwrap();
            }
            Err(e) => {
                &msg.channel_id
                    .say(
                        &ctx,
                        format!("Errro Occured. Dagpi Returned a {}.\nTook {:?}", e, diff),
                    )
                    .await
                    .unwrap();
            }
        };
    }
    typing.stop();
    Ok(())
}

#[command]
#[num_args(2)]
#[required_permissions("ADMINISTRATOR")]
async fn approve(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let mut success = false;
    let typing = msg.channel_id.start_typing(&ctx.http).unwrap();
    let member = args.single::<id::UserId>().unwrap();
    let cliet = data.get::<client::ClientKey>().expect("No Client");
    let now = Instant::now();
    let tok = std::env::var("DAGPI_ADMIN").expect("No token");
    let resp = cliet
        .post(
            &format!("https://central.dagpi.xyz/tokens/{}", member.as_u64()),
            &tok,
        )
        .await;
    let new_now = Instant::now();
    let diff = new_now.duration_since(now);
    match resp {
        Ok(_r) => {
            &msg.channel_id
                .say(
                    &ctx,
                    format!("Selected Token was succesfully added.\nTook {:?}", diff),
                )
                .await
                .unwrap();
            let app_id = args.single::<String>().unwrap();
            println!("Test, {}", app_id);
            let resp_b = cliet
                .patch(&format!("https://central.dagpi.xyz/app/{}", app_id), &tok)
                .await;
            match resp_b {
                Ok(_r_b) => {
                    &msg.channel_id
                        .say(
                            &ctx,
                            format!("Selected App was succesfully Patched.\nTook {:?}", diff),
                        )
                        .await
                        .unwrap();

                    success = true;
                }
                Err(e_c) => {
                    &msg.channel_id
                        .say(
                            &ctx,
                            format!(
                                "Errror Occured With Patching. Dagpi Returned a {}.\nTook {:?}",
                                e_c, diff
                            ),
                        )
                        .await
                        .unwrap();
                }
            }
        }
        Err(e) => {
            &msg.channel_id
                .say(
                    &ctx,
                    format!("Errro Occured. Dagpi Returned a {}.\nTook {:?}", e, diff),
                )
                .await
                .unwrap();
        }
    };
    typing.stop();
    if success {
        let cached_guild = msg
            .guild_id
            .unwrap()
            .to_guild_cached(&ctx.cache)
            .await
            .unwrap();
        if let Some(member) = cached_guild.members.get(&member) {
            match member
                .user
                .direct_message(&ctx, |f| {
                    f.content("Congratulations Your dagpi app approved")
                })
                .await
            {
                Ok(_v) => (),
                Err(_e) => {
                    msg.channel_id
                        .say(&ctx, "Couldn't Dm User. Please Contact Manually")
                        .await
                        .unwrap();
                }
            };
        };
    }

    Ok(())
}

#[command]
#[num_args(2)]
#[required_permissions("ADMINISTRATOR")]
async fn reject(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let typing = msg.channel_id.start_typing(&ctx.http).unwrap();
    let member = args.single::<String>().unwrap();
    let cliet = data.get::<client::ClientKey>().expect("No Client");
    let now = Instant::now();
    let tok = std::env::var("DAGPI_ADMIN").expect("No token");
    let reason = args.single_quoted::<String>().unwrap();
    let json = serde_json::json!({
        "uu": member,
        "reason": reason
    })
    .to_string();
    println!("{}", json);
    let resp = cliet
        .post_body("https://central.dagpi.xyz/app/reject", &tok, json)
        .await;
    let new_now = Instant::now();
    let diff = new_now.duration_since(now);
    match resp {
        Ok(_r) => {
            &msg.channel_id
                .say(&ctx, format!("App was succesful deleted.\nTook {:?}", diff))
                .await
                .unwrap();
        }
        Err(e) => {
            &msg.channel_id
                .say(
                    &ctx,
                    format!("Errro Occured. Dagpi Returned a {}.\nTook {:?}", e, diff),
                )
                .await
                .unwrap();
        }
    };
    typing.stop();
    Ok(())
}

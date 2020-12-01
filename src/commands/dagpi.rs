use crate::utils::client;
use serde::{Deserialize, Serialize};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
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
            &format!("https://central.dagpi.xyz/addtoken/{}/", member.as_u64()),
            &tok,
        )
        .await;
    let new_now = Instant::now();
    let diff = new_now.duration_since(now);
    match resp {
        Ok(_r) => {
            success = true;
            &msg.channel_id
                .say(
                    &ctx,
                    format!("Selected Token was succesfully added.\nTook {:?}", diff),
                )
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
    if success {
        let cached_guild = msg
            .guild_id
            .unwrap()
            .to_guild_cached(&ctx.cache)
            .await
            .unwrap();
        let u = args.single::<id::UserId>().unwrap();
        if let Some(member) = cached_guild.members.get(&u) {
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

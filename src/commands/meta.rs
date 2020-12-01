use crate::utils::uptimer;
use crate::CommandCounter;
use crate::ShardManagerContainer;
use serenity::builder::{CreateEmbed, CreateEmbedAuthor};
use serenity::client::bridge::gateway::ShardId;
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::Colour;
use std::fmt::Write;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id.say(&ctx.http, "Pong!").await?;

    Ok(())
}

#[command]
async fn uptime(ctx: &Context, msg: &Message) -> CommandResult {
    let uptime = {
        let data = ctx.data.read().await;
        let uptimer = data
            .get::<uptimer::UptimerKey>()
            .expect("Failed to get Uptimer");
        uptimer.uptime_string()
    };
    msg.channel_id
        .send_message(&ctx, |cm| {
            cm.embed(|ce| {
                ce.title("Uptime")
                    .description(&uptime)
                    .color(Colour::BLURPLE)
            })
        })
        .await
        .unwrap();
    Ok(())
}

#[command]
async fn latency(ctx: &Context, msg: &Message) -> CommandResult {
    // The shard manager is an interface for mutating, stopping, restarting, and
    // retrieving information about shards.
    let data = ctx.data.read().await;

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            msg.reply(ctx, "There was a problem getting the shard manager")
                .await?;

            return Ok(());
        }
    };

    let manager = shard_manager.lock().await;
    let runners = manager.runners.lock().await;

    // Shards are backed by a "shard runner" responsible for processing events
    // over the shard, so we'll get the information about the shard runner for
    // the shard this command was sent over.
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner.latency,
        None => {
            msg.reply(ctx, "No shard found").await?;

            return Ok(());
        }
    };

    msg.reply(ctx, &format!("The shard latency is {:?}", runner.unwrap()))
        .await?;

    Ok(())
}

#[command]
// Options are passed via subsequent attributes.
// Make this command use the "complicated" bucket.
#[bucket = "complicated"]
async fn commands(ctx: &Context, msg: &Message) -> CommandResult {
    let mut contents = "Commands used:\n".to_string();

    let data = ctx.data.read().await;
    let counter = data
        .get::<CommandCounter>()
        .expect("Expected CommandCounter in TypeMap.");

    for (k, v) in counter {
        writeln!(contents, "- {name}: {amount}", name = k, amount = v)?;
    }

    msg.channel_id.say(&ctx.http, &contents).await?;

    Ok(())
}

#[command]
#[description("About the bot")]
async fn about(ctx: &Context, msg: &Message) -> CommandResult {
    let mut embed = CreateEmbed::default();
    embed.color(Colour::BLURPLE);
    embed.title("About R.Daggy");
    embed.description("A basic Moderation bot witha few fun stuff designed for the DaggyTech server. More like a learning experiment. Built with \u{002764} using Rust!");
    embed.field("Urls", "[Invite Link](https://discordapp.com/api/oauth2/authorize?client_id=675589737372975124&permissions=378944&scope=bot)\n[Support Server](https://discord.gg/grGkdeS)\n[API](https://dagpi.xyz)\n[Website](https://dagbot.daggy.tech)\n[Source](https://github.com/Daggy1234/r.daggy)",true);
    embed.field(
        "Metadata",
        "Rustc: 1.47.0\nSerenity: 0.9.1\nBot: v0.1.0",
        true,
    );
    let mut auth = CreateEmbedAuthor::default();
    auth.name(&msg.author.name);
    auth.url(
        &msg.author
            .avatar_url()
            .unwrap_or(String::from(&msg.author.default_avatar_url())),
    );
    msg.channel_id
        .send_message(&ctx, |f| {
            f.content("").embed(|e| {
                e.0 = embed.0;
                e
            })
        })
        .await
        .unwrap();
    Ok(())
}

#[command]
#[description("About the bot")]
async fn source(ctx: &Context, msg: &Message) -> CommandResult {
    let mut embed = CreateEmbed::default();
    embed.color(Colour::BLURPLE);
    embed.title("R.Daggy Source Code");
    embed.description("Here is the source code for R.Daggy\nPlease drop a star if you use it!\nhttps://github.com/Daggy1234/r.daggy");
    embed.image("https://i.imgur.com/rp82uiz.png");
    msg.channel_id
        .send_message(&ctx, |f| {
            f.content("").embed(|e| {
                e.0 = embed.0;
                e
            })
        })
        .await
        .unwrap();
    Ok(())
}

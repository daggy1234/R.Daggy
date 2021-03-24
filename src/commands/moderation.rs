use serenity::builder::CreateEmbed;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::{Color, MessageBuilder};
use std::char;
use std::time::Duration;
use tokio::time::sleep;
#[command]
#[required_permissions("KICK_MEMBERS")]
#[num_args(2)]
#[only_in("guilds")]
#[aliases("ki", "yeet")]
#[usage = "member reason"]
async fn kick(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let member = args.single::<id::UserId>();
    match member {
        Err(_) => {
            msg.channel_id
                .say(&ctx, "Couldn't find the Member ")
                .await
                .unwrap();
        }
        Ok(mem) => match args.single::<String>() {
            Err(_) => {
                msg.channel_id
                    .say(&ctx, "Kick needs a Valid reason ")
                    .await
                    .unwrap();
            }
            Ok(res) => match msg.guild_id {
                None => {
                    msg.channel_id
                        .say(&ctx, "Need a Guild to kick mem from")
                        .await
                        .unwrap();
                }
                Some(g) => {
                    let member = g.member(&ctx, mem).await.unwrap();
                    member.kick_with_reason(&ctx, &res).await.unwrap();
                    msg.channel_id
                        .say(&ctx, format!("Succesfully Kicked {}", member.user.name))
                        .await
                        .unwrap();
                }
            },
        },
    };
    Ok(())
}

#[command]
#[required_permissions("KICK_MEMBERS")]
#[num_args(3)]
#[only_in("guilds")]
#[aliases("b")]
#[usage = "member reason"]
async fn ban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let member = args.single::<id::UserId>();
    match member {
        Err(_) => {
            msg.channel_id
                .say(&ctx, "Couldn't find the Member ")
                .await
                .unwrap();
        }
        Ok(mem) => match args.single::<String>() {
            Err(_) => {
                msg.channel_id
                    .say(&ctx, "Mute needs a Valid reason ")
                    .await
                    .unwrap();
            }
            Ok(res) => match msg.guild_id {
                None => {
                    msg.channel_id
                        .say(&ctx, "Need a Guild to mute from")
                        .await
                        .unwrap();
                }
                Some(g) => {
                    let member = g.member(&ctx, mem).await.unwrap();
                    member.ban_with_reason(&ctx, 0, &res).await.unwrap();
                    msg.channel_id
                        .say(&ctx, format!("Succesfully Banned {}", member.user.name))
                        .await
                        .unwrap();
                }
            },
        },
    };
    Ok(())
}

#[command]
#[only_in("guilds")]
#[usage = "member, duration"]
async fn verify(ctx: &Context, msg: &Message) -> CommandResult {
    let member = msg.author.id;
    println!("Verify");
    let uuid: u64 = 699134157510672424;
    if (msg.channel_id.as_u64()) == &uuid {
        match msg.guild_id {
            None => {
                msg.channel_id
                    .say(&ctx, "Need a Guild to verify")
                    .await
                    .unwrap();
            }
            Some(g) => {
                let mut us = g.member(&ctx, member).await.unwrap();
                if let Some(guild) = g.to_guild_cached(&ctx).await {
                    if let Some(role) = guild.role_by_name("User") {
                        let rid = role.id;
                        if us.roles.contains(&rid) {
                            let rs = msg.channel_id.say(&ctx, "Aldready Has Role").await.unwrap();
                            rs.delete(&ctx).await.unwrap();
                        } else {
                            us.add_role(&ctx, rid).await.unwrap();
                            let rtr = guild.role_by_name("Unverified").unwrap();
                            us.remove_role(&ctx, rtr.id).await.unwrap();
                            let rs = msg
                                .channel_id
                                .say(&ctx, "You Have been verified")
                                .await
                                .unwrap();
                            rs.delete(&ctx).await.unwrap();
                        }
                    }
                }
            }
        }
        msg.delete(&ctx).await.unwrap();
    }
    Ok(())
}

#[command]
#[required_permissions("KICK_MEMBERS")]
#[num_args(2)]
#[only_in("guilds")]
#[aliases("mu")]
#[usage = "member, duration"]
async fn mute(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let member = args.single::<id::UserId>();
    match member {
        Err(_) => {
            msg.channel_id
                .say(&ctx, "Couldn't find the Member ")
                .await
                .unwrap();
        }
        Ok(mem) => match msg.guild_id {
            None => {
                msg.channel_id
                    .say(&ctx, "Need a Guild to kick mem from")
                    .await
                    .unwrap();
            }
            Some(g) => {
                let mut us = g.member(&ctx, mem).await.unwrap();
                if let Some(guild) = g.to_guild_cached(&ctx).await {
                    if let Some(role) = guild.role_by_name("Mute") {
                        let rid = role.id;
                        if us.roles.contains(&rid) {
                            msg.channel_id.say(&ctx, "Aldready Has Role").await.unwrap();
                        } else {
                            us.add_role(&ctx, rid).await.unwrap();
                            msg.channel_id.say(&ctx, "Muted the User").await.unwrap();
                            let time = args.single::<u64>().unwrap();
                            sleep(Duration::from_secs(time)).await;
                            us.remove_role(&ctx, rid).await.unwrap();
                        }
                    }
                }
            }
        },
    };

    Ok(())
}

#[command]
#[required_permissions("KICK_MEMBERS")]
#[num_args(2)]
#[only_in("guilds")]
#[aliases("ub")]
#[usage = "member"]
async fn unban(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let member = args.single::<id::UserId>();
    match member {
        Err(_) => {
            msg.channel_id
                .say(&ctx, "Couldn't find the Member ")
                .await
                .unwrap();
        }
        Ok(mem) => {
            let mut banned = false;
            match msg.guild_id {
                None => {
                    msg.channel_id
                        .say(&ctx, "Need a Guild to kick mem from")
                        .await
                        .unwrap();
                }
                Some(g) => {
                    let us = mem.to_user(&ctx).await.unwrap();
                    let _bad = g
                        .bans(&ctx)
                        .await
                        .unwrap()
                        .iter()
                        .map(|ban| {
                            if ban.user.id == us.clone().id {
                                banned = true;
                            }
                            true
                        })
                        .collect::<Vec<bool>>();
                    if banned {
                        g.unban(&ctx, us.id).await.unwrap();
                        msg.channel_id
                            .say(&ctx, format!("Succesfully Unbanned {}", us.name))
                            .await
                            .unwrap();
                    } else {
                        msg.channel_id
                            .say(&ctx, format!("{} was not Banned. Didn;t unban", us.name))
                            .await
                            .unwrap();
                    }
                }
            }
        }
    };
    Ok(())
}

#[command]
#[required_permissions("KICK_MEMBERS")]
#[num_args(6)]
#[usage("<@Role> <r> <g> <b> <emoji> <title>")]
async fn role_embed(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let role = args.single::<RoleId>().unwrap();
    let r = args.single::<u8>().unwrap();
    let g = args.single::<u8>().unwrap();
    let b = args.single::<u8>().unwrap();
    let emoji = args.single::<char>().unwrap();
    let tit = args.single_quoted::<String>().unwrap();
    let mut embed = CreateEmbed::default();
    embed.color(Color::from_rgb(r, g, b));
    embed.title(tit);
    let msga = MessageBuilder::new()
        .push(format!("{}", emoji))
        .push_bold(":    ")
        .mention(&role)
        .build();
    embed.description(msga);
    let m = msg
        .channel_id
        .send_message(&ctx, |f| {
            f.content("").embed(|e| {
                e.0 = embed.0;
                e
            })
        })
        .await
        .unwrap();
    m.react(&ctx, ReactionType::Unicode(emoji.to_string()))
        .await
        .unwrap();
    Ok(())
}

#[command]
#[required_permissions("MANAGE_MESSAGES")]
#[num_args(1)]
#[only_in("guilds")]
#[aliases("prune", "clear")]
#[usage = "amount"]
async fn purge(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let delete_num = args.single::<u64>();
    match delete_num {
        Err(_) => {
            msg.channel_id
                .say(
                    ctx,
                    ":no_entry_sign: The value provided was not a valid number",
                )
                .await?;
        }
        Ok(delete_n) => {
            let mut find_msg = msg
                .channel_id
                .say(
                    ctx,
                    format!(":hourglass: Finding and deleting {} messages...", delete_n),
                )
                .await?;

            let channel = &msg.channel(ctx).await.unwrap().guild().unwrap();

            let messages = &channel
                .messages(ctx, |r| r.before(&msg.id).limit(delete_n))
                .await?;
            let message_ids = messages.iter().map(|m| m.id).collect::<Vec<MessageId>>();

            channel.delete_messages(ctx, message_ids).await?;

            find_msg
                .edit(ctx, |m| {
                    m.content(format!(":white_check_mark: Deleted {} messages", delete_n));
                    m
                })
                .await?;
            msg.delete(&ctx).await.unwrap();
            find_msg.delete(&ctx).await.unwrap();
        }
    }

    Ok(())
}

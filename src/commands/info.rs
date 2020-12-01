use crate::utils::timeparser;
use serenity::builder::{CreateEmbed, CreateEmbedAuthor};
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::model::Permissions;
use serenity::prelude::*;
use serenity::utils::Colour;
use std::convert::TryInto;
use time::OffsetDateTime;

fn parse_serenity(t: u64) -> i64 {
    let mut provess = format!("{}", t.to_string());
    if provess.len() > 10 {
        provess = provess[..10].to_string();
    }
    let start_t = provess.parse::<i64>().unwrap();
    start_t
}

#[command]
#[bucket("info")]
#[description("View IDE RPC for data")]
#[only_in("guilds")]
#[num_args(1)]
#[aliases("rpc", "code")]
#[usage = "<@member>"]
async fn ide(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let typing = msg.channel_id.start_typing(&ctx.http).unwrap();
    let mut message = msg
        .channel_id
        .say(
            &ctx.http,
            "<a:loading:776804948633059338> Loading information about the User...",
        )
        .await?;
    let member = args.single::<id::UserId>();
    let cached_guild = msg
        .guild_id
        .unwrap()
        .to_guild_cached(&ctx.cache)
        .await
        .unwrap();

    let mut embed = CreateEmbed::default();
    embed.color(Colour::BLUE);
    let mut done: bool = false;
    match member {
        Err(_) => {
            msg.channel_id
                .say(&ctx, "Couldn't find the Member ")
                .await
                .unwrap();
        }
        Ok(mem) => {
            let user = mem.to_user(&ctx).await.unwrap();
            let _activities = match &cached_guild.presences.get(&mem) {
                Some(p) => {
                    let _vc = p.activities.iter().map(|f| {
                        let fake: u64 = 383226320970055682;
                        let vsc_a: u64 = 383226320970055681;
                        let vsc_b: u64 = 732565262704050298;
                        let jetb: u64 = 547843598369161278;
                        let jetba: u64 = 547842884448026624;
                        let jetbb: u64 = 384215522050572288;
                        let vc: u64 = 391385173045936131;
                        let id : Vec<u64> = vec![vsc_a, vsc_b, jetb, jetba, jetbb, vc];
                        let appid = match f.application_id{
                            Some(i) => i,
                            None => {
                                done = false;
                                ApplicationId::from(fake)
                            }
                        };
                        if  id.contains(appid.as_u64()) {
                            let t_str = match &f.timestamps {
                                Some(v) => match v.start {
                                    Some(sub) => {
                                        
                                        let current = OffsetDateTime::now_utc();
                                        let start_t = parse_serenity(sub);
                                        let start = OffsetDateTime::from_unix_timestamp(start_t);
                                        let diff = current - start;
                                        timeparser::humanise_time(diff)
                    
                                    },
                                    None => "No Time".to_string()
                                },
                                None => " No Time".to_string()
                            };
                            println!("{}", t_str);
                            embed.description(format!("Working on  **{}**", match &f.details { 
                               Some(v) => v, None => "Unkown"}));
                            
                            let img = match &f.assets {
                                Some(v) => match &v.large_image { Some(i) => i, None => "https://dagbot-is.the-be.st/logo.png"},
                                None => "https://dagbot-is.the-be.st/logo.png"
                            };
                            println!("https://cdn.discordapp.com/app-assets/383226320970055681/{}",img);
                            embed.thumbnail(format!("https://cdn.discordapp.com/app-assets/383226320970055681/{}",img));
                            embed.field("Time",t_str, true);
                            embed.field("Details", match &f.state {Some(v) => v, None => "Unkown"}, true);
                            let mut auth = CreateEmbedAuthor::default();
                            auth.name(&user.name);
                            auth.icon_url("https://cdn.freebiesupply.com/logos/large/2x/visual-studio-code-logo-png-transparent.png");
                            embed.set_author(auth);
                            if appid.as_u64() != &fake {
                                done = true;
                            };
                        }
                        "Memes"
                    }).collect::<Vec<&str>>();
                    "Text"
                }
                None => "No",
            };
            if done {
                message
                    .edit(&ctx, |f| {
                        f.content("").embed(|e| {
                            e.0 = embed.0;
                            e
                        })
                    })
                    .await
                    .unwrap();
            } else {
                msg.channel_id
                    .say(&ctx, "No Supported IDE Detected. Or discord messed up and didn't send us the data we needed.")
                    .await
                    .unwrap();
                message.delete(&ctx).await.unwrap();
            }
        }
    }
    typing.stop();
    Ok(())
}

#[command]
#[bucket("info")]
#[description("Get Spotify Info")]
#[only_in("guilds")]
#[num_args(1)]
#[aliases("spot")]
#[usage = "<@member>"]
async fn spotify(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let typing = msg.channel_id.start_typing(&ctx.http).unwrap();
    let mut message = msg
        .channel_id
        .say(
            &ctx.http,
            "<a:loading:776804948633059338> Loading information about the User...",
        )
        .await?;
    let member = args.single::<id::UserId>();
    let cached_guild = msg
        .guild_id
        .unwrap()
        .to_guild_cached(&ctx.cache)
        .await
        .unwrap();

    let mut embed = CreateEmbed::default();
    embed.color(Colour::new(0x1db954));
    let mut done: bool = false;
    let mut dur: i64 = 0;
    let mut ss = "".to_string();
    let mut se = "".to_string();
    match member {
        Err(_) => {
            msg.channel_id
                .say(&ctx, "Couldn't find the Member ")
                .await
                .unwrap();
        }
        Ok(mem) => {
            let user = mem.to_user(&ctx).await.unwrap();
            let _activities = match &cached_guild.presences.get(&mem) {
                Some(p) => {
                    let _vc = p.activities.iter().map(|f| {
                        if &f.name == "Spotify" {
                            let t_str = match &f.timestamps {
                                Some(v) => match v.start {
                                    Some(sub) => {
                                        
                                        let current = OffsetDateTime::now_utc();
                                        let start_t = parse_serenity(sub);
                                        let start = OffsetDateTime::from_unix_timestamp(start_t);
                                        let diff = current - start;
                                        dur = diff.whole_seconds();
                                        ss = timeparser::min_sec_parse(diff);
                                        timeparser::humanise_time(diff)
                    
                                    },
                                    None => "No Time".to_string()
                                },
                                None => " No Time".to_string()
                            };
                            println!("{}", t_str);
                            embed.description(format!("Listening to **{}**", match &f.details { 
                               Some(v) => v, None => "Unkown"}));
                            embed.field("Artists",match  &f.state { Some(v  )=> v, None => "-"}, true);
                            embed.field("Album",match &f.assets { Some(v) => match &v.large_text {Some(t) => t, None => " No Album"}, None => "No Album"}, true);
                            embed.field("Elapsed", t_str, false);
                            let img = match &f.assets {
                                Some(v) => match &v.large_image { Some(i) => i, None => "https://dagbot-is.the-be.st/logo.png"},
                                None => "https://dagbot-is.the-be.st/logo.png"
                            };
                            println!("https://i.scdn.co/image/{}",img);
                            embed.thumbnail(format!("https://i.scdn.co/image/{}",img.replace("spotify:", "")));
                            let secs = match &f.timestamps {
                                Some(s) => {
                                    let sta = OffsetDateTime::from_unix_timestamp(parse_serenity(s.start.unwrap()));
                                    let end = OffsetDateTime::from_unix_timestamp(parse_serenity(s.end.unwrap()));
                                    let secs = (end-sta).whole_seconds();
                                    se = timeparser::min_sec_parse(end-sta);
                                    secs
                                },
                                None => 0
                            };
                            let mut pcent = (dur as f64 /secs as f64) * 20.0;
                            if pcent == 0.0 {
                                pcent = 1.0;
                            };
                            println!("{}",pcent);
                            let bar = format!("`{}`|{}⚪️{}|`{}`",ss,"─".repeat(((pcent - 1.0) as i64).try_into().unwrap() ),"─".repeat(((20.0 - pcent) as i64).try_into().unwrap() ),se);
                            embed.field("Progress", bar, false);
                            let mut auth = CreateEmbedAuthor::default();
                            auth.name(&user.name);
                            auth.icon_url("https://netsbar.com/wp-content/uploads/2018/10/Spotify_Icon.png");
                            embed.set_author(auth);
                            done = true;

                        }
                        "Memes"
                    }).collect::<Vec<&str>>();
                    "Text"
                }
                None => "No",
            };
            if done {
                message
                    .edit(&ctx, |f| {
                        f.content("").embed(|e| {
                            e.0 = embed.0;
                            e
                        })
                    })
                    .await
                    .unwrap();
            } else {
                msg.channel_id
                    .say(&ctx, "No Spotify Detected")
                    .await
                    .unwrap();
            }
        }
    }
    typing.stop();
    Ok(())
}

#[command]
#[bucket("info")]
#[description("Get User's Information")]
#[only_in("guilds")]
#[num_args(1)]
#[aliases("ui", "user")]
#[usage = "<@member>"]
async fn userinfo(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let typing = msg.channel_id.start_typing(&ctx.http).unwrap();
    let mut message = msg
        .channel_id
        .say(
            &ctx.http,
            "<a:loading:776804948633059338> Loading information about the User...",
        )
        .await?;
    let member = args.single::<id::UserId>();
    let cached_guild = msg
        .guild_id
        .unwrap()
        .to_guild_cached(&ctx.cache)
        .await
        .unwrap();

    let mut embed = CreateEmbed::default();
    match member {
        Err(_) => {
            msg.channel_id
                .say(&ctx, "Couldn't find the Member ")
                .await
                .unwrap();
        }
        Ok(mem) => {
            let user = mem.to_user(&ctx).await.unwrap();
            embed
                .title(format!("{}#{}", &user.name, &user.discriminator))
                .thumbnail(
                    &user
                        .avatar_url()
                        .unwrap_or(String::from(&user.default_avatar_url())),
                )
                .color(Colour::BLURPLE)
                .footer(|f| f.text(format!("ID: {} Created", &user.id)))
                .timestamp(&user.created_at());
            let status = match &cached_guild.presences.get(&mem) {
                Some(presence) => match presence.status {
                    OnlineStatus::Online => "<:online:724328584621064193>",
                    OnlineStatus::DoNotDisturb => "<:dnd:724328585078243438>",
                    OnlineStatus::Idle => "<:idle:724328584893956097>",
                    OnlineStatus::Offline => "<:offline:724328584751349903>",
                    OnlineStatus::Invisible => "<:offline:724328584751349903>",
                    _ => "Error",
                },
                None => "Unknown",
            };

            let activities = match &cached_guild.presences.get(&mem) {
                Some(p) => p
                    .activities
                    .iter()
                    .map(|f| -> String {
                        let pre = match f.kind {
                            ActivityType::Competing => "Competing in",
                            ActivityType::Streaming => "Streaming",
                            ActivityType::Playing => "Playing",
                            ActivityType::Listening => "Listening to",
                            ActivityType::Custom => "",
                            _ => "Unknown",
                        };
                        format!("{} **{}**", pre, f.name)
                    })
                    .collect::<Vec<String>>()
                    .join("\n"),
                None => String::from("None"),
            };
            embed.field("Status", status, true);
            if activities.len() > 5 {
                embed.field("Activities", activities, false);
            };
            message
                .edit(&ctx, |f| {
                    f.content("").embed(|e| {
                        e.0 = embed.0;
                        e
                    })
                })
                .await
                .unwrap();
        }
    };
    typing.stop();
    Ok(())
}

#[command]
#[bucket("info")]
#[description("Get a servers information")]
#[description = "Gives information about a guild"]
#[only_in("guilds")]
#[aliases("server", "guild", "guildinfo")]
async fn serverinfo(ctx: &Context, msg: &Message) -> CommandResult {
    let typing = msg.channel_id.start_typing(&ctx.http).unwrap();
    let mut message = msg
        .channel_id
        .say(
            &ctx.http,
            "<a:loading:776804948633059338> Loading information about the guild...",
        )
        .await?;

    let cached_guild = msg
        .guild_id
        .unwrap()
        .to_guild_cached(&ctx.cache)
        .await
        .unwrap();

    let mut embed = CreateEmbed::default();

    embed
        .title(&cached_guild.name)
        .thumbnail(&cached_guild.icon_url().unwrap_or(String::new()))
        .description(&cached_guild.description.clone().unwrap_or(String::new()))
        .color(Colour::BLURPLE)
        .footer(|f| f.text(format!("ID: {} Created", &cached_guild.id.0)))
        .timestamp(&msg.guild_id.unwrap().created_at());

    // Get the guild owner
    let owner: User = cached_guild.owner_id.to_user(&ctx).await?;
    embed.author(|f| {
        f.name(format!("{}#{} 👑", owner.name, owner.discriminator))
            .icon_url(owner.avatar_url().unwrap_or(String::new()))
    });

    // Emote list
    let mut animated_emotes = 0;
    let mut regular_emotes = 0;
    for emoji in cached_guild.emojis {
        if emoji.1.animated {
            animated_emotes += 1;
        } else {
            regular_emotes += 1;
        };
    }
    let emoji_limit = cached_guild.premium_tier.num() * 50 + 50;
    let emote_string = format!(
        "Regular: {}/{}\nAnimated: {}/{}",
        regular_emotes, emoji_limit, animated_emotes, emoji_limit
    );
    embed.field("Emotes", emote_string, true);

    // Collect the channel count from cache to be speedy
    let mut text_channels = 0;
    let mut voice_channels = 0;

    for channel in &cached_guild.channels {
        let channel = channel.1;
        if channel.kind == ChannelType::Text {
            text_channels += 1;
        } else if channel.kind == ChannelType::Voice {
            voice_channels += 1;
        }
    }
    let channels_text = format!(
        "<:textchannel:724637677395116072> {}\n\
    <:voicechannel:724637677130875001> {}",
        text_channels, voice_channels
    );
    embed.field("Channels", channels_text, true);

    // Collect the member count
    let mut bot_count = 0;
    let mut human_count = 0;

    let mut online_count = 0;
    let mut idle_count = 0;
    let mut dnd_count = 0;
    let mut offline_count = 0;

    for member_result in &cached_guild.members {
        if member_result.1.user.bot {
            bot_count += 1
        } else {
            human_count += 1
        };

        match cached_guild.presences.get(member_result.0) {
            Some(presence) => match presence.status {
                OnlineStatus::Online => online_count += 1,
                OnlineStatus::DoNotDisturb => dnd_count += 1,
                OnlineStatus::Idle => idle_count += 1,
                OnlineStatus::Offline => offline_count += 1,
                OnlineStatus::Invisible => offline_count += 1,
                _ => {}
            },
            None => {
                offline_count += 1;
            }
        }
    }
    let member_count = bot_count + human_count;
    // Add the member count to the embed
    let member_string = format!(
        "<:online:724328584621064193> {} \
    <:idle:724328584893956097> {} \
    <:dnd:724328585078243438> {} \
    <:offline:724328584751349903> {}\n\
    {} :detective: {} :robot: {} total",
        online_count, idle_count, dnd_count, offline_count, human_count, bot_count, member_count
    );

    embed.field("Members", member_string, false);

    // Boosts
    let boosts_string = format!(
        "Level {}\n{} boosts",
        cached_guild.premium_tier.num(),
        cached_guild.premium_subscription_count
    );
    embed.field("Boosts", boosts_string, true);

    // Role count
    embed.field("Roles", format!("{} roles", cached_guild.roles.len()), true);
    // Send the embed
    message
        .edit(&ctx, |f| {
            f.content("").embed(|e| {
                e.0 = embed.0;
                e
            })
        })
        .await?;
    typing.stop();
    Ok(())
}

#[command]
#[description = "Gives you an invite link"]
async fn invite(ctx: &Context, msg: &Message) -> CommandResult {
    let mut permissions = Permissions::default();
    permissions.set(Permissions::ADMINISTRATOR, true);

    let invite_url = match ctx
        .cache
        .current_user()
        .await
        .invite_url(ctx, permissions)
        .await
    {
        Ok(v) => v,
        Err(why) => {
            println!("Error creating invite url: {:?}", why);

            msg.channel_id
                .say(&ctx, ":no_entry_sign: Error creating invite url")
                .await?;

            return Ok(());
        }
    };

    msg.channel_id
        .send_message(&ctx, |m| {
            m.embed(|e| {
                e.title("Invite link")
                    .url(invite_url)
                    .color(Colour::BLURPLE)
            })
        })
        .await?;

    Ok(())
}

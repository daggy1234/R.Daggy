#![allow(dead_code)]

use serenity::framework::standard::{
    help_commands::{
        create_customised_help_data, Command, CustomisedHelpData, GroupCommandsPair,
        SuggestedCommandName, Suggestions,
    },
    Args, CommandGroup, HelpOptions,
};

use serenity::{
    client::Context,
    http::Http,
    model::channel::Message,
    model::id::{ChannelId, UserId},
    utils::Colour,
    Error,
};

use std::borrow::Borrow;

use std::{collections::HashSet, fmt::Write};
use tracing::warn;

macro_rules! warn_about_failed_send {
    ($customised_help:expr, $error:expr) => {
        warn!(
            "Failed to send {:?} because: {:?}",
            $customised_help, $error
        );
    };
}

fn flatten_group_to_string(
    group_text: &mut String,
    group: &GroupCommandsPair,
    nest_level: usize,
    help_options: &HelpOptions,
    single_group: bool,
) {
    let repeated_indent_str = help_options.indention_prefix.repeat(nest_level);

    if nest_level > 0 {
        let _ = writeln!(group_text, "{}__**{}**__", repeated_indent_str, group.name,);
    }

    if !group.prefixes.is_empty() {
        let _ = writeln!(
            group_text,
            "{}{}: `{}`",
            &repeated_indent_str,
            help_options.group_prefix,
            group.prefixes.join("`, `"),
        );
    };
    let mut sep = " ";
    if single_group {
        sep = "\n";
    }
    let mut joined_commands = group
        .command_names
        .join(&format!("{}{}", sep, &repeated_indent_str));

    if !group.command_names.is_empty() {
        joined_commands.insert_str(0, &repeated_indent_str);
    }

    let _ = writeln!(group_text, "{}", joined_commands);

    for sub_group in &group.sub_groups {
        if !(sub_group.command_names.is_empty() && sub_group.sub_groups.is_empty()) {
            let mut sub_group_text = String::default();

            flatten_group_to_string(
                &mut sub_group_text,
                &sub_group,
                nest_level + 1,
                &help_options,
                false,
            );

            let _ = write!(group_text, "{}", sub_group_text);
        }
    }
}

fn flatten_group_to_plain_string(
    group_text: &mut String,
    group: &GroupCommandsPair,
    nest_level: usize,
    help_options: &HelpOptions,
) {
    let repeated_indent_str = help_options.indention_prefix.repeat(nest_level);

    if nest_level > 0 {
        let _ = write!(group_text, "\n{}**{}**", repeated_indent_str, group.name);
    }

    if group.prefixes.is_empty() {
        let _ = write!(group_text, ": ");
    } else {
        let _ = write!(
            group_text,
            " ({}: `{}`): ",
            help_options.group_prefix,
            group.prefixes.join("`, `"),
        );
    }

    let joined_commands = group.command_names.join(", ");

    let _ = write!(group_text, "{}", joined_commands);

    for sub_group in &group.sub_groups {
        let mut sub_group_text = String::default();

        flatten_group_to_plain_string(
            &mut sub_group_text,
            &sub_group,
            nest_level + 1,
            &help_options,
        );

        let _ = write!(group_text, "{}", sub_group_text);
    }
}

async fn send_grouped_commands_embed(
    http: impl AsRef<Http>,
    help_options: &HelpOptions,
    channel_id: ChannelId,
    help_description: &str,
    groups: &[GroupCommandsPair],
    colour: Colour,
) -> Result<Message, Error> {
    channel_id
        .send_message(&http, |m| {
            m.embed(|embed| {
                if groups.len() == 1 {
                    embed.title(format!("{} Help", &groups[0].name));
                } else {
                    embed.title("R.Daggy Help Command");
                }
                embed.colour(colour);
                embed.description(help_description);

                for group in groups {
                    let mut embed_text = String::default();

                    flatten_group_to_string(
                        &mut embed_text,
                        &group,
                        0,
                        &help_options,
                        groups.len() == 1,
                    );
                    if groups.len() == 1 {
                        embed.field("Commands", embed_text, true);
                    } else {
                        embed.field(group.name, embed_text, true);
                    };
                }

                embed
            });
            m
        })
        .await
}

async fn send_single_command_embed(
    http: impl AsRef<Http>,
    help_options: &HelpOptions,
    channel_id: ChannelId,
    command: &Command<'_>,
    colour: Colour,
) -> Result<Message, Error> {
    channel_id
        .send_message(&http, |m| {
            m.embed(|embed| {
                embed.title(&command.name);
                embed.colour(colour);

                if let Some(ref desc) = command.description {
                    embed.description(desc);
                }

                if let Some(ref usage) = command.usage {
                    let full_usage_text = if let Some(first_prefix) = command.group_prefixes.get(0)
                    {
                        format!("`{} {} {}`", first_prefix, command.name, usage)
                    } else {
                        format!("`{} {}`", command.name, usage)
                    };

                    embed.field(&help_options.usage_label, full_usage_text, true);
                }

                if !command.usage_sample.is_empty() {
                    let full_example_text = if let Some(first_prefix) =
                        command.group_prefixes.get(0)
                    {
                        let format_example =
                            |example| format!("`{} {} {}`\n", first_prefix, command.name, example);
                        command
                            .usage_sample
                            .iter()
                            .map(format_example)
                            .collect::<String>()
                    } else {
                        let format_example = |example| format!("`{} {}`\n", command.name, example);
                        command
                            .usage_sample
                            .iter()
                            .map(format_example)
                            .collect::<String>()
                    };
                    embed.field(&help_options.usage_sample_label, full_example_text, true);
                }

                embed.field(&help_options.grouped_label, command.group_name, true);

                if !command.aliases.is_empty() {
                    embed.field(
                        &help_options.aliases_label,
                        format!("`{}`", command.aliases.join("`, `")),
                        true,
                    );
                }

                embed.field(&help_options.available_text, &command.availability, true);

                if !command.checks.is_empty() {
                    embed.field(
                        &help_options.checks_label,
                        format!("`{}`", command.checks.join("`, `")),
                        true,
                    );
                }

                if !command.sub_commands.is_empty() {
                    embed.field(
                        &help_options.sub_commands_label,
                        format!("`{}`", command.sub_commands.join("`, `")),
                        true,
                    );
                }

                embed
            });
            m
        })
        .await
}

fn as_vec(v: &Suggestions) -> &Vec<SuggestedCommandName> {
    &v.0
}

fn join(v: &Suggestions, separator: &str) -> String {
    let mut iter = as_vec(v).iter();

    let first_iter_element = match iter.next() {
        Some(first_iter_element) => first_iter_element,
        None => return String::new(),
    };

    let size = as_vec(v)
        .iter()
        .fold(0, |total_size, size| total_size + size.name.len());
    let byte_len_of_sep = as_vec(v).len().saturating_sub(1) * separator.len();
    let mut result = String::with_capacity(size + byte_len_of_sep);
    result.push_str(first_iter_element.name.borrow());

    for element in iter {
        result.push_str(&*separator);
        result.push_str(element.name.borrow());
    }

    result
}

async fn send_suggestion_embed(
    http: impl AsRef<Http>,
    channel_id: ChannelId,
    help_description: &str,
    suggestions: &Suggestions,
    colour: Colour,
) -> Result<Message, Error> {
    let text = help_description.replace("{}", &join(&suggestions, "`, `"));

    channel_id
        .send_message(&http, |m| {
            m.embed(|e| {
                e.colour(colour);
                e.description(text);
                e
            });
            m
        })
        .await
}

async fn send_error_embed(
    http: impl AsRef<Http>,
    channel_id: ChannelId,
    input: &str,
    colour: Colour,
) -> Result<Message, Error> {
    channel_id
        .send_message(&http, |m| {
            m.embed(|e| {
                e.colour(colour);
                e.description(input);
                e
            });
            m
        })
        .await
}

/// Posts an embed showing each individual command group and its commands.
///
/// # Examples
///
/// Use the command with `exec_help`:
///
/// ```rust,no_run
/// # use serenity::prelude::*;
/// use std::{collections::HashSet, hash::BuildHasher};
/// use serenity::{framework::standard::{Args, CommandGroup, CommandResult,
///     StandardFramework, macros::help, HelpOptions,
///     help_commands::*}, model::prelude::*,
/// };
///
/// #[help]
/// async fn my_help(
///     context: &Context,
///     msg: &Message,
///     args: Args,
///     help_options: &'static HelpOptions,
///     groups: &[&'static CommandGroup],
///     owners: HashSet<UserId>
/// ) -> CommandResult {
///     let _ = with_embeds(context, msg, args, &help_options, groups, owners).await;
///     Ok(())
/// }
///
/// let framwork = StandardFramework::new()
///     .help(&MY_HELP);
/// ```

pub async fn with_embeds(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> Option<Message> {
    let formatted_help =
        create_customised_help_data(ctx, msg, &args, &groups, &owners, help_options).await;

    let response_result = match formatted_help {
        CustomisedHelpData::SuggestedCommands {
            ref help_description,
            ref suggestions,
        } => {
            send_suggestion_embed(
                &ctx.http,
                msg.channel_id,
                &help_description,
                &suggestions,
                help_options.embed_error_colour,
            )
            .await
        }
        CustomisedHelpData::NoCommandFound {
            ref help_error_message,
        } => {
            send_error_embed(
                &ctx.http,
                msg.channel_id,
                help_error_message,
                help_options.embed_error_colour,
            )
            .await
        }
        CustomisedHelpData::GroupedCommands {
            ref help_description,
            ref groups,
        } => {
            send_grouped_commands_embed(
                &ctx.http,
                &help_options,
                msg.channel_id,
                &help_description,
                &groups,
                help_options.embed_success_colour,
            )
            .await
        }
        CustomisedHelpData::SingleCommand { ref command } => {
            send_single_command_embed(
                &ctx.http,
                &help_options,
                msg.channel_id,
                &command,
                help_options.embed_success_colour,
            )
            .await
        }

        _ => panic!("WTF"),
    };

    match response_result {
        Ok(response) => Some(response),
        Err(why) => {
            warn_about_failed_send!(&formatted_help, why);
            None
        }
    }
}

/// Turns grouped commands into a `String` taking plain help format into account.
fn grouped_commands_to_plain_string(
    help_options: &HelpOptions,
    help_description: &str,
    groups: &[GroupCommandsPair],
) -> String {
    let mut result = "__**Commands**__\n".to_string();
    let _ = writeln!(result, "{}", &help_description);

    for group in groups {
        let _ = write!(result, "\n**{}**", &group.name);

        flatten_group_to_plain_string(&mut result, &group, 0, &help_options);
    }

    result
}

fn single_command_to_plain_string(help_options: &HelpOptions, command: &Command<'_>) -> String {
    let mut result = String::default();
    let _ = writeln!(result, "__**{}**__", command.name);

    if !command.aliases.is_empty() {
        let _ = writeln!(
            result,
            "**{}**: `{}`",
            help_options.aliases_label,
            command.aliases.join("`, `")
        );
    }

    if let Some(ref description) = command.description {
        let _ = writeln!(
            result,
            "**{}**: {}",
            help_options.description_label, description
        );
    };

    if let Some(ref usage) = command.usage {
        if let Some(first_prefix) = command.group_prefixes.get(0) {
            let _ = writeln!(
                result,
                "**{}**: `{} {} {}`",
                help_options.usage_label, first_prefix, command.name, usage
            );
        } else {
            let _ = writeln!(
                result,
                "**{}**: `{} {}`",
                help_options.usage_label, command.name, usage
            );
        }
    }

    if !command.usage_sample.is_empty() {
        if let Some(first_prefix) = command.group_prefixes.get(0) {
            let format_example = |example| {
                let _ = writeln!(
                    result,
                    "**{}**: `{} {} {}`",
                    help_options.usage_sample_label, first_prefix, command.name, example
                );
            };
            command.usage_sample.iter().for_each(format_example);
        } else {
            let format_example = |example| {
                let _ = writeln!(
                    result,
                    "**{}**: `{} {}`",
                    help_options.usage_sample_label, command.name, example
                );
            };
            command.usage_sample.iter().for_each(format_example);
        }
    }

    let _ = writeln!(
        result,
        "**{}**: {}",
        help_options.grouped_label, command.group_name
    );
    let _ = writeln!(
        result,
        "**{}**: {}",
        help_options.available_text, command.availability
    );

    result
}

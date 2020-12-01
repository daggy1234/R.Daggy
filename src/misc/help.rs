#![allow(dead_code)]

use serenity::{
    framework::standard::{
        Args, Check, CheckResult, Command as InternalCommand, CommandGroup, CommandOptions,
        HelpBehaviour, HelpOptions, OnlyIn,
    },
    model::{
        guild::{Member, Role},
        id::RoleId,
    },
};

use serenity::{
    cache::Cache,
    client::Context,
    framework::standard::CommonOptions,
    http::CacheHttp,
    http::Http,
    model::channel::Message,
    model::id::{ChannelId, UserId},
    utils::Colour,
    Error,
};

use std::{
    borrow::Borrow,
    collections::{HashMap, HashSet},
    fmt::Write,
    ops::{Index, IndexMut},
};

use futures::future::{BoxFuture, FutureExt};
use tracing::warn;

async fn has_correct_permissions(
    cache: impl AsRef<Cache>,
    options: &impl CommonOptions,
    message: &Message,
) -> bool {
    if options.required_permissions().is_empty() {
        true
    } else if let Some(guild) = message.guild(&cache).await {
        let perms = guild.user_permissions_in(message.channel_id, message.author.id);

        perms.contains(*options.required_permissions())
    } else {
        false
    }
}

fn has_correct_roles(
    options: &impl CommonOptions,
    roles: &HashMap<RoleId, Role>,
    member: &Member,
) -> bool {
    if options.allowed_roles().is_empty() {
        true
    } else {
        options
            .allowed_roles()
            .iter()
            .flat_map(|r| roles.values().find(|role| *r == role.name))
            .any(|g| member.roles.contains(&g.id))
    }
}

macro_rules! format_command_name {
    ($behaviour:expr, $command_name:expr) => {
        match $behaviour {
            HelpBehaviour::Strike => format!("~~`{}`~~", $command_name),
            HelpBehaviour::Nothing => format!("`{}`", $command_name),
            HelpBehaviour::Hide => continue,
            _ => unreachable!(),
        }
    };
}

macro_rules! warn_about_failed_send {
    ($customised_help:expr, $error:expr) => {
        warn!(
            "Failed to send {:?} because: {:?}",
            $customised_help, $error
        );
    };
}

/// A single group containing its name and all related commands that are eligible
/// in relation of help-settings measured to the user.
#[derive(Clone, Debug, Default)]
pub struct GroupCommandsPair {
    pub name: &'static str,
    pub prefixes: Vec<&'static str>,
    pub command_names: Vec<String>,
    pub sub_groups: Vec<GroupCommandsPair>,
}

/// A single suggested command containing its name and Levenshtein distance
/// to the actual user's searched command name.
#[derive(Clone, Debug, Default)]
pub struct SuggestedCommandName {
    pub name: String,
    pub levenshtein_distance: usize,
}

/// A single command containing all related pieces of information.
#[derive(Clone, Debug)]
pub struct Command<'a> {
    pub name: &'static str,
    pub group_name: &'static str,
    pub group_prefixes: &'a [&'static str],
    pub sub_commands: Vec<String>,
    pub aliases: Vec<&'static str>,
    pub availability: &'a str,
    pub description: Option<&'static str>,
    pub usage: Option<&'static str>,
    pub usage_sample: Vec<&'static str>,
    pub checks: Vec<String>,
    pub(crate) _nonexhaustive: (),
}

/// Contains possible suggestions in case a command could not be found
/// but are similar enough.
#[derive(Clone, Debug, Default)]
pub struct Suggestions(pub Vec<SuggestedCommandName>);

impl Suggestions {
    /// Immutably borrow inner `Vec`.
    #[inline]
    fn as_vec(&self) -> &Vec<SuggestedCommandName> {
        &self.0
    }

    /// Concats names of suggestions with a given `separator`.
    fn join(&self, separator: &str) -> String {
        let mut iter = self.as_vec().iter();

        let first_iter_element = match iter.next() {
            Some(first_iter_element) => first_iter_element,
            None => return String::new(),
        };

        let size = self
            .as_vec()
            .iter()
            .fold(0, |total_size, size| total_size + size.name.len());
        let byte_len_of_sep = self.as_vec().len().saturating_sub(1) * separator.len();
        let mut result = String::with_capacity(size + byte_len_of_sep);
        result.push_str(first_iter_element.name.borrow());

        for element in iter {
            result.push_str(&*separator);
            result.push_str(element.name.borrow());
        }

        result
    }
}

/// Covers possible outcomes of a help-request and
/// yields relevant data in customised textual
/// representation.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CustomisedHelpData<'a> {
    /// To display suggested commands.
    SuggestedCommands {
        help_description: String,
        suggestions: Suggestions,
    },
    /// To display groups and their commands by name.
    GroupedCommands {
        help_description: String,
        groups: Vec<GroupCommandsPair>,
    },
    /// To display one specific command.
    SingleCommand { command: Command<'a> },
    /// To display failure in finding a fitting command.
    NoCommandFound { help_error_message: &'a str },
}

/// Wraps around a `Vec<Vec<T>>` and provides access
/// via indexing of tuples representing x and y.
#[derive(Debug)]
struct Matrix {
    vec: Vec<usize>,
    width: usize,
}

impl Matrix {
    fn new(columns: usize, rows: usize) -> Matrix {
        Matrix {
            vec: vec![0; columns * rows],
            width: rows,
        }
    }
}

impl Index<(usize, usize)> for Matrix {
    type Output = usize;

    fn index(&self, matrix_entry: (usize, usize)) -> &usize {
        &self.vec[matrix_entry.1 * self.width + matrix_entry.0]
    }
}

impl IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, matrix_entry: (usize, usize)) -> &mut usize {
        &mut self.vec[matrix_entry.1 * self.width + matrix_entry.0]
    }
}

pub(crate) fn levenshtein_distance(word_a: &str, word_b: &str) -> usize {
    let len_a = word_a.chars().count();
    let len_b = word_b.chars().count();

    if len_a == 0 {
        return len_b;
    } else if len_b == 0 {
        return len_a;
    }

    let mut matrix = Matrix::new(len_b + 1, len_a + 1);

    for x in 0..len_a {
        matrix[(x + 1, 0)] = matrix[(x, 0)] + 1;
    }

    for y in 0..len_b {
        matrix[(0, y + 1)] = matrix[(0, y)] + 1;
    }

    for (x, char_a) in word_a.chars().enumerate() {
        for (y, char_b) in word_b.chars().enumerate() {
            matrix[(x + 1, y + 1)] = (matrix[(x, y + 1)] + 1)
                .min(matrix[(x + 1, y)] + 1)
                .min(matrix[(x, y)] + if char_a == char_b { 0 } else { 1 });
        }
    }

    matrix[(len_a, len_b)]
}

pub async fn has_all_requirements(
    cache_http: impl CacheHttp + AsRef<Cache>,
    cmd: &CommandOptions,
    msg: &Message,
) -> bool {
    let cache = cache_http.as_ref();

    if let Some(guild_id) = msg.guild_id {
        if let Some(member) = cache.member(guild_id, &msg.author.id).await {
            if let Ok(permissions) = member.permissions(&cache_http).await {
                return if cmd.allowed_roles.is_empty() {
                    permissions.administrator() || has_correct_permissions(&cache, &cmd, msg).await
                } else if let Some(roles) = cache.guild_roles(guild_id).await {
                    permissions.administrator()
                        || (has_correct_roles(&cmd, &roles, &member)
                            && has_correct_permissions(&cache, &cmd, msg).await)
                } else {
                    warn!("Failed to find the guild and its roles.");

                    false
                };
            }
        }
    }

    cmd.only_in != OnlyIn::Guild
}

/// Checks if `search_on` starts with `word` and is then cleanly followed by a
/// `" "`.
#[inline]
fn starts_with_whole_word(search_on: &str, word: &str) -> bool {
    search_on.starts_with(word)
        && search_on
            .get(word.len()..=word.len())
            .map_or(false, |slice| slice == " ")
}

#[inline]
fn find_any_command_matches(
    command: &'static InternalCommand,
    group: &CommandGroup,
    name_to_find: &mut String,
    found_prefix: &mut bool,
) -> Option<&'static str> {
    command
        .options
        .names
        .iter()
        .find(|command_name| {
            group.options.prefixes.iter().any(|prefix| {
                if *found_prefix || starts_with_whole_word(&name_to_find, &prefix) {
                    if !*found_prefix {
                        *found_prefix = true;
                        name_to_find.drain(..=prefix.len());
                    }

                    &name_to_find == command_name
                } else {
                    false
                }
            })
        })
        .cloned()
}

async fn check_common_behaviour(
    cache: impl AsRef<Cache>,
    msg: &Message,
    options: &impl CommonOptions,
    owners: &HashSet<UserId>,
    help_options: &HelpOptions,
) -> HelpBehaviour {
    if !options.help_available() {
        return HelpBehaviour::Hide;
    }

    if options.only_in() == OnlyIn::Dm && !msg.is_private()
        || options.only_in() == OnlyIn::Guild && msg.is_private()
    {
        return help_options.wrong_channel;
    }

    if options.owners_only() && !owners.contains(&msg.author.id) {
        return help_options.lacking_ownership;
    }

    if options.owner_privilege() && owners.contains(&msg.author.id) {
        return HelpBehaviour::Nothing;
    }

    if !has_correct_permissions(&cache, options, msg).await {
        return help_options.lacking_permissions;
    }

    if let Some(guild) = msg.guild(&cache).await {
        if let Some(member) = guild.members.get(&msg.author.id) {
            if !has_correct_roles(options, &guild.roles, &member) {
                return help_options.lacking_role;
            }
        }
    }

    HelpBehaviour::Nothing
}

async fn check_command_behaviour(
    ctx: &Context,
    msg: &Message,
    options: &CommandOptions,
    group_checks: &[&Check],
    owners: &HashSet<UserId>,
    help_options: &HelpOptions,
) -> HelpBehaviour {
    let b = check_common_behaviour(&ctx, msg, &options, owners, help_options).await;

    if b == HelpBehaviour::Nothing && (!options.owner_privilege || !owners.contains(&msg.author.id))
    {
        for check in group_checks.iter().chain(options.checks) {
            if !check.check_in_help {
                continue;
            }

            let mut args = Args::new("", &[]);

            if let CheckResult::Failure(_) = (check.function)(ctx, msg, &mut args, options).await {
                return help_options.lacking_conditions;
            }
        }
    }

    b
}

#[allow(clippy::too_many_arguments)]
async fn _nested_group_command_search<'rec, 'a: 'rec>(
    ctx: &'rec Context,
    msg: &'rec Message,
    groups: &'rec [&'static CommandGroup],
    name: &'rec mut String,
    help_options: &'a HelpOptions,
    similar_commands: &'rec mut Vec<SuggestedCommandName>,
    owners: &'rec HashSet<UserId>,
) -> Result<CustomisedHelpData<'a>, ()> {
    for group in groups {
        let group = *group;
        let mut found: Option<&'static InternalCommand> = None;

        let group_behaviour =
            check_common_behaviour(&ctx, msg, &group.options, &owners, &help_options).await;

        match &group_behaviour {
            HelpBehaviour::Nothing => (),
            _ => {
                continue;
            }
        }

        let mut found_group_prefix: bool = false;
        for command in group.options.commands {
            let command = *command;

            let search_command_name_matched = if group.options.prefixes.is_empty() {
                if starts_with_whole_word(&name, &group.name) {
                    name.drain(..=group.name.len());
                }

                command.options.names.iter().find(|n| **n == name).cloned()
            } else {
                find_any_command_matches(&command, &group, name, &mut found_group_prefix)
            };

            if search_command_name_matched.is_some() {
                if HelpBehaviour::Nothing
                    == check_command_behaviour(
                        ctx,
                        msg,
                        &command.options,
                        group.options.checks,
                        &owners,
                        &help_options,
                    )
                    .await
                {
                    found = Some(command);
                } else {
                    break;
                }
            } else if help_options.max_levenshtein_distance > 0 {
                let command_name = if let Some(first_prefix) = group.options.prefixes.get(0) {
                    format!("{} {}", &first_prefix, &command.options.names[0])
                } else {
                    command.options.names[0].to_string()
                };

                let levenshtein_distance = levenshtein_distance(&command_name, &name);

                if levenshtein_distance <= help_options.max_levenshtein_distance
                    && HelpBehaviour::Nothing
                        == check_command_behaviour(
                            ctx,
                            msg,
                            &command.options,
                            group.options.checks,
                            &owners,
                            &help_options,
                        )
                        .await
                {
                    similar_commands.push(SuggestedCommandName {
                        name: command_name,
                        levenshtein_distance,
                    });
                }
            }
        }

        if let Some(command) = found {
            let options = &command.options;

            if !options.help_available {
                return Ok(CustomisedHelpData::NoCommandFound {
                    help_error_message: &help_options.no_help_available_text,
                });
            }

            let available_text = if options.only_in == OnlyIn::Dm {
                &help_options.dm_only_text
            } else if options.only_in == OnlyIn::Guild {
                &help_options.guild_only_text
            } else {
                &help_options.dm_and_guild_text
            };

            similar_commands
                .sort_unstable_by(|a, b| a.levenshtein_distance.cmp(&b.levenshtein_distance));

            let check_names: Vec<String> = command
                .options
                .checks
                .iter()
                .chain(group.options.checks.iter())
                .filter_map(|check| {
                    if check.display_in_help {
                        Some(check.name.to_string())
                    } else {
                        None
                    }
                })
                .collect();

            let sub_command_names: Vec<String> = options
                .sub_commands
                .iter()
                .filter_map(|cmd| {
                    if (*cmd).options.help_available {
                        Some((*cmd).options.names[0].to_string())
                    } else {
                        None
                    }
                })
                .collect();

            return Ok(CustomisedHelpData::SingleCommand {
                command: Command {
                    name: options.names[0],
                    description: options.desc,
                    group_name: group.name,
                    group_prefixes: &group.options.prefixes,
                    checks: check_names,
                    aliases: options.names[1..].to_vec(),
                    availability: available_text,
                    usage: options.usage,
                    usage_sample: options.examples.to_vec(),
                    sub_commands: sub_command_names,
                    _nonexhaustive: (),
                },
            });
        }

        match nested_group_command_search(
            ctx,
            msg,
            &group.options.sub_groups,
            name,
            help_options,
            similar_commands,
            owners,
        )
        .await
        {
            Ok(found) => return Ok(found),
            Err(()) => (),
        }
    }

    Err(())
}

fn nested_group_command_search<'rec, 'a: 'rec>(
    ctx: &'rec Context,
    msg: &'rec Message,
    groups: &'rec [&'static CommandGroup],
    name: &'rec mut String,
    help_options: &'a HelpOptions,
    similar_commands: &'rec mut Vec<SuggestedCommandName>,
    owners: &'rec HashSet<UserId>,
) -> BoxFuture<'rec, Result<CustomisedHelpData<'a>, ()>> {
    _nested_group_command_search(
        ctx,
        msg,
        groups,
        name,
        help_options,
        similar_commands,
        owners,
    )
    .boxed()
}

async fn fetch_single_command<'a>(
    ctx: &Context,
    msg: &Message,
    groups: &[&'static CommandGroup],
    name: &'a str,
    help_options: &'a HelpOptions,
    owners: &HashSet<UserId>,
) -> Result<CustomisedHelpData<'a>, Vec<SuggestedCommandName>> {
    let mut similar_commands: Vec<SuggestedCommandName> = Vec::new();
    let mut name = name.to_string();

    match nested_group_command_search(
        ctx,
        msg,
        &groups,
        &mut name,
        &help_options,
        &mut similar_commands,
        &owners,
    )
    .await
    {
        Ok(found) => Ok(found),
        Err(()) => Err(similar_commands),
    }
}

#[allow(clippy::too_many_arguments)]
async fn fill_eligible_commands<'a>(
    ctx: &Context,
    msg: &Message,
    commands: &[&'static InternalCommand],
    owners: &HashSet<UserId>,
    help_options: &'a HelpOptions,
    group: &'a CommandGroup,
    to_fill: &mut GroupCommandsPair,
    highest_formatter: &mut HelpBehaviour,
) {
    to_fill.name = group.name;
    to_fill.prefixes = group.options.prefixes.to_vec();

    let group_behaviour = {
        if let HelpBehaviour::Hide = highest_formatter {
            HelpBehaviour::Hide
        } else {
            std::cmp::max(
                *highest_formatter,
                check_common_behaviour(&ctx, msg, &group.options, owners, help_options).await,
            )
        }
    };

    *highest_formatter = group_behaviour;

    for command in commands {
        let command = *command;
        let options = &command.options;
        let name = &options.names[0];

        match &group_behaviour {
            HelpBehaviour::Nothing => (),
            _ => {
                let name = format_command_name!(&group_behaviour, &name);
                to_fill.command_names.push(name);

                continue;
            }
        }

        let command_behaviour = check_command_behaviour(
            ctx,
            msg,
            &command.options,
            group.options.checks,
            owners,
            help_options,
        )
        .await;

        let name = format_command_name!(command_behaviour, &name);
        to_fill.command_names.push(name);
    }
}

/// Tries to fetch all commands visible to the user within a group and
/// its sub-groups.
#[allow(clippy::too_many_arguments)]
fn fetch_all_eligible_commands_in_group<'rec, 'a: 'rec>(
    ctx: &'rec Context,
    msg: &'rec Message,
    commands: &'rec [&'static InternalCommand],
    owners: &'rec HashSet<UserId>,
    help_options: &'a HelpOptions,
    group: &'a CommandGroup,
    highest_formatter: HelpBehaviour,
) -> BoxFuture<'rec, GroupCommandsPair> {
    async move {
        let mut group_with_cmds = GroupCommandsPair::default();
        let mut highest_formatter = highest_formatter;

        fill_eligible_commands(
            ctx,
            msg,
            &commands,
            &owners,
            &help_options,
            &group,
            &mut group_with_cmds,
            &mut highest_formatter,
        )
        .await;

        for sub_group in group.options.sub_groups {
            if HelpBehaviour::Hide == highest_formatter {
                break;
            } else if sub_group.options.commands.is_empty()
                && sub_group.options.sub_groups.is_empty()
            {
                continue;
            }

            let grouped_cmd = fetch_all_eligible_commands_in_group(
                ctx,
                msg,
                &sub_group.options.commands,
                &owners,
                &help_options,
                &sub_group,
                highest_formatter,
            )
            .await;

            group_with_cmds.sub_groups.push(grouped_cmd);
        }

        group_with_cmds
    }
    .boxed()
}

async fn create_command_group_commands_pair_from_groups<'a>(
    ctx: &Context,
    msg: &Message,
    groups: &[&'static CommandGroup],
    owners: &HashSet<UserId>,
    help_options: &'a HelpOptions,
) -> Vec<GroupCommandsPair> {
    let mut listed_groups: Vec<GroupCommandsPair> = Vec::default();

    for group in groups {
        let group = *group;

        let group_with_cmds = create_single_group(ctx, msg, group, &owners, &help_options).await;

        if !group_with_cmds.command_names.is_empty() || !group_with_cmds.sub_groups.is_empty() {
            listed_groups.push(group_with_cmds);
        }
    }

    listed_groups
}

async fn create_single_group(
    ctx: &Context,
    msg: &Message,
    group: &CommandGroup,
    owners: &HashSet<UserId>,
    help_options: &HelpOptions,
) -> GroupCommandsPair {
    let mut group_with_cmds = fetch_all_eligible_commands_in_group(
        ctx,
        &msg,
        &group.options.commands,
        &owners,
        &help_options,
        &group,
        HelpBehaviour::Nothing,
    )
    .await;

    group_with_cmds.name = group.name;

    group_with_cmds
}

fn trim_prefixless_group(group_name: &str, searched_group: &mut String) -> bool {
    if group_name == searched_group.as_str() {
        return true;
    } else if starts_with_whole_word(&searched_group, &group_name) {
        searched_group.drain(..=group_name.len());
    }

    false
}

#[allow(clippy::implicit_hasher)]
pub fn searched_lowercase<'rec, 'a: 'rec>(
    ctx: &'rec Context,
    msg: &'rec Message,
    group: &'rec CommandGroup,
    owners: &'rec HashSet<UserId>,
    help_options: &'a HelpOptions,
    searched_named_lowercase: &'rec mut String,
) -> BoxFuture<'rec, Option<CustomisedHelpData<'a>>> {
    async move {
        let is_prefixless_group = {
            group.options.prefixes.is_empty()
                && trim_prefixless_group(&group.name.to_lowercase(), searched_named_lowercase)
        };
        let mut progressed = is_prefixless_group;
        let is_word_prefix = group.options.prefixes.iter().any(|prefix| {
            if starts_with_whole_word(&searched_named_lowercase, &prefix) {
                searched_named_lowercase.drain(..=prefix.len());
                progressed = true;
            }

            prefix == searched_named_lowercase
        });

        if is_prefixless_group || is_word_prefix {
            let single_group = create_single_group(ctx, msg, &group, owners, &help_options).await;

            if !single_group.command_names.is_empty() {
                return Some(CustomisedHelpData::GroupedCommands {
                    help_description: group
                        .options
                        .description
                        .as_ref()
                        .map(|s| s.to_string())
                        .unwrap_or_default(),
                    groups: vec![single_group],
                });
            }
        } else if progressed || group.options.prefixes.is_empty() {
            for sub_group in group.options.sub_groups {
                if let Some(found_set) = searched_lowercase(
                    ctx,
                    msg,
                    sub_group,
                    owners,
                    help_options,
                    searched_named_lowercase,
                )
                .await
                {
                    return Some(found_set);
                }
            }
        }

        None
    }
    .boxed()
}

#[allow(clippy::implicit_hasher)]
pub async fn create_customised_help_data<'a>(
    ctx: &Context,
    msg: &Message,
    args: &'a Args,
    groups: &[&'static CommandGroup],
    owners: &HashSet<UserId>,
    help_options: &'a HelpOptions,
) -> CustomisedHelpData<'a> {
    if !args.is_empty() {
        let name = args.message();

        return match fetch_single_command(ctx, msg, &groups, &name, &help_options, owners).await {
            Ok(single_command) => single_command,
            Err(suggestions) => {
                let mut searched_named_lowercase = name.to_lowercase();

                for group in groups {
                    if let Some(found_command) = searched_lowercase(
                        ctx,
                        msg,
                        group,
                        owners,
                        help_options,
                        &mut searched_named_lowercase,
                    )
                    .await
                    {
                        return found_command;
                    }
                }

                if suggestions.is_empty() {
                    CustomisedHelpData::NoCommandFound {
                        help_error_message: &help_options.no_help_available_text,
                    }
                } else {
                    CustomisedHelpData::SuggestedCommands {
                        help_description: help_options.suggestion_text.to_string(),
                        suggestions: Suggestions(suggestions),
                    }
                }
            }
        };
    }

    let strikethrough_command_tip = if msg.is_private() {
        &help_options.strikethrough_commands_tip_in_dm
    } else {
        &help_options.strikethrough_commands_tip_in_guild
    };

    let description = if let Some(ref strikethrough_command_text) = strikethrough_command_tip {
        format!(
            "{}\n{}",
            &help_options.individual_command_tip, &strikethrough_command_text
        )
    } else {
        help_options.individual_command_tip.to_string()
    };

    let listed_groups =
        create_command_group_commands_pair_from_groups(ctx, msg, &groups, owners, &help_options)
            .await;

    if listed_groups.is_empty() {
        CustomisedHelpData::NoCommandFound {
            help_error_message: &help_options.no_help_available_text,
        }
    } else {
        CustomisedHelpData::GroupedCommands {
            help_description: description,
            groups: listed_groups,
        }
    }
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

async fn send_suggestion_embed(
    http: impl AsRef<Http>,
    channel_id: ChannelId,
    help_description: &str,
    suggestions: &Suggestions,
    colour: Colour,
) -> Result<Message, Error> {
    let text = help_description.replace("{}", &suggestions.join("`, `"));

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

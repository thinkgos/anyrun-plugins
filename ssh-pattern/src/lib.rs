use std::{fs, mem};

use abi_stable::std_types::{RString, RVec};
use anyrun_plugin::{HandleResult, Match, PluginInfo, get_matches, handler, info, init};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use serde::Deserialize;
use which::which;

mod querier;

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default = "querier::default_prefix")]
    prefix: String,
    #[serde(default = "querier::default_delimiter")]
    terminal_delimiter: char,
    #[serde(default)]
    terminal: Option<querier::Terminal>,
    #[serde(default)]
    terminals: Vec<querier::Terminal>,
    #[serde(default = "querier::default_max_entries")]
    max_entries: usize,
    #[serde(default = "querier::default_ssh_config_paths")]
    ssh_config_paths: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prefix: querier::default_prefix(),
            terminal_delimiter: querier::default_delimiter(),
            terminal: None,
            terminals: vec![],
            max_entries: querier::default_max_entries(),
            ssh_config_paths: querier::default_ssh_config_paths(),
        }
    }
}

struct State {
    config: Config,
    terminals: Vec<querier::Terminal>,
    host_entries: Vec<querier::HostEntry>,
    querier_terminal: Option<String>,
}

impl State {
    fn get_querier_terminal(&self) -> Option<&querier::Terminal> {
        self.querier_terminal
            .as_deref()
            .and_then(|s| self.terminals.iter().find(|t| t.key == s || t.command == s))
    }
}

#[init]
fn init(config_dir: RString) -> State {
    let mut config: Config = fs::File::open(format!("{}/ssh-pattern.ron", config_dir))
        .map(|f| {
            ron::de::from_reader(f).unwrap_or_else(|e| {
                eprintln!("[shhs] Error parsing config, using default: {}", e);
                Config::default()
            })
        })
        .unwrap_or_else(|e| {
            eprintln!("[shhs] Error reading config, using default: {}", e);
            Config::default()
        });
    let mut terminals = vec![
        querier::Terminal {
            key: "a".to_string(),
            command: "alacritty".to_string(),
            args: "-e {}".to_string(),
        },
        querier::Terminal {
            key: "k".to_string(),
            command: "kitty".to_string(),
            args: "-e {}".to_string(),
        },
        querier::Terminal {
            key: "g".to_string(),
            command: "ghostty".to_string(),
            args: "-e {}".to_string(),
        },
        querier::Terminal {
            key: "we".to_string(),
            command: "wezterm".to_string(),
            args: "-e {}".to_string(),
        },
        querier::Terminal {
            key: "f".to_string(),
            command: "foot".to_string(),
            args: "-e {}".to_string(),
        },
        querier::Terminal {
            key: "wt".to_string(),
            command: "wterm".to_string(),
            args: "-e {}".to_string(),
        },
    ];
    let added_terminals = mem::take(&mut config.terminals);
    terminals.extend(added_terminals);

    let host_entries = querier::parse_ssh_config(&config.ssh_config_paths);
    State {
        config,
        terminals,
        host_entries,
        querier_terminal: None,
    }
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "SshPattern".into(),
        icon: "computer".into(),
    }
}

#[get_matches]
fn get_matches(input: RString, state: &mut State) -> RVec<Match> {
    let querier = querier::Querier::parse_input(
        input.as_str(),
        &state.config.prefix,
        state.config.terminal_delimiter,
    );
    let Some(querier) = querier else {
        return RVec::new();
    };
    state.querier_terminal = querier.terminal.map(ToOwned::to_owned);

    let matches: Vec<(&querier::HostEntry, i64)> = if querier.search.is_empty() {
        state
            .host_entries
            .iter()
            .take(state.config.max_entries)
            .map(|h| (h, 0))
            .collect()
    } else {
        let matcher = SkimMatcherV2::default();
        let mut matches: Vec<(&querier::HostEntry, i64)> = state
            .host_entries
            .iter()
            .filter_map(|h| {
                matcher
                    .fuzzy_match(&h.host, querier.search)
                    .map(|score| (h, score))
            })
            .collect();
        matches.sort_by_key(|b| std::cmp::Reverse(b.1));
        matches
    };

    let results: Vec<Match> = matches
        .into_iter()
        .take(state.config.max_entries)
        .map(|(he, _)| {
            let desc = match (&he.user, &he.hostname) {
                (Some(user), Some(hostname)) => format!("{}@{}", user, hostname),
                (None, Some(hostname)) => hostname.clone(),
                (Some(user), None) => user.clone(),
                (None, None) => he.host.clone(),
            };
            Match {
                title: he.host.clone().into(),
                description: Some(desc.into()).into(),
                use_pango: false,
                icon: Some("computer".into()).into(),
                id: None.into(),
            }
        })
        .collect();
    results.into()
}

#[handler]
fn handler(matched: Match, state: &State) -> HandleResult {
    let he = state
        .host_entries
        .iter()
        .find(|h| h.host == matched.title.as_str());
    let Some(he) = he else {
        return HandleResult::Close;
    };

    let exec = format!("ssh {}", he.host);
    let terminal = state
        .get_querier_terminal()
        .or(state.config.terminal.as_ref());
    if let Some(term) = terminal
        && which(&term.command).is_ok()
        && term.execute(&exec)
    {
        return HandleResult::Close;
    }
    // fallback to default terminals
    for term in state.terminals.iter() {
        if which(&term.command).is_ok() && term.execute(&exec) {
            break;
        }
    }
    HandleResult::Close
}

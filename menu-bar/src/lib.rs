use std::{fs, io, mem};

use abi_stable::std_types::{ROption, RString, RVec};
use anyrun_plugin::{HandleResult, Match, PluginInfo, get_matches, handler, info, init};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use serde::Deserialize;
use which::which;

use crate::querier::Querier;

mod querier;

#[derive(Debug, Deserialize)]
pub(crate) struct Terminal {
    pub command: String,
    pub args: String,
}

impl Terminal {
    pub fn execute(&self, exec: &str) -> Result<std::process::Child, io::Error> {
        let mut cmd = std::process::Command::new("sh");
        cmd.arg("-c").arg(format!(
            "{} {}",
            &self.command,
            self.args.replace("{}", exec)
        ));
        cmd.spawn()
    }
}

#[derive(Debug, Deserialize)]
struct Menu {
    key: char,
    title: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    children: Vec<MenuItem>,
}

#[derive(Debug, Deserialize, Clone)]
struct MenuItem {
    title: String,
    #[serde(default)]
    description: Option<String>,
    exec: String,
    #[serde(default)]
    term: bool,
    #[serde(default)]
    id: u64,
}

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default = "querier::default_prefix")]
    prefix: String,
    #[serde(default = "querier::default_delimiter")]
    delimiter: char,
    #[serde(default = "querier::default_max_entries")]
    max_entries: usize,
    #[serde(default)]
    terminal: Option<Terminal>,
    #[serde(default)]
    menus: Vec<Menu>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            prefix: querier::default_prefix(),
            delimiter: querier::default_delimiter(),
            max_entries: querier::default_max_entries(),
            terminal: None,
            menus: Vec::new(),
        }
    }
}

struct State {
    config: Config,
    menus: Vec<Menu>,
}

#[init]
fn init(config_dir: RString) -> State {
    let mut config: Config = fs::File::open(format!("{}/menu-bar.ron", config_dir))
        .map(|f| {
            ron::de::from_reader(f).unwrap_or_else(|e| {
                eprintln!("[menu-bar] Error parsing config, using default: {}", e);
                Config::default()
            })
        })
        .unwrap_or_else(|e| {
            eprintln!("[menu-bar] Error reading config, using default: {}", e);
            Config::default()
        });

    let mut menus = mem::take(&mut config.menus);
    menus
        .iter_mut()
        .flat_map(|item| item.children.iter_mut())
        .enumerate()
        .for_each(|(i, child)| child.id = (i + 1) as u64);

    State { config, menus }
}

#[info]
fn info() -> PluginInfo {
    PluginInfo {
        name: "MenuBar".into(),
        icon: "view-list-tree".into(),
    }
}

#[get_matches]
fn get_matches(input: RString, state: &mut State) -> RVec<Match> {
    let Some(querier) =
        Querier::parse_input(input.as_str(), &state.config.prefix, state.config.delimiter)
    else {
        return RVec::new();
    };

    let is_match_menu = |m: &Menu| {
        querier.menu.is_empty() || querier.menu == m.key.to_string() || querier.menu == m.title
    };
    let results = if let Some(search) = querier.search {
        let matcher = SkimMatcherV2::default();
        let mut matches: Vec<(Match, i64)> = state
            .menus
            .iter()
            .filter(|m| is_match_menu(m))
            .flat_map(|v| {
                v.children
                    .iter()
                    .filter_map(|c| {
                        matcher.fuzzy_match(&c.title, search).map(|score| {
                            (
                                Match {
                                    title: c.title.clone().into(),
                                    description: c.description.as_deref().map(|d| d.into()).into(),
                                    use_pango: false,
                                    icon: None.into(),
                                    id: Some(c.id).into(),
                                },
                                score,
                            )
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        matches.sort_by_key(|b| std::cmp::Reverse(b.1));
        matches
            .into_iter()
            .take(state.config.max_entries)
            .map(|v| v.0)
            .collect()
    } else {
        let result: Vec<Match> = state
            .menus
            .iter()
            .take(state.config.max_entries)
            .filter(|m| is_match_menu(m))
            .map(|m| Match {
                title: format!("({}) {}", m.key, m.title).into(),
                description: m.description.as_deref().map(|d| d.into()).into(),
                use_pango: false,
                icon: Some("folder".into()).into(),
                id: None.into(),
            })
            .collect();
        result
    };
    results.into()
}

#[handler]
fn handler(matched: Match, state: &mut State) -> HandleResult {
    let ROption::RSome(id) = matched.id else {
        return HandleResult::Refresh(false);
    };
    let Some(menu_item) = state
        .menus
        .iter()
        .flat_map(|m| m.children.iter())
        .find(|v| v.id == id)
    else {
        return HandleResult::Close;
    };

    if menu_item.term {
        if let Some(term) = state.config.terminal.as_ref() {
            if let Err(e) = term.execute(&menu_item.exec) {
                eprintln!("[menu-bar] Error running executable: {}", e);
            }
        } else {
            let terminals = &[
                Terminal {
                    command: "alacritty".to_string(),
                    args: "-e {}".to_string(),
                },
                Terminal {
                    command: "kitty".to_string(),
                    args: "-e {}".to_string(),
                },
                Terminal {
                    command: "ghostty".to_string(),
                    args: "-e {}".to_string(),
                },
                Terminal {
                    command: "wezterm".to_string(),
                    args: "-e {}".to_string(),
                },
                Terminal {
                    command: "foot".to_string(),
                    args: "-e {}".to_string(),
                },
                Terminal {
                    command: "wterm".to_string(),
                    args: "-e {}".to_string(),
                },
            ];
            for term in terminals {
                if which(&term.command).is_ok() {
                    if let Err(e) = term.execute(&menu_item.exec) {
                        eprintln!("[menu-bar] Error running executable: {}", e);
                    }
                    break;
                }
            }
        }
    } else {
        if let Err(e) = std::process::Command::new("sh")
            .arg("-c")
            .arg(&menu_item.exec)
            .spawn()
        {
            eprintln!("[menu-bar] Error running executable: {}", e);
        }
    }
    HandleResult::Close
}

use std::io::BufReader;
use std::path::PathBuf;
use std::{collections::HashMap, fs::File};

use serde::Deserialize;
use ssh2_config::{ParseRule, SshConfig};

pub(crate) const DEFAULT_PREFIX: &str = "/";
pub(crate) const DEFAULT_SSH_CONFIG_PATH: &str = "~/.ssh/config";
pub(crate) const DEFAULT_MAX_ENTRIES: usize = 8;
pub(crate) const DEFAULT_TERMINAL_DELIMITER: char = '>';

pub(crate) fn default_prefix() -> String {
    DEFAULT_PREFIX.to_string()
}

pub(crate) fn default_delimiter() -> char {
    DEFAULT_TERMINAL_DELIMITER
}

pub(crate) fn default_ssh_config_paths() -> Vec<String> {
    vec![DEFAULT_SSH_CONFIG_PATH.to_string()]
}

pub(crate) fn default_max_entries() -> usize {
    DEFAULT_MAX_ENTRIES
}

#[derive(Debug, Deserialize)]
pub(crate) struct Terminal {
    pub key: String,
    pub command: String,
    pub args: String,
}

impl Terminal {
    pub fn execute(&self, exec: &str) -> bool {
        let mut cmd = std::process::Command::new("sh");
        cmd.arg("-c").arg(format!(
            "{} {}",
            &self.command,
            self.args.replace("{}", exec)
        ));
        cmd.spawn().is_ok()
    }
}

#[derive(Debug, Default)]
pub(crate) struct Querier<'a> {
    pub terminal: Option<&'a str>,
    pub search: &'a str,
}

impl<'a> Querier<'a> {
    pub(crate) fn parse_input(
        input: &'a str,
        prefix: &str,
        delimiter: char,
    ) -> Option<Querier<'a>> {
        if !input.starts_with(prefix) {
            return None;
        }
        let rest = input.trim_start_matches(prefix);
        if rest.is_empty() {
            return Some(Querier::default());
        }

        if rest.starts_with(delimiter) {
            let rest = rest.trim_start_matches(delimiter);
            if let Some(space_pos) = rest.find(' ') {
                let terminal = &rest[..space_pos];
                let search = rest[space_pos + 1..].trim();

                Some(Querier {
                    terminal: Some(terminal).filter(|s| !s.is_empty()),
                    search,
                })
            } else {
                Some(Querier {
                    terminal: Some(rest).filter(|s| !s.is_empty()),
                    search: "",
                })
            }
        } else {
            Some(Querier {
                terminal: None,
                search: rest.trim(),
            })
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct HostEntry {
    pub host: String,
    pub hostname: Option<String>,
    pub user: Option<String>,
    pub port: Option<u16>,
}

pub(crate) fn parse_ssh_config(paths: &[String]) -> Vec<HostEntry> {
    let home_dir = dirs::home_dir();
    let mut hosts = HashMap::new();
    for path in paths {
        let expanded_path = if path.starts_with("~/")
            && let Some(home_dir) = home_dir.as_ref()
        {
            home_dir.join(path.trim_start_matches("~/"))
        } else {
            PathBuf::from(path)
        };
        let Ok(file) = File::open(expanded_path) else {
            continue;
        };
        let Ok(config) =
            SshConfig::default().parse(&mut BufReader::new(file), ParseRule::ALLOW_UNKNOWN_FIELDS)
        else {
            continue;
        };

        for host in config.get_hosts() {
            for p in host.pattern.iter() {
                if p.negated {
                    continue;
                }
                if p.pattern.chars().any(|c| c == '?' || c == '*') {
                    continue;
                }
                let he = hosts.entry(p.pattern.clone()).or_insert(HostEntry {
                    host: p.pattern.clone(),
                    hostname: None,
                    user: None,
                    port: None,
                });
                if let Some(s) = host.params.host_name.as_deref().map(ToString::to_string) {
                    he.hostname = Some(s);
                }
                if let Some(s) = host.params.user.as_deref().map(ToString::to_string) {
                    he.user = Some(s);
                }
                he.port = host.params.port.or(he.port);
            }
        }
    }
    hosts.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCase<'a> {
        input: &'a str,
        prefix: &'a str,
        delimiter: char,
        expected_terminal: Option<&'a str>,
        expected_search: &'a str,
    }

    fn run_test_case(t: &TestCase) {
        let result = Querier::parse_input(t.input, t.prefix, t.delimiter);
        let querier = match result {
            Some(q) => q,
            None => {
                if t.expected_terminal.is_none() && t.expected_search.is_empty() {
                    return;
                }
                panic!("parse_input returned None, expected Some");
            }
        };
        assert_eq!(t.expected_terminal, querier.terminal, "input: {}", t.input);
        assert_eq!(t.expected_search, querier.search, "input: {}", t.input);
    }

    #[test]
    fn test_parse_input() {
        let cases = [
            TestCase {
                input: "/",
                prefix: "/",
                delimiter: '>',
                expected_terminal: None,
                expected_search: "",
            },
            TestCase {
                input: "/>",
                prefix: "/",
                delimiter: '>',
                expected_terminal: None,
                expected_search: "",
            },
            TestCase {
                input: "/>alacritty",
                prefix: "/",
                delimiter: '>',
                expected_terminal: Some("alacritty"),
                expected_search: "",
            },
            TestCase {
                input: "/>alacritty myhost",
                prefix: "/",
                delimiter: '>',
                expected_terminal: Some("alacritty"),
                expected_search: "myhost",
            },
            TestCase {
                input: "/myhost",
                prefix: "/",
                delimiter: '>',
                expected_terminal: None,
                expected_search: "myhost",
            },
            TestCase {
                input: "myhost",
                prefix: "/",
                delimiter: '>',
                expected_terminal: None,
                expected_search: "",
            },
            TestCase {
                input: "/>  myhost",
                prefix: "/",
                delimiter: '>',
                expected_terminal: None,
                expected_search: "myhost",
            },
            TestCase {
                input: "/ host",
                prefix: "/",
                delimiter: '>',
                expected_terminal: None,
                expected_search: "host",
            },
            TestCase {
                input: "/my host name",
                prefix: "/",
                delimiter: '>',
                expected_terminal: None,
                expected_search: "my host name",
            },
            TestCase {
                input: "//host",
                prefix: "/",
                delimiter: '>',
                expected_terminal: None,
                expected_search: "host",
            },
        ];

        for case in &cases {
            run_test_case(case);
        }
    }
}

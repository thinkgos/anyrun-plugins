pub(crate) const DEFAULT_PREFIX: &str = ":m";
pub(crate) const DEFAULT_MAX_ENTRIES: usize = 8;
pub(crate) const DEFAULT_TERMINAL_DELIMITER: char = '/';

pub(crate) fn default_prefix() -> String {
    DEFAULT_PREFIX.to_string()
}

pub(crate) fn default_delimiter() -> char {
    DEFAULT_TERMINAL_DELIMITER
}

pub(crate) fn default_max_entries() -> usize {
    DEFAULT_MAX_ENTRIES
}

#[derive(Debug, Default)]
pub(crate) struct Querier<'a> {
    pub menu: &'a str,
    pub search: Option<&'a str>,
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
        let rest = input.trim_start_matches(prefix).trim();
        if rest.is_empty() {
            return Some(Querier::default());
        }
        let search = rest.split_once(delimiter);
        let querier = if let Some((search_menu, search_item)) = search {
            Querier {
                menu: search_menu,
                search: Some(search_item),
            }
        } else {
            Querier {
                menu: rest,
                search: None,
            }
        };
        Some(querier)
    }
}

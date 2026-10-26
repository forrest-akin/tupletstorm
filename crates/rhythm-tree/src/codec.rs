//! The v2 link format.

use crate::bar::Bar;
use crate::issue::Issue;
use crate::score::Score;

/// What `decode` read, and the problems it worked around.
#[derive(Clone, Debug, PartialEq)]
pub struct Decoded {
    /// The groove.
    pub score: Score,
    /// Problems, in the order the fields are read.
    pub issues: Vec<Issue>,
}

/// Decodes a v2 fragment or a URL holding one. Never fails: problems come back as issues.
pub fn decode(input: &str) -> Decoded {
    let input = input.trim_matches(is_js_whitespace);
    let params = params(groove_body(input));
    if value(&params, "v") != Some("2") {
        let groove_scribe = params
            .iter()
            .any(|(key, _)| GROOVE_SCRIBE_KEYS.contains(&key.to_lowercase().as_str()));
        let issue = if groove_scribe {
            Issue::GrooveScribeNotImported
        } else {
            Issue::NoGroove
        };
        return Decoded {
            score: Score::new(vec![Bar::sixteenths()]),
            issues: vec![issue],
        };
    }
    let mut issues = vec![];
    let bars = decode_bars(value(&params, "b").unwrap_or("4/4:4"), &mut issues);
    Decoded {
        score: Score::new(bars),
        issues,
    }
}

/// What JavaScript's `trim`, as the reference uses, strips: Unicode White_Space but U+0085, and U+FEFF.
fn is_js_whitespace(c: char) -> bool {
    (c.is_whitespace() && c != '\u{85}') || c == '\u{feff}'
}

/// The `;`-separated bars, each one that can't be read replaced with 4/4 in 16ths.
fn decode_bars(raw: &str, issues: &mut Vec<Issue>) -> Vec<Bar> {
    let mut bars: Vec<Bar> = raw
        .split(';')
        .filter(|item| !item.is_empty())
        .map(|item| {
            Bar::parse(item).unwrap_or_else(|e| {
                issues.push(Issue::BarReplaced(e));
                Bar::sixteenths()
            })
        })
        .collect();
    if bars.is_empty() {
        bars.push(Bar::sixteenths());
    }
    bars
}

const GROOVE_SCRIBE_KEYS: [&str; 6] = ["timesig", "div", "h", "s", "k", "measures"];

/// The part of a URL that holds the groove: a v2 fragment, else the query string, else the input itself.
fn groove_body(s: &str) -> &str {
    let hash_at = s.find('#');
    let hash = hash_at.map_or("", |at| &s[at + 1..]);
    if hash_at.is_some() && hash.split('&').any(|part| part == "v=2") {
        return hash;
    }
    match (s.find('?'), hash_at) {
        (None, Some(_)) => hash,
        (None, None) => s,
        (Some(q), Some(h)) if h > q => &s[q + 1..h],
        (Some(q), _) => &s[q + 1..],
    }
}

/// The `&`-separated `key=value` pairs of a link body: each key at its first position, with its last value.
fn params(body: &str) -> Vec<(&str, &str)> {
    let mut params: Vec<(&str, &str)> = vec![];
    for part in body.split('&').filter(|part| !part.is_empty()) {
        let (key, value) = part.split_once('=').unwrap_or((part, ""));
        match params.iter_mut().find(|(k, _)| *k == key) {
            Some(param) => param.1 = value,
            None => params.push((key, value)),
        }
    }
    params
}

fn value<'a>(params: &[(&'a str, &'a str)], key: &str) -> Option<&'a str> {
    params.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::issue::Level;

    #[test]
    fn reads_bars() {
        let decoded = decode("#v=2&b=4/4:4;7/8:4,4,6");
        assert_eq!(decoded.score.bars().len(), 2);
        assert_eq!(decoded.issues, []);
    }

    #[test]
    fn reads_a_bar_of_at_most_512_leaves() {
        let full = decode("#v=2&b=8/4:64");
        assert_eq!(
            full.score.bars()[0].leaves().len(),
            crate::bar::MAX_LEAVES_PER_BAR as usize
        );
        assert_eq!(full.issues, []);
        let over = decode("#v=2&b=9/4:64,64,64,64,64,64,64,64,1");
        assert_eq!(over.score.bars(), [Bar::sixteenths()]);
        let issues: Vec<_> = over.issues.iter().map(ToString::to_string).collect();
        assert_eq!(
            issues,
            [
                "'9/4:64,64,64,64,64,64,64,64,1' is not a bar (expected like 4/4:4); replaced with 4/4 in 16ths"
            ]
        );
    }

    #[test]
    fn defaults_to_one_bar_of_4_4() {
        for input in ["#v=2", "#v=2&b=", "#v=2&b=;;"] {
            let decoded = decode(input);
            assert_eq!(
                decoded.score.bars(),
                [Bar::parse("4/4:4").unwrap()],
                "{input}"
            );
            assert_eq!(decoded.issues, [], "{input}");
        }
    }

    #[test]
    fn starts_empty_without_a_groove() {
        let decoded = decode("https://example.com/");
        assert_eq!(decoded.score.bars(), [Bar::parse("4/4:4").unwrap()]);
        assert_eq!(decoded.issues, [Issue::NoGroove]);
    }

    #[test]
    fn does_not_import_groove_scribe_links() {
        let decoded = decode("?TimeSig=4/4&Div=16&H=|x-x-|");
        assert_eq!(decoded.score.bars(), [Bar::parse("4/4:4").unwrap()]);
        assert_eq!(decoded.issues, [Issue::GrooveScribeNotImported]);
        assert_eq!(
            decoded.issues[0].to_string(),
            "Groove Scribe link not imported; starting empty"
        );
    }

    #[test]
    fn replaces_a_bad_bar() {
        let decoded = decode("#v=2&b=4/4;3/4:4");
        assert_eq!(
            decoded.score.bars(),
            [Bar::parse("4/4:4").unwrap(), Bar::parse("3/4:4").unwrap()]
        );
        let issues: Vec<_> = decoded
            .issues
            .iter()
            .map(|i| (i.level(), i.to_string()))
            .collect();
        assert_eq!(
            issues,
            [(
                Level::Error,
                "'4/4' is not a bar (expected like 4/4:4); replaced with 4/4 in 16ths".to_string()
            )]
        );
    }

    #[test]
    fn keeps_the_last_value_of_a_repeated_field() {
        assert_eq!(
            decode("#v=2&b=3/4:4&b=4/4:4").score.bars(),
            [Bar::parse("4/4:4").unwrap()]
        );
    }

    #[test]
    fn trims_the_input() {
        for input in [" #v=2&b=7/8:4,4,6\n", "\u{feff}v=2&b=7/8:4,4,6"] {
            let decoded = decode(input);
            assert_eq!(
                decoded.score.bars(),
                [Bar::parse("7/8:4,4,6").unwrap()],
                "{input:?}"
            );
            assert_eq!(decoded.issues, [], "{input:?}");
        }
    }

    #[test]
    fn keeps_what_javascript_trim_keeps() {
        assert_eq!(decode("\u{85}v=2&b=7/8:4,4,6").issues, [Issue::NoGroove]);
    }

    #[test]
    fn finds_the_groove_in_a_url() {
        for (url, body) in [
            ("https://x/#v=2&b=4/4:4", "v=2&b=4/4:4"),
            ("https://x/?TimeSig=4/4#top", "TimeSig=4/4"),
            ("https://x/?v=2&q=1", "v=2&q=1"),
            ("v=2&b=4/4:4", "v=2&b=4/4:4"),
        ] {
            assert_eq!(groove_body(url), body, "{url}");
        }
    }
}

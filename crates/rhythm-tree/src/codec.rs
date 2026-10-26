//! The v2 link format.

use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::bar::{Bar, MAX_LEAVES_PER_BAR, int};
use crate::issue::{Fit, Issue};
use crate::score::{KNOWN_KEYS, Score, is_lane_key, lane};

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
            score: Score::new(vec![Bar::sixteenths()], BTreeMap::new()),
            issues: vec![issue],
        };
    }
    let mut issues = vec![];
    let bars = decode_bars(
        &safe_decode(value(&params, "b").unwrap_or("4/4:4")),
        &mut issues,
    );
    let mut hits = BTreeMap::new();
    for (key, raw) in params.iter().filter(|(key, _)| !KNOWN_KEYS.contains(key)) {
        if is_lane_key(key) {
            hits.insert(key.to_string(), decode_lane(key, raw, &bars, &mut issues));
        } else {
            issues.push(Issue::UnknownField {
                key: key.to_string(),
            });
        }
    }
    Decoded {
        score: Score::new(bars, hits),
        issues,
    }
}

/// What JavaScript's `trim`, as the reference uses, strips: Unicode White_Space but U+0085, and U+FEFF.
fn is_js_whitespace(c: char) -> bool {
    (c.is_whitespace() && c != '\u{85}') || c == '\u{feff}'
}

/// One lane's `;`-separated bars of hits, one string per bar.
fn decode_lane(key: &str, raw: &str, bars: &[Bar], issues: &mut Vec<Issue>) -> Vec<String> {
    let allowed = lane(key).map(|lane| lane.chars);
    if allowed.is_none() {
        issues.push(Issue::CustomLane {
            lane: key.to_string(),
        });
    }
    let raw = safe_decode(raw);
    let parts: Vec<&str> = raw.split(';').collect();
    if parts.len() > bars.len() {
        issues.push(Issue::ExtraBars {
            lane: key.to_string(),
            parts: parts.len(),
            bars: bars.len(),
        });
    }
    let mut strs: Vec<String> = Vec::with_capacity(bars.len());
    for (b, bar) in bars.iter().enumerate() {
        let at = LaneBar {
            lane: key,
            bar: b,
            slots: bar.leaves().len(),
            allowed,
        };
        let prev = strs.last().map_or("", String::as_str);
        let str = decode_lane_bar(&at, parts.get(b).copied().unwrap_or_default(), prev, issues);
        strs.push(str);
    }
    strs
}

/// Where a bar of hits sits: its lane, its bar, how many slots it fills and the letters it may use.
struct LaneBar<'a> {
    lane: &'a str,
    bar: usize,
    slots: usize,
    allowed: Option<&'static str>,
}

/// One bar of one lane: `.` repeats `prev`, rest runs expand, letters the lane doesn't have become
/// rests, and the hits are fitted to the slots.
fn decode_lane_bar(at: &LaneBar, part: &str, prev: &str, issues: &mut Vec<Issue>) -> String {
    let mut str = if part == "." {
        prev.to_string()
    } else {
        unrle(part)
    };
    if let Some(allowed) = at.allowed {
        let is_letter = |c: char| c == '-' || allowed.contains(c);
        let bad: String = str.chars().filter(|&c| !is_letter(c)).collect();
        if !bad.is_empty() {
            issues.push(Issue::BadLetters {
                lane: at.lane.to_string(),
                bar: at.bar,
                letters: bad,
            });
            str = str
                .chars()
                .map(|c| if is_letter(c) { c } else { '-' })
                .collect();
        }
    }
    let hits = str.chars().count();
    if hits != at.slots && !part.is_empty() {
        issues.push(Issue::HitCount {
            lane: at.lane.to_string(),
            bar: at.bar,
            hits,
            slots: at.slots,
            fit: if hits > at.slots {
                Fit::Cut
            } else {
                Fit::Padded
            },
        });
    }
    str.chars()
        .chain(std::iter::repeat('-'))
        .take(at.slots)
        .collect()
}

/// Expands each `-N` into N rests, N capped at `MAX_LEAVES_PER_BAR`.
fn unrle(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(c) = rest.chars().next() {
        rest = &rest[c.len_utf8()..];
        let digits = rest.len() - rest.trim_start_matches(|d: char| d.is_ascii_digit()).len();
        match int(&rest[..digits]) {
            Some(n) if c == '-' => {
                out.extend(std::iter::repeat_n('-', n.min(MAX_LEAVES_PER_BAR) as usize));
                rest = &rest[digits..];
            }
            _ => out.push(c),
        }
    }
    out
}

/// The `;`-separated bars, each one that can't be read replaced with 4/4 in 16ths, up to `MAX_BARS`.
fn decode_bars(raw: &str, issues: &mut Vec<Issue>) -> Vec<Bar> {
    let mut bars = vec![];
    for item in raw.split(';').filter(|item| !item.is_empty()) {
        if bars.len() == MAX_BARS {
            issues.push(Issue::TooManyBars);
            break;
        }
        bars.push(Bar::parse(item).unwrap_or_else(|e| {
            issues.push(Issue::BarReplaced(e));
            Bar::sixteenths()
        }));
    }
    if bars.is_empty() {
        bars.push(Bar::sixteenths());
    }
    bars
}

/// The most bars a groove holds.
pub(crate) const MAX_BARS: usize = 512;

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

/// Percent-decodes like `decodeURIComponent`; text it can't decode comes back as it is.
fn safe_decode(s: &str) -> Cow<'_, str> {
    if !s.contains('%') {
        return Cow::Borrowed(s);
    }
    match unescape(s.as_bytes()).map(String::from_utf8) {
        Some(Ok(decoded)) => Cow::Owned(decoded),
        _ => Cow::Borrowed(s),
    }
}

/// The bytes `%XX` escapes stand for, or `None` when an escape is malformed.
fn unescape(b: &[u8]) -> Option<Vec<u8>> {
    let hex = |at: usize| Some(char::from(*b.get(at)?).to_digit(16)? as u8);
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' {
            out.push(hex(i + 1)? * 16 + hex(i + 2)?);
            i += 3;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    Some(out)
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
            ["513 slots in one bar is too many; replaced with 4/4 in 16ths"]
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
    fn reports_unknown_fields() {
        let decoded = decode("#v=2&b=4/4:4&Weird=1");
        assert_eq!(
            decoded.issues,
            [Issue::UnknownField {
                key: "Weird".to_string()
            }]
        );
        assert_eq!(
            decoded.issues[0].to_string(),
            "unknown field 'Weird' ignored"
        );
    }

    #[test]
    fn caps_the_bar_count() {
        let decoded = decode(&format!("#v=2&b={}", "4/4:4;".repeat(513)));
        assert_eq!(decoded.score.bars().len(), 512);
        assert_eq!(decoded.issues, [Issue::TooManyBars]);
        assert_eq!(
            decoded.issues[0].to_string(),
            "more than 512 bars; kept the first 512"
        );
    }

    #[test]
    fn reads_percent_encoded_bars() {
        let decoded = decode("#v=2&b=4/4:4%3B3/4:3");
        assert_eq!(
            decoded.score.bars(),
            [Bar::parse("4/4:4").unwrap(), Bar::parse("3/4:3").unwrap()]
        );
        assert_eq!(decoded.issues, []);
    }

    #[test]
    fn decodes_percent_escapes_safely() {
        for (s, decoded) in [
            ("%E2%99%A9%3D96", "\u{2669}=96"),
            ("100%", "100%"),
            ("%E2%80", "%E2%80"),
            ("a+b", "a+b"),
        ] {
            assert_eq!(safe_decode(s), decoded, "{s}");
        }
    }

    fn hits(decoded: &Decoded, lane: &str) -> Vec<String> {
        decoded.score.hits(lane).unwrap_or_default().to_vec()
    }

    fn messages(decoded: &Decoded) -> Vec<String> {
        decoded.issues.iter().map(Issue::to_string).collect()
    }

    #[test]
    fn reads_lane_hits() {
        let decoded = decode("#v=2&b=4/4:4&hh=x-x-x-x-x-x-x-x-");
        assert_eq!(hits(&decoded, "hh"), ["x-x-x-x-x-x-x-x-"]);
        assert_eq!(messages(&decoded), [] as [String; 0]);
    }

    #[test]
    fn expands_rest_runs() {
        let decoded = decode("#v=2&b=4/4:4&sn=-4o-11");
        assert_eq!(hits(&decoded, "sn"), ["----o-----------"]);
        assert_eq!(messages(&decoded), [] as [String; 0]);
    }

    #[test]
    fn repeats_the_previous_bar() {
        let decoded = decode("#v=2&b=4/4:4;4/4:4&hh=x-x-x-x-x-x-x-x-;.");
        assert_eq!(hits(&decoded, "hh"), ["x-x-x-x-x-x-x-x-"; 2]);
        assert_eq!(messages(&decoded), [] as [String; 0]);
    }

    #[test]
    fn turns_bad_letters_into_rests() {
        let decoded = decode("#v=2&b=4/4:4&sn=ozz-");
        assert_eq!(hits(&decoded, "sn"), [format!("o{}", "-".repeat(15))]);
        assert_eq!(
            messages(&decoded),
            [
                "sn bar 0: 'zz' not valid; made rests",
                "sn bar 0: 4 hits for 16 slots; padded"
            ]
        );
        assert!(decoded.issues.iter().all(|i| i.level() == Level::Warn));
    }

    #[test]
    fn pads_and_cuts_in_field_order() {
        let decoded = decode("#v=2&b=4/4:4&kk=o-o&sn=o---o---o---o---o---");
        assert_eq!(hits(&decoded, "kk"), [format!("o-o{}", "-".repeat(13))]);
        assert_eq!(hits(&decoded, "sn"), ["o---o---o---o---"]);
        assert_eq!(
            messages(&decoded),
            [
                "kk bar 0: 3 hits for 16 slots; padded",
                "sn bar 0: 20 hits for 16 slots; cut"
            ]
        );
    }

    #[test]
    fn reports_extra_bars() {
        let decoded = decode("#v=2&b=4/4:4&hh=x;x");
        assert_eq!(hits(&decoded, "hh"), [format!("x{}", "-".repeat(15))]);
        assert_eq!(
            messages(&decoded),
            [
                "hh: hits for 2 bars, groove has 1",
                "hh bar 0: 1 hits for 16 slots; padded"
            ]
        );
    }

    #[test]
    fn keeps_custom_lanes() {
        let decoded = decode("#v=2&b=4/4:4&zz9=x-x");
        assert_eq!(hits(&decoded, "zz9"), [format!("x-x{}", "-".repeat(13))]);
        assert_eq!(
            messages(&decoded),
            [
                "custom lane 'zz9' kept as-is",
                "zz9 bar 0: 3 hits for 16 slots; padded"
            ]
        );
        let decoded = decode("#v=2&b=4/4:4&zz9=x&hh=x&ab1=x");
        assert_eq!(decoded.score.lanes(), ["hh", "ab1", "zz9"]);
    }

    #[test]
    fn caps_rest_runs() {
        let decoded = decode("#v=2&b=4/4:4&hh=-9999");
        assert_eq!(hits(&decoded, "hh"), ["-".repeat(16)]);
        assert_eq!(messages(&decoded), ["hh bar 0: 512 hits for 16 slots; cut"]);
    }

    #[test]
    fn reads_percent_encoded_hits() {
        let decoded = decode("#v=2&b=4/4:4;4/4:4&hh=x-15%3B.");
        let bar = format!("x{}", "-".repeat(15));
        assert_eq!(hits(&decoded, "hh"), [bar.clone(), bar]);
        assert_eq!(messages(&decoded), [] as [String; 0]);
    }

    #[test]
    fn reserved_keys_are_not_lanes() {
        let decoded = decode("#v=2&b=4/4:4&sw=20&tc=x&ov=y");
        assert_eq!(
            messages(&decoded),
            [
                "unknown field 'sw' ignored",
                "unknown field 'tc' ignored",
                "unknown field 'ov' ignored"
            ]
        );
        assert_eq!(decoded.score.lanes(), [] as [&str; 0]);
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

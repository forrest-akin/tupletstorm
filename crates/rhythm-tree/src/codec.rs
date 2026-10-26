//! The v2 link format.

use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::bar::{Bar, MAX_LEAVES_PER_BAR};
use crate::issue::{Fit, Issue};
use crate::node::int;
use crate::score::{KNOWN_KEYS, Score, TempoChange, is_lane_key, lane};

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
            score: Score::new(
                String::new(),
                String::new(),
                String::new(),
                DEFAULT_TEMPO,
                0.0,
                vec![Bar::sixteenths()],
                vec![],
            ),
            issues: vec![issue],
        };
    }
    let text = |key| safe_decode(value(&params, key).unwrap_or_default()).into_owned();
    let (title, author, comments) = (text("t"), text("a"), text("c"));
    let mut issues = vec![];
    let tempo = decode_tempo(value(&params, "q"), &mut issues);
    let swing = decode_swing(value(&params, "sw"), &mut issues);
    let bars = decode_bars(
        &safe_decode(value(&params, "b").unwrap_or("4/4:4")),
        &mut issues,
    );
    let tempo_changes = decode_tempo_changes(value(&params, "tc"), bars.len(), &mut issues);
    let mut score = Score::new(title, author, comments, tempo, swing, bars, tempo_changes);
    let mut hits = BTreeMap::new();
    for (key, raw) in params.iter().filter(|(key, _)| !KNOWN_KEYS.contains(key)) {
        if is_lane_key(key) {
            hits.insert(key.to_string(), decode_lane(key, raw, &score, &mut issues));
        } else {
            issues.push(Issue::UnknownField {
                key: key.to_string(),
            });
        }
    }
    score.set_lanes(hits);
    Decoded { score, issues }
}

/// What JavaScript's `trim` and `Number()`, as the reference uses them, strip: Unicode White_Space but U+0085, and U+FEFF.
fn is_js_whitespace(c: char) -> bool {
    (c.is_whitespace() && c != '\u{85}') || c == '\u{feff}'
}

/// Writes a score as canonical v2 link text, without the leading `#`.
pub fn encode(score: &Score) -> String {
    let mut fields = vec!["v=2".to_string()];
    for (key, text) in [
        ("t", score.title()),
        ("a", score.author()),
        ("c", score.comments()),
    ] {
        if !text.is_empty() {
            fields.push(format!("{key}={}", percent_encode(text)));
        }
    }
    fields.push(format!("q={}", score.tempo()));
    if score.swing() != 0.0 {
        fields.push(format!("sw={}", score.swing()));
    }
    fields.push(format!("b={}", encode_bars(score.bars())));
    if !score.tempo_changes().is_empty() {
        let items: Vec<String> = score
            .tempo_changes()
            .iter()
            .map(ToString::to_string)
            .collect();
        fields.push(format!("tc={}", items.join(";")));
    }
    for (id, hits) in score
        .lanes()
        .into_iter()
        .filter_map(|id| Some((id, score.hits(id)?)))
    {
        let lane = encode_lane(hits);
        if !lane.is_empty() {
            fields.push(format!("{id}={lane}"));
        }
    }
    fields.join("&")
}

/// Escapes each byte of `s` as `%XX` except ``A-Za-z0-9-_.!~*'()``, like `encodeURIComponent`.
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&b) {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Bars separated by `;`, a run of identical bars written once with `*k`.
fn encode_bars(bars: &[Bar]) -> String {
    let specs: Vec<String> = bars.iter().map(Bar::to_string).collect();
    specs
        .chunk_by(|a, b| a == b)
        .map(|run| match run.len() {
            1 => run[0].clone(),
            k => format!("{}*{k}", run[0]),
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// One lane's bars separated by `;`: an all-rest bar empty, a bar equal to the one before `.`,
/// trailing empty bars dropped, and custom-lane letters that are link syntax percent-encoded.
fn encode_lane(hits: &[String]) -> String {
    let mut parts: Vec<String> = hits
        .iter()
        .enumerate()
        .map(|(b, bar)| {
            if bar.chars().all(|c| c == '-') {
                String::new()
            } else if b > 0 && *bar == hits[b - 1] {
                ".".to_string()
            } else {
                escape_link_syntax(&rle(bar))
            }
        })
        .collect();
    while parts.last().is_some_and(String::is_empty) {
        parts.pop();
    }
    parts.join(";")
}

/// Percent-encodes `%`, `&`, `=`, `#` and `?`, which would otherwise end or garble the field, and
/// the whitespace `decode` trims from the link's ends. The rest stays as it is: `+` is a hi-hat letter.
fn escape_link_syntax(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if matches!(c, '%' | '&' | '=' | '#' | '?') || is_js_whitespace(c) {
            for b in c.encode_utf8(&mut [0; 4]).bytes() {
                out.push_str(&format!("%{b:02X}"));
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Quarter notes per minute: a decimal number from 10 to 999, else 80.
fn decode_tempo(raw: Option<&str>, issues: &mut Vec<Issue>) -> f64 {
    let Some(raw) = raw else {
        return DEFAULT_TEMPO;
    };
    match safe_decode(raw)
        .trim_matches(is_js_whitespace)
        .parse::<f64>()
    {
        Ok(q) if (10.0..=999.0).contains(&q) => q,
        _ => {
            issues.push(Issue::TempoOutOfRange {
                raw: raw.to_string(),
            });
            DEFAULT_TEMPO
        }
    }
}

const DEFAULT_TEMPO: f64 = 80.0;

/// Swing: a number from 0 to 100, surrounding whitespace ignored; 0 when absent or blank.
fn decode_swing(raw: Option<&str>, issues: &mut Vec<Issue>) -> f64 {
    let raw = raw.unwrap_or_default();
    let trimmed = raw.trim_matches(is_js_whitespace);
    if trimmed.is_empty() {
        return 0.0;
    }
    match trimmed.parse::<f64>() {
        Ok(sw) if (0.0..=100.0).contains(&sw) => sw,
        _ => {
            issues.push(Issue::SwingOutOfRange {
                raw: raw.to_string(),
            });
            0.0
        }
    }
}

/// One lane's `;`-separated bars of hits, one string per bar.
fn decode_lane(key: &str, raw: &str, score: &Score, issues: &mut Vec<Issue>) -> Vec<String> {
    let bars = score.bars();
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
    for b in 0..bars.len() {
        let at = LaneBar {
            lane: key,
            bar: b,
            slots: score.lane_leaves(key, b).len(),
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
/// rests, the hits are fitted to the slots, and a custom lane's bar of just `.` becomes a rest.
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
    let fitted: String = str
        .chars()
        .chain(std::iter::repeat('-'))
        .take(at.slots)
        .collect();
    // A lone `.` (only a custom lane can keep one) would be written as `.`, a repeat.
    if fitted == "." {
        issues.push(Issue::BadLetters {
            lane: at.lane.to_string(),
            bar: at.bar,
            letters: fitted,
        });
        return "-".to_string();
    }
    fitted
}

/// Writes each run of three or more rests as `-N`.
fn rle(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(c) = rest.chars().next() {
        let run = rest.len() - rest.trim_start_matches('-').len();
        match run {
            0 => out.push(c),
            1 | 2 => out.push_str(&rest[..run]),
            _ => out.push_str(&format!("-{run}")),
        }
        rest = &rest[run.max(c.len_utf8())..];
    }
    out
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

/// The `;`-separated tempo changes, each one that can't be read dropped.
fn decode_tempo_changes(
    raw: Option<&str>,
    bars: usize,
    issues: &mut Vec<Issue>,
) -> Vec<TempoChange> {
    let raw = safe_decode(raw.unwrap_or_default());
    raw.split(';')
        .filter(|item| !item.is_empty())
        .filter_map(|item| {
            TempoChange::parse(item, bars)
                .map_err(|e| issues.push(Issue::TempoChangeIgnored(e)))
                .ok()
        })
        .collect()
}

/// The `;`-separated bars, each one that can't be read replaced with 4/4 in 16ths, up to `MAX_BARS`.
fn decode_bars(raw: &str, issues: &mut Vec<Issue>) -> Vec<Bar> {
    let mut bars = vec![];
    'items: for item in raw.split(';').filter(|item| !item.is_empty()) {
        let (bar, count) = match Bar::parse_item(item) {
            Ok((bar, 0)) => {
                issues.push(Issue::RepeatZero {
                    spec: item.to_string(),
                });
                (bar, 1)
            }
            Ok(repeated) => repeated,
            Err(e) => {
                issues.push(Issue::BarReplaced(e));
                (Bar::sixteenths(), 1)
            }
        };
        for _ in 0..count {
            if bars.len() == MAX_BARS {
                issues.push(Issue::TooManyBars);
                break 'items;
            }
            bars.push(bar.clone());
        }
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
        let decoded = decode("#v=2&b=4/4:4&ov=y");
        assert_eq!(messages(&decoded), ["unknown field 'ov' ignored"]);
        assert_eq!(decoded.score.lanes(), [] as [&str; 0]);
    }

    #[test]
    fn reads_the_title() {
        let decoded = decode("#v=2&t=A%20%26%20B");
        assert_eq!(decoded.score.title(), "A & B");
        assert_eq!(decoded.issues, []);
    }

    #[test]
    fn reads_tempo() {
        let decoded = decode("#v=2&q=92.5");
        assert_eq!(decoded.score.tempo(), 92.5);
        assert_eq!(decoded.issues, []);
        let decoded = decode("#v=2");
        assert_eq!(decoded.score.tempo(), 80.0);
        assert_eq!(decoded.issues, []);
        let decoded = decode("#v=2&q=%EF%BB%BF90");
        assert_eq!(decoded.score.tempo(), 90.0);
        assert_eq!(decoded.issues, []);
    }

    #[test]
    fn reports_a_bad_tempo() {
        for (q, msg) in [
            ("abc", "tempo 'abc' out of range; using 80"),
            ("5", "tempo '5' out of range; using 80"),
            ("0x5A", "tempo '0x5A' out of range; using 80"),
            ("%C2%8590", "tempo '%C2%8590' out of range; using 80"),
        ] {
            let decoded = decode(&format!("#v=2&q={q}"));
            assert_eq!(decoded.score.tempo(), 80.0, "{q}");
            assert_eq!(messages(&decoded), [msg], "{q}");
        }
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

    fn reencode(input: &str) -> String {
        encode(&decode(input).score)
    }

    #[test]
    fn writes_compact_fields() {
        let hh = ["x-x-x-x-x-x-x-x-"; 4].join(";");
        assert_eq!(
            reencode(&format!("#v=2&b=4/4:4;4/4:4;4/4:4;4/4:4&hh={hh}")),
            "v=2&q=80&b=4/4:4*4&hh=x-x-x-x-x-x-x-x-;.;.;."
        );
    }

    #[test]
    fn percent_encodes_free_text() {
        for (title, encoded) in [
            ("Bars 1–2", "Bars%201%E2%80%932"),
            ("a+b&c=d", "a%2Bb%26c%3Dd"),
            ("(ok)!*'~-_.", "(ok)!*'~-_."),
        ] {
            let score = decode(&format!("#v=2&t={encoded}")).score;
            assert_eq!(score.title(), title);
            assert_eq!(
                encode(&score),
                format!("v=2&t={encoded}&q=80&b=4/4:4"),
                "{title}"
            );
        }
    }

    #[test]
    fn compresses_rest_runs() {
        assert_eq!(
            reencode("#v=2&b=5/4:4&hh=x-x--x---x----------"),
            "v=2&q=80&b=5/4:4&hh=x-x--x-3x-10"
        );
    }

    #[test]
    fn reads_bar_repeats() {
        for (bars, count, issues) in [
            ("4/4:4*3", 3, vec![]),
            (
                "4/4:4*0",
                1,
                vec!["bar '4/4:4*0' repeats 0 times; kept once"],
            ),
            (
                "4/4:4*99999999999",
                512,
                vec!["more than 512 bars; kept the first 512"],
            ),
            (
                "4/4*3",
                1,
                vec!["'4/4*3' is not a bar (expected like 4/4:4); replaced with 4/4 in 16ths"],
            ),
            (
                "5/8:*0",
                1,
                vec!["expected a number at 0 in '*0'; replaced with 4/4 in 16ths"],
            ),
            (
                "1/4~3-3:*3",
                1,
                vec!["grouping 3+3 doesn't add up to 1; replaced with 4/4 in 16ths"],
            ),
        ] {
            let decoded = decode(&format!("#v=2&b={bars}"));
            assert_eq!(decoded.score.bars().len(), count, "{bars}");
            assert_eq!(messages(&decoded), issues as Vec<&str>, "{bars}");
        }
    }

    #[test]
    fn drops_trailing_empty_bars() {
        for (input, encoded) in [
            ("#v=2&b=4/4:4;4/4:4&hh=x-15;", "v=2&q=80&b=4/4:4*2&hh=x-15"),
            ("#v=2&b=4/4:4&hh=-16", "v=2&q=80&b=4/4:4"),
        ] {
            assert_eq!(reencode(input), encoded, "{input}");
        }
    }

    #[test]
    fn round_trips_author_and_comments() {
        let decoded = decode("#v=2&a=F.%20Akin&c=%E2%99%A9%3D96");
        assert_eq!(
            (decoded.score.author(), decoded.score.comments()),
            ("F. Akin", "\u{2669}=96")
        );
        assert_eq!(decoded.issues, []);
        assert_eq!(
            encode(&decoded.score),
            "v=2&a=F.%20Akin&c=%E2%99%A9%3D96&q=80&b=4/4:4"
        );
    }

    #[test]
    fn escapes_link_syntax_in_custom_lanes() {
        for (input, encoded) in [
            ("#v=2&b=4/4:4&zz9=a%26b=1", "zz9=a%26b%3D1-11"),
            ("#v=2&b=4/4:4&zz9=%2541", "zz9=%2541-13"),
            ("#v=2&b=4/4:4&zz9=%23x%3F", "zz9=%23x%3F-13"),
        ] {
            let once = encode(&decode(input).score);
            assert!(once.ends_with(encoded), "{input}: {once}");
            assert_eq!(encode(&decode(&once).score), once, "{input}");
        }
    }

    #[test]
    fn escapes_whitespace_in_custom_lanes() {
        for (input, encoded) in [
            ("#v=2&b=1/4:1&zz9=%0A", "zz9=%0A"),
            ("#v=2&b=1/4:1&zz9=%20", "zz9=%20"),
            ("#v=2&b=1/4:1&zz9=%E3%80%80", "zz9=%E3%80%80"),
            ("#v=2&b=1/4:1&zz9=%EF%BB%BF", "zz9=%EF%BB%BF"),
        ] {
            let once = encode(&decode(input).score);
            assert!(once.ends_with(encoded), "{input}: {once:?}");
            assert_eq!(encode(&decode(&once).score), once, "{input}");
        }
    }

    #[test]
    fn reads_a_lone_dot_in_a_custom_lane_as_a_rest() {
        let decoded = decode("#v=2&b=1/4:1;1/4:1&zz9=x;.x");
        assert_eq!(
            decoded.score.hits("zz9"),
            Some(&["x".to_string(), "-".to_string()][..])
        );
        assert_eq!(
            messages(&decoded),
            [
                "custom lane 'zz9' kept as-is",
                "zz9 bar 1: 2 hits for 1 slots; cut",
                "zz9 bar 1: '.' not valid; made rests",
            ]
        );
        let once = encode(&decoded.score);
        assert_eq!(encode(&decode(&once).score), once);
    }

    #[test]
    fn reads_swing() {
        for (input, encoded) in [
            ("#v=2&sw=20", "v=2&q=80&sw=20&b=4/4:4"),
            ("#v=2&sw=", "v=2&q=80&b=4/4:4"),
            ("#v=2&sw= 20", "v=2&q=80&sw=20&b=4/4:4"),
            ("#v=2&sw= &b=4/4:4", "v=2&q=80&b=4/4:4"),
            ("#v=2&sw=\u{feff}20", "v=2&q=80&sw=20&b=4/4:4"),
        ] {
            let decoded = decode(input);
            assert_eq!(decoded.issues, [], "{input}");
            assert_eq!(encode(&decoded.score), encoded, "{input}");
        }
    }

    #[test]
    fn reports_a_bad_swing() {
        for (sw, msg) in [
            ("101", "swing '101' out of range; using 0"),
            ("\u{85}20", "swing '\u{85}20' out of range; using 0"),
        ] {
            let decoded = decode(&format!("#v=2&sw={sw}"));
            assert_eq!(decoded.score.swing(), 0.0, "{sw}");
            assert_eq!(messages(&decoded), [msg], "{sw}");
        }
    }

    #[test]
    fn reads_tempo_changes() {
        let decoded = decode("#v=2&b=4/4:4;6/8:6&tc=1:3/2~1");
        assert_eq!(messages(&decoded), [] as [String; 0]);
        assert_eq!(encode(&decoded.score), "v=2&q=80&b=4/4:4;6/8:6&tc=1:3/2~1");
    }

    #[test]
    fn rejects_bad_tempo_changes() {
        let rows = [
            ("x:1~1", "tempo change ignored: 'x:1~1' is not bar:unit~old"),
            ("1:1/0~1", "tempo change ignored: bad rational 1/0"),
            ("1:1/2/3~1", "tempo change ignored: bad fraction '1/2/3'"),
            ("1:0~1", "tempo change ignored: 0 is not a note value"),
            ("9:3/2~1", "tempo change ignored: bar 9 doesn't exist"),
        ];
        for (tc, msg) in rows {
            let decoded = decode(&format!("#v=2&b=4/4:4;6/8:6&tc={tc}"));
            assert_eq!(messages(&decoded), [msg], "{tc}");
            assert_eq!(encode(&decoded.score), "v=2&q=80&b=4/4:4;6/8:6", "{tc}");
        }
    }
}

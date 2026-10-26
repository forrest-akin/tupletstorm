//! Properties of decoding and encoding that hold for any input.

use proptest::prelude::*;
use rhythm_tree::{Bar, Decoded, Rat, decode, encode};

const CASES: u32 = 2048;

/// The leaves start at 0, each where the last ends, and end where the bar does.
fn tiles(bar: &Bar) -> bool {
    let mut at = Rat::new(0, 1);
    for leaf in bar.leaves() {
        if leaf.start != at || leaf.span.num() <= 0 {
            return false;
        }
        at = at + leaf.span;
    }
    at == bar.length()
}

/// Decodes on a thread with a 1 MiB stack, the wasm default.
fn decode_on_wasm_stack(s: String) -> Decoded {
    std::thread::Builder::new()
        .stack_size(1 << 20)
        .spawn(move || decode(&s))
        .unwrap()
        .join()
        .unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(CASES))]

    #[test]
    fn any_text_decodes(s in "[ -~]{0,160}") {
        let decoded = decode_on_wasm_stack(s);
        prop_assert!(decoded.score.bars().iter().all(tiles));
    }

    #[test]
    fn grammar_shaped_text_decodes(tail in "[0-9/:~,()@*;-]{0,40}") {
        let decoded = decode_on_wasm_stack(format!("#v=2&b={tail}"));
        prop_assert!(decoded.score.bars().iter().all(tiles));
    }

    #[test]
    fn any_text_reencodes_stably(s in "[ -~]{0,160}") {
        let once = encode(&decode(&s).score);
        prop_assert_eq!(encode(&decode(&once).score), once);
    }

    #[test]
    fn grammar_shaped_text_reencodes_stably(tail in "[0-9/:~,()@*;-]{0,40}") {
        let once = encode(&decode(&format!("#v=2&b={tail}")).score);
        prop_assert_eq!(encode(&decode(&once).score), once);
    }

    #[test]
    fn scores_survive_a_round_trip(link in groove()) {
        let decoded = decode(&link);
        prop_assert!(decoded.issues.is_empty(), "{:?}", decoded.issues);
        let e = encode(&decoded.score);
        let again = decode(&e).score;
        prop_assert_eq!(encode(&again), e.clone());
        for lane in ["hh", "sn", "kk"] {
            let hits = decoded.score.hits(lane).unwrap();
            if hits.iter().any(|bar| bar.chars().any(|c| c != '-')) {
                prop_assert_eq!(again.hits(lane), Some(hits), "{}", lane);
            }
        }
    }

    #[test]
    fn custom_lane_text_survives_a_round_trip((link, lane) in custom_lane()) {
        let decoded = decode(&link);
        let once = encode(&decoded.score);
        let again = decode(&once).score;
        prop_assert_eq!(encode(&again), once.clone());
        let read: Vec<String> = lane
            .iter()
            .map(|bar| if bar == "." { "-".to_string() } else { bar.clone() })
            .collect();
        prop_assert_eq!(decoded.score.hits("zz9"), Some(&read[..]));
        let kept = read.iter().any(|bar| bar.chars().any(|c| c != '-'));
        prop_assert_eq!(again.hits("zz9"), kept.then_some(&read[..]), "{}", once);
    }

    #[test]
    fn leaves_tile_the_bar(spec in bar_spec()) {
        let bar = Bar::parse(&spec).unwrap();
        prop_assert!(tiles(&bar), "{spec}");
    }
}

const METERS: [(u32, u32); 8] = [
    (4, 4),
    (3, 4),
    (5, 4),
    (7, 8),
    (5, 8),
    (6, 8),
    (9, 8),
    (11, 16),
];

/// A bar like `7/8:3,1,8`: one split 1–8 per default beat group.
fn bar_spec() -> impl Strategy<Value = String> {
    proptest::sample::select(&METERS[..]).prop_flat_map(|(num, den)| {
        let groups = Bar::parse(&format!("{num}/{den}:1"))
            .unwrap()
            .leaves()
            .len();
        proptest::collection::vec(1u32..=8, groups).prop_map(move |splits| {
            let splits: Vec<_> = splits.iter().map(u32::to_string).collect();
            format!("{num}/{den}:{}", splits.join(","))
        })
    })
}

/// Link text for 1–4 bars with a title, a tempo and every leaf of `hh`, `sn` and `kk` written out.
fn groove() -> impl Strategy<Value = String> {
    (
        proptest::collection::vec(bar_spec(), 1..=4),
        30u32..=300,
        "[A-Za-z]{0,12}",
    )
        .prop_flat_map(|(bars, tempo, title)| {
            let slots: Vec<usize> = bars
                .iter()
                .map(|bar| Bar::parse(bar).unwrap().leaves().len())
                .collect();
            (
                Just(format!("#v=2&t={title}&q={tempo}&b={}", bars.join(";"))),
                lane_hits("-xXo+", &slots),
                lane_hits("-oOgxfdb", &slots),
                lane_hits("-oO", &slots),
            )
        })
        .prop_map(|(head, hh, sn, kk)| format!("{head}&hh={hh}&sn={sn}&kk={kk}"))
}

/// One letter from `letters` per slot, bars separated by `;`.
fn lane_hits(letters: &str, slots: &[usize]) -> impl Strategy<Value = String> + use<> {
    let letters: Vec<char> = letters.chars().collect();
    slots
        .iter()
        .map(|&n| {
            proptest::collection::vec(proptest::sample::select(letters.clone()), n)
                .prop_map(String::from_iter)
        })
        .collect::<Vec<_>>()
        .prop_map(|bars| bars.join(";"))
}

/// Link text for 1–4 bars and a custom lane `zz9` of one `CUSTOM_LANE_CHARS` character per slot,
/// written fully percent-encoded, with the lane's bars as written.
fn custom_lane() -> impl Strategy<Value = (String, Vec<String>)> {
    let bar = prop_oneof![Just("1/4:1".to_string()), bar_spec()];
    proptest::collection::vec(bar, 1..=4)
        .prop_flat_map(|bars| {
            let lane: Vec<_> = bars
                .iter()
                .map(|bar| {
                    let slots = Bar::parse(bar).unwrap().leaves().len();
                    proptest::collection::vec(
                        proptest::sample::select(&CUSTOM_LANE_CHARS[..]),
                        slots,
                    )
                    .prop_map(String::from_iter)
                })
                .collect();
            (Just(bars.join(";")), lane)
        })
        .prop_map(|(bars, lane)| {
            let parts: Vec<String> = lane
                .iter()
                .map(|bar| {
                    // A written `.` repeats the bar before; `.-` is cut to the lone `.` itself.
                    let bar = if bar == "." { ".-" } else { bar };
                    bar.bytes().map(|b| format!("%{b:02X}")).collect()
                })
                .collect();
            (format!("#v=2&b={bars}&zz9={}", parts.join(";")), lane)
        })
}

/// Link syntax, the whitespace `decode` trims, and lane letters.
const CUSTOM_LANE_CHARS: [char; 16] = [
    '&', '=', '%', '#', '?', '.', ' ', '\t', '\n', '\u{3000}', '\u{feff}', '-', 'x', 'X', 'o', '+',
];

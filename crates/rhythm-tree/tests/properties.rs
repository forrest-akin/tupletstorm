//! Properties of decoding that hold for any input.

use proptest::prelude::*;
use rhythm_tree::{Bar, Decoded, Rat, decode};

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

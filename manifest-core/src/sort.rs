use crate::model::ModFile;
use std::collections::{BTreeMap, BTreeSet};

/// Pinned mod names: `top` entries load first, `bottom` entries load last.
/// Relative order WITHIN a pin block follows the current load order, not the
/// order the names appear in the pins file (see `keep_order` in [`propose`]).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Pins {
    pub top: Vec<String>,
    pub bottom: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Move {
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SortResult {
    pub proposed: Vec<String>,
    pub moves: Vec<Move>,
}

pub fn propose(mods: &[ModFile], current: &[String], pins: &Pins) -> SortResult {
    let by_name: BTreeMap<&str, &ModFile> = mods
        .iter()
        .filter(|m| m.enabled && m.error.is_none())
        .map(|m| (m.name.as_str(), m))
        .collect();

    let mut working: Vec<String> = current
        .iter()
        .filter(|n| by_name.contains_key(n.as_str()))
        .cloned()
        .collect();
    let mut extras: Vec<String> = by_name
        .keys()
        .filter(|n| !working.iter().any(|w| w == *n))
        .map(|n| n.to_string())
        .collect();
    extras.sort();
    working.extend(extras);

    // Deduplicate working, keeping first occurrence (Finding 2 fix)
    {
        let mut seen = BTreeSet::new();
        working.retain(|n| seen.insert(n.clone()));
    }

    let is_pinned = |n: &str| pins.top.iter().any(|p| p == n) || pins.bottom.iter().any(|p| p == n);
    let index: BTreeMap<&str, usize> = working
        .iter()
        .enumerate()
        .map(|(i, n)| (n.as_str(), i))
        .collect();

    // Edges: precede[a] = set of b that must come after a.
    let mut successors: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    let mut indegree: BTreeMap<&str, usize> = working.iter().map(|n| (n.as_str(), 0)).collect();
    let mut blame: BTreeMap<&str, (&str, usize, usize)> = BTreeMap::new();

    let names: Vec<&str> = working.iter().map(|s| s.as_str()).collect();
    for i in 0..names.len() {
        for j in (i + 1)..names.len() {
            let (a, b) = (names[i], names[j]);
            if is_pinned(a) || is_pinned(b) {
                continue;
            }
            let (ma, mb) = (by_name[a], by_name[b]);
            if ma.assets.intersection(&mb.assets).next().is_none() {
                continue;
            }
            let (first, second) = match ma.assets.len().cmp(&mb.assets.len()) {
                std::cmp::Ordering::Greater => (a, b),
                std::cmp::Ordering::Less => (b, a),
                std::cmp::Ordering::Equal => continue,
            };
            if successors.entry(first).or_default().insert(second) {
                *indegree.get_mut(second).unwrap() += 1;
                let (fm, sm) = (by_name[first], by_name[second]);
                let entry =
                    blame
                        .entry(second)
                        .or_insert((first, sm.assets.len(), fm.assets.len()));
                if fm.assets.len() > entry.2 {
                    *entry = (first, sm.assets.len(), fm.assets.len());
                }
            }
        }
    }

    // Kahn, ready set ordered by current index for stability.
    let mut ready: BTreeSet<(usize, &str)> = indegree
        .iter()
        .filter(|(n, d)| **d == 0 && !is_pinned(n))
        .map(|(n, _)| (index[n], *n))
        .collect();
    let mut sorted_middle: Vec<String> = Vec::new();
    while let Some(&(i, n)) = ready.iter().next() {
        ready.remove(&(i, n));
        sorted_middle.push(n.to_string());
        if let Some(next) = successors.get(n) {
            for &s in next {
                let d = indegree.get_mut(s).unwrap();
                *d -= 1;
                if *d == 0 {
                    ready.insert((index[s], s));
                }
            }
        }
    }

    // Pin blocks keep the mods' current relative order; the pins file's list
    // order carries no meaning beyond membership.
    let keep_order = |list: &[String]| -> Vec<String> {
        let mut v: Vec<String> = working
            .iter()
            .filter(|n| list.iter().any(|p| p == *n))
            .cloned()
            .collect();
        v.dedup();
        v
    };
    let mut proposed = keep_order(&pins.top);
    proposed.extend(sorted_middle);

    // For bottom pins, exclude any name already in proposed (Finding 1 fix: top wins)
    let mut bottom_pins = keep_order(&pins.bottom);
    bottom_pins.retain(|n| !proposed.iter().any(|p| p == n));
    proposed.extend(bottom_pins);

    let mut moves = Vec::new();
    for (new_index, name) in proposed.iter().enumerate() {
        if index.get(name.as_str()) != Some(&new_index) {
            let reason = match blame.get(name.as_str()) {
                Some((other, mine, theirs)) => format!(
                    "moved after {other}: its {mine} assets overlap {other}'s {theirs} and the more specific mod wins"
                ),
                None => "repositioned to preserve relative order after conflict-driven moves".to_string(),
            };
            moves.push(Move {
                name: name.clone(),
                reason,
            });
        }
    }
    SortResult { proposed, moves }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ModFile;
    use std::collections::BTreeSet;

    fn mk(name: &str, assets: &[&str]) -> ModFile {
        ModFile {
            path: format!("/tmp/{name}.otr").into(),
            name: name.into(),
            enabled: true,
            assets: assets
                .iter()
                .map(|s| s.to_string())
                .collect::<BTreeSet<_>>(),
            error: None,
            gamebanana_mod_id: None,
        }
    }

    #[test]
    fn specific_mod_moves_after_broad_pack() {
        let mods = vec![mk("SwordSkin", &["a"]), mk("BigOverhaul", &["a", "b", "c"])];
        let current: Vec<String> = ["SwordSkin", "BigOverhaul"].map(String::from).into();
        let result = propose(&mods, &current, &Pins::default());
        assert_eq!(
            result.proposed,
            vec!["BigOverhaul".to_string(), "SwordSkin".to_string()]
        );
        assert_eq!(result.moves.len(), 2);
        assert!(result
            .moves
            .iter()
            .any(|m| m.name == "SwordSkin" && m.reason.contains("BigOverhaul")));
    }

    #[test]
    fn non_conflicting_mods_keep_relative_order() {
        let mods = vec![mk("Zeta", &["x"]), mk("Alpha", &["y"]), mk("Mid", &["z"])];
        let current: Vec<String> = ["Zeta", "Alpha", "Mid"].map(String::from).into();
        let result = propose(&mods, &current, &Pins::default());
        assert_eq!(result.proposed, current);
        assert!(result.moves.is_empty());
    }

    #[test]
    fn pins_override_heuristics() {
        let mods = vec![mk("SwordSkin", &["a"]), mk("BigOverhaul", &["a", "b", "c"])];
        let current: Vec<String> = ["SwordSkin", "BigOverhaul"].map(String::from).into();
        let pins = Pins {
            top: vec!["SwordSkin".into()],
            bottom: vec![],
        };
        let result = propose(&mods, &current, &pins);
        assert_eq!(
            result.proposed,
            vec!["SwordSkin".to_string(), "BigOverhaul".to_string()]
        );
    }

    #[test]
    fn deterministic_across_runs() {
        let mods = vec![
            mk("A", &["1", "2", "3", "4"]),
            mk("B", &["2", "3"]),
            mk("C", &["3"]),
            mk("D", &["9"]),
        ];
        let current: Vec<String> = ["C", "A", "D", "B"].map(String::from).into();
        let first = propose(&mods, &current, &Pins::default()).proposed;
        for _ in 0..10 {
            assert_eq!(propose(&mods, &current, &Pins::default()).proposed, first);
        }
        // A (4 assets) precedes B (2), B precedes C (1); D untouched.
        let idx = |n: &str| first.iter().position(|x| x == n).unwrap();
        assert!(idx("A") < idx("B"));
        assert!(idx("B") < idx("C"));
    }

    #[test]
    fn pins_no_duplicates_top_wins() {
        // Finding 1 regression test: mod listed in both top and bottom should appear exactly once at top
        let mods = vec![mk("A", &["x"]), mk("B", &["y"])];
        let current: Vec<String> = ["A", "B"].map(String::from).into();
        let pins = Pins {
            top: vec!["A".into()],
            bottom: vec!["A".into()],
        };
        let result = propose(&mods, &current, &pins);
        // Count occurrences of "A"
        let a_count = result.proposed.iter().filter(|n| *n == "A").count();
        assert_eq!(a_count, 1, "Mod A should appear exactly once");
        // "A" should be first (in top)
        assert_eq!(result.proposed[0], "A");
        // "B" should appear exactly once
        let b_count = result.proposed.iter().filter(|n| *n == "B").count();
        assert_eq!(b_count, 1, "Mod B should appear exactly once");
    }

    #[test]
    fn no_duplicates_in_current() {
        // Finding 2 regression test: duplicates in current should be deduplicated in proposed
        let mods = vec![mk("A", &["x"]), mk("B", &["y"])];
        let current: Vec<String> = ["A", "A", "B"].map(String::from).into();
        let result = propose(&mods, &current, &Pins::default());
        // Each mod should appear exactly once
        let a_count = result.proposed.iter().filter(|n| *n == "A").count();
        let b_count = result.proposed.iter().filter(|n| *n == "B").count();
        assert_eq!(a_count, 1, "Mod A should appear exactly once");
        assert_eq!(b_count, 1, "Mod B should appear exactly once");
        // proposed should have exactly 2 elements (A and B)
        assert_eq!(
            result.proposed.len(),
            2,
            "proposed should have exactly 2 mods"
        );
        // proposed should equal ["A", "B"]
        assert_eq!(result.proposed, vec!["A".to_string(), "B".to_string()]);
    }

    #[test]
    fn error_and_disabled_mods_are_excluded_from_proposal() {
        let mut broken = mk("Broken", &["a"]);
        broken.error = Some("no (listfile)".into());
        let mut parked = mk("Parked", &["b"]);
        parked.enabled = false;
        let mods = vec![mk("Fine", &["c"]), broken, parked];
        let current: Vec<String> = ["Broken", "Parked", "Fine"].map(String::from).into();
        let result = propose(&mods, &current, &Pins::default());
        assert_eq!(result.proposed, vec!["Fine".to_string()]);
    }

    #[test]
    fn mods_missing_from_current_append_alphabetically() {
        let mods = vec![mk("Zulu", &["z"]), mk("Alpha", &["a"]), mk("Kept", &["k"])];
        let current: Vec<String> = ["Kept"].map(String::from).into();
        let result = propose(&mods, &current, &Pins::default());
        assert_eq!(
            result.proposed,
            vec!["Kept".to_string(), "Alpha".to_string(), "Zulu".to_string()],
            "missing mods should append after current order, alphabetically"
        );
    }
}

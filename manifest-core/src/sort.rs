use crate::model::ModFile;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
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

    let is_pinned = |n: &str| pins.top.iter().any(|p| p == n) || pins.bottom.iter().any(|p| p == n);
    let index: BTreeMap<&str, usize> =
        working.iter().enumerate().map(|(i, n)| (n.as_str(), i)).collect();

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
                let entry = blame.entry(second).or_insert((first, sm.assets.len(), fm.assets.len()));
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
    proposed.extend(keep_order(&pins.bottom));

    let mut moves = Vec::new();
    for (new_index, name) in proposed.iter().enumerate() {
        if index.get(name.as_str()) != Some(&new_index) {
            let reason = match blame.get(name.as_str()) {
                Some((other, mine, theirs)) => format!(
                    "moved after {other}: its {mine} assets overlap {other}'s {theirs} and the more specific mod wins"
                ),
                None => "position shifted by other moves".to_string(),
            };
            moves.push(Move { name: name.clone(), reason });
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
            assets: assets.iter().map(|s| s.to_string()).collect::<BTreeSet<_>>(),
            error: None,
            gamebanana_mod_id: None,
        }
    }

    #[test]
    fn specific_mod_moves_after_broad_pack() {
        let mods = vec![mk("SwordSkin", &["a"]), mk("BigOverhaul", &["a", "b", "c"])];
        let current: Vec<String> = ["SwordSkin", "BigOverhaul"].map(String::from).into();
        let result = propose(&mods, &current, &Pins::default());
        assert_eq!(result.proposed, vec!["BigOverhaul".to_string(), "SwordSkin".to_string()]);
        assert_eq!(result.moves.len(), 2);
        assert!(result.moves.iter().any(|m| m.name == "SwordSkin" && m.reason.contains("BigOverhaul")));
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
        let pins = Pins { top: vec!["SwordSkin".into()], bottom: vec![] };
        let result = propose(&mods, &current, &pins);
        assert_eq!(result.proposed, vec!["SwordSkin".to_string(), "BigOverhaul".to_string()]);
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
}

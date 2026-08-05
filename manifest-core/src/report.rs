use crate::conflicts::ConflictGraph;
use crate::model::ModFile;
use crate::sort::{propose, Move, Pins};
use crate::warnings::{detect, Warning};

pub const SCHEMA_VERSION: u32 = 3;

#[derive(Debug, serde::Serialize)]
pub struct ReportMod {
    pub name: String,
    pub path: String,
    pub enabled: bool,
    pub asset_count: usize,
    pub error: Option<String>,
    pub pinned: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ReportConflict {
    pub asset: String,
    pub providers: Vec<String>,
    pub winner: String,
}

#[derive(Debug, serde::Serialize)]
pub struct Report {
    pub schema_version: u32,
    pub mods: Vec<ReportMod>,
    pub conflicts: Vec<ReportConflict>,
    pub warnings: Vec<Warning>,
    pub current_order: Vec<String>,
    pub proposed_order: Vec<String>,
    pub moves: Vec<Move>,
    // True when the current arrangement only looks optimal because pins are
    // holding cargo the sorter would otherwise move. Additive field; schema
    // version is unchanged.
    pub sort_held_by_pins: bool,
}

pub fn build(mods: &[ModFile], order: &[String], pins: &Pins) -> Report {
    let graph = ConflictGraph::build(mods, order);
    let warnings = detect(mods, order, &graph);
    let sort = propose(mods, order, pins);
    // Stale names linger in the order until the game's next boot; compare
    // only names that can actually load, mirroring the GUI's sortNeeded.
    let loadable: std::collections::BTreeSet<&str> = mods
        .iter()
        .filter(|m| m.enabled)
        .map(|m| m.name.as_str())
        .collect();
    fn filt<'a>(o: &'a [String], loadable: &std::collections::BTreeSet<&str>) -> Vec<&'a str> {
        o.iter()
            .map(String::as_str)
            .filter(|n| loadable.contains(n))
            .collect()
    }
    let current = filt(order, &loadable);
    let sort_held_by_pins = (!pins.top.is_empty() || !pins.bottom.is_empty())
        && filt(&sort.proposed, &loadable) == current
        && filt(&propose(mods, order, &Pins::default()).proposed, &loadable) != current;
    let conflicts = graph
        .conflicting()
        .into_iter()
        .map(|(asset, providers)| ReportConflict {
            asset: asset.clone(),
            providers: providers.clone(),
            winner: providers.last().cloned().unwrap_or_default(),
        })
        .collect();
    Report {
        schema_version: SCHEMA_VERSION,
        mods: mods
            .iter()
            .map(|m| ReportMod {
                name: m.name.clone(),
                path: m.path.to_string_lossy().to_string(),
                enabled: m.enabled,
                asset_count: m.assets.len(),
                error: m.error.clone(),
                pinned: if pins.top.contains(&m.name) {
                    Some("top".to_string())
                } else if pins.bottom.contains(&m.name) {
                    Some("bottom".to_string())
                } else {
                    None
                },
            })
            .collect(),
        conflicts,
        warnings,
        current_order: order.to_vec(),
        proposed_order: sort.proposed,
        moves: sort.moves,
        sort_held_by_pins,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ModFile;
    use crate::sort::Pins;
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
    fn held_by_pins_when_a_pin_silences_the_sorter() {
        // Unpinned, the sorter wants Big (broad) before Small (specific).
        let mods = vec![mk("Small", &["a"]), mk("Big", &["a", "b"])];
        let order: Vec<String> = ["Small", "Big"].map(String::from).into();
        let pins = Pins {
            top: vec!["Small".to_string()],
            bottom: vec![],
        };
        let report = build(&mods, &order, &pins);
        assert!(report.moves.is_empty(), "pin should silence the sorter");
        assert!(report.sort_held_by_pins);
    }

    #[test]
    fn not_held_without_pins_or_when_pins_change_nothing() {
        let mods = vec![mk("Small", &["a"]), mk("Big", &["a", "b"])];
        let order: Vec<String> = ["Small", "Big"].map(String::from).into();
        let unpinned = build(&mods, &order, &Pins::default());
        assert!(!unpinned.moves.is_empty());
        assert!(!unpinned.sort_held_by_pins);

        let sorted_order: Vec<String> = ["Big", "Small"].map(String::from).into();
        let harmless_pin = Pins {
            top: vec!["Big".to_string()],
            bottom: vec![],
        };
        let report = build(&mods, &sorted_order, &harmless_pin);
        assert!(report.moves.is_empty());
        assert!(!report.sort_held_by_pins);
    }

    #[test]
    fn not_held_while_a_sort_is_still_proposed() {
        let mods = vec![mk("Small", &["a"]), mk("Big", &["a", "b"])];
        let order: Vec<String> = ["Big", "Small"].map(String::from).into();
        let pins = Pins {
            top: vec!["Small".to_string()],
            bottom: vec![],
        };
        let report = build(&mods, &order, &pins);
        assert!(
            !report.moves.is_empty(),
            "pinned proposal should still differ"
        );
        assert!(!report.sort_held_by_pins);
    }

    #[test]
    fn golden_report_shape() {
        let mods = vec![mk("Small", &["a"]), mk("Big", &["a", "b"])];
        let order: Vec<String> = ["Small", "Big"].map(String::from).into();
        let report = build(&mods, &order, &Pins::default());
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["schema_version"], 3);
        assert_eq!(json["mods"].as_array().unwrap().len(), 2);
        assert_eq!(json["conflicts"][0]["asset"], "a");
        assert_eq!(json["conflicts"][0]["winner"], "Big");
        assert_eq!(json["proposed_order"][0], "Big");
        assert_eq!(json["proposed_order"][1], "Small");
        assert!(!json["moves"].as_array().unwrap().is_empty());
    }

    #[test]
    fn report_marks_pinned_mods_and_ignores_stale_pins() {
        let mods = vec![
            mk("Small", &["a"]),
            mk("Big", &["a", "b"]),
            mk("Tiny", &["z"]),
        ];
        let order: Vec<String> = ["Small", "Big", "Tiny"].map(String::from).into();
        let pins = Pins {
            top: vec!["Small".to_string()],
            bottom: vec!["Tiny".to_string(), "Ghost".to_string()],
        };
        let report = build(&mods, &order, &pins);
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["schema_version"], 3);
        assert_eq!(json["mods"][0]["pinned"], "top");
        assert_eq!(json["mods"][1]["pinned"], serde_json::Value::Null);
        assert_eq!(json["mods"][2]["pinned"], "bottom");
        // Pinned mod holds the top despite the specificity heuristic.
        assert_eq!(json["proposed_order"][0], "Small");
        // A pin naming a mod that does not exist never enters the order.
        assert!(report.proposed_order.iter().all(|n| n != "Ghost"));
    }

    #[test]
    fn report_mods_carry_their_file_paths() {
        let mods = vec![mk("Small", &["a"])];
        let order: Vec<String> = ["Small"].map(String::from).into();
        let report = build(&mods, &order, &Pins::default());
        let json = serde_json::to_value(&report).unwrap();
        assert_eq!(json["schema_version"], 3);
        assert_eq!(json["mods"][0]["path"], "/tmp/Small.otr");
    }
}

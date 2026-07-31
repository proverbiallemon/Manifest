use crate::conflicts::ConflictGraph;
use crate::model::ModFile;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Warning {
    TotalEclipse { name: String },
    MutualOverlap { names: Vec<String> },
    Unlistable { name: String, reason: String },
    DuplicateGamebananaMod { mod_id: u64, names: Vec<String> },
}

pub fn detect(mods: &[ModFile], _order: &[String], graph: &ConflictGraph) -> Vec<Warning> {
    let mut warnings = Vec::new();
    let enabled: Vec<&ModFile> = mods.iter().filter(|m| m.enabled).collect();

    for m in &enabled {
        if let Some(reason) = &m.error {
            warnings.push(Warning::Unlistable { name: m.name.clone(), reason: reason.clone() });
        }
    }

    // Mutual overlap: identical non-empty asset sets.
    let mut by_assets: BTreeMap<Vec<String>, Vec<String>> = BTreeMap::new();
    for m in &enabled {
        if m.error.is_none() && !m.assets.is_empty() {
            by_assets
                .entry(m.assets.iter().cloned().collect())
                .or_default()
                .push(m.name.clone());
        }
    }
    let mut in_overlap_group: Vec<String> = Vec::new();
    for (_, mut names) in by_assets {
        if names.len() >= 2 {
            names.sort();
            in_overlap_group.extend(names.clone());
            warnings.push(Warning::MutualOverlap { names });
        }
    }

    for m in &enabled {
        if m.error.is_some() || m.assets.is_empty() || in_overlap_group.contains(&m.name) {
            continue;
        }
        let eclipsed = m
            .assets
            .iter()
            .all(|a| graph.winner(a).map(|w| w != &m.name).unwrap_or(false));
        if eclipsed {
            warnings.push(Warning::TotalEclipse { name: m.name.clone() });
        }
    }

    // Duplicate GameBanana mod across distinct folders.
    let mut by_gb: BTreeMap<u64, Vec<(String, String)>> = BTreeMap::new();
    for m in mods.iter() {
        if let (Some(id), Some(folder)) = (
            m.gamebanana_mod_id,
            m.path.parent().map(|p| p.to_string_lossy().to_string()),
        ) {
            by_gb.entry(id).or_default().push((folder, m.name.clone()));
        }
    }
    for (mod_id, entries) in by_gb {
        let folders: std::collections::BTreeSet<&String> = entries.iter().map(|(f, _)| f).collect();
        if folders.len() >= 2 {
            let mut names: Vec<String> = entries.into_iter().map(|(_, n)| n).collect();
            names.sort();
            names.dedup();
            warnings.push(Warning::DuplicateGamebananaMod { mod_id, names });
        }
    }

    // Sort by kind (0-3) then by primary name for determinism
    warnings.sort_by_key(|w| {
        let (kind_rank, primary_name) = match w {
            Warning::DuplicateGamebananaMod { mod_id, names: _ } => (0, mod_id.to_string()),
            Warning::MutualOverlap { names } => (1, names.first().cloned().unwrap_or_default()),
            Warning::TotalEclipse { name } => (2, name.clone()),
            Warning::Unlistable { name, reason: _ } => (3, name.clone()),
        };
        (kind_rank, primary_name)
    });
    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conflicts::ConflictGraph;
    use crate::model::ModFile;
    use std::collections::BTreeSet;

    fn mk(name: &str, assets: &[&str], error: Option<&str>) -> ModFile {
        ModFile {
            path: format!("/tmp/{name}/f.otr").into(),
            name: name.into(),
            enabled: true,
            assets: assets.iter().map(|s| s.to_string()).collect::<BTreeSet<_>>(),
            error: error.map(String::from),
            gamebanana_mod_id: None,
        }
    }

    #[test]
    fn detects_eclipse_mutual_overlap_and_unlistable() {
        let mods = vec![
            mk("Eclipsed", &["a", "b"], None),
            mk("CoverA", &["a"], None),
            mk("CoverB", &["b"], None),
            mk("TwinOne", &["z"], None),
            mk("TwinTwo", &["z"], None),
            mk("Sick", &[], Some("no (listfile)")),
        ];
        let order: Vec<String> =
            ["Eclipsed", "CoverA", "CoverB", "TwinOne", "TwinTwo"].map(String::from).into();
        let graph = ConflictGraph::build(&mods, &order);
        let warnings = detect(&mods, &order, &graph);
        assert!(warnings.contains(&Warning::TotalEclipse { name: "Eclipsed".into() }));
        assert!(warnings.contains(&Warning::MutualOverlap {
            names: vec!["TwinOne".into(), "TwinTwo".into()]
        }));
        assert!(warnings.contains(&Warning::Unlistable {
            name: "Sick".into(),
            reason: "no (listfile)".into()
        }));
        // TwinOne is overridden on its only asset, but mutual overlap is the
        // more precise diagnosis - it must NOT also be reported as eclipsed.
        assert!(!warnings.contains(&Warning::TotalEclipse { name: "TwinOne".into() }));
    }

    #[test]
    fn detects_duplicate_gamebanana_across_folders_with_disabled_mod() {
        // Test that DuplicateGamebananaMod is detected across different parent folders
        // even when one mod is disabled.
        let mod1 = ModFile {
            path: "/folder1/mod_a.otr".into(),
            name: "ModA".into(),
            enabled: true,
            assets: BTreeSet::new(),
            error: None,
            gamebanana_mod_id: Some(777),
        };
        let mod2 = ModFile {
            path: "/folder2/mod_b.otr".into(),
            name: "ModB".into(),
            enabled: false, // disabled
            assets: BTreeSet::new(),
            error: None,
            gamebanana_mod_id: Some(777),
        };
        let mods = vec![mod1, mod2];
        let order: Vec<String> = vec![];
        let graph = ConflictGraph::build(&mods, &order);
        let warnings = detect(&mods, &order, &graph);

        // Should detect duplicate across folders despite disabled status
        assert!(warnings.contains(&Warning::DuplicateGamebananaMod {
            mod_id: 777,
            names: vec!["ModA".into(), "ModB".into()]
        }));
    }

    #[test]
    fn does_not_detect_duplicate_gamebanana_in_same_folder() {
        // Test that two mods with same gamebanana_mod_id in the SAME folder
        // do NOT trigger the duplicate warning.
        let mod1 = ModFile {
            path: "/shared/mod_a.otr".into(),
            name: "ModA".into(),
            enabled: true,
            assets: BTreeSet::new(),
            error: None,
            gamebanana_mod_id: Some(777),
        };
        let mod2 = ModFile {
            path: "/shared/mod_b.otr".into(),
            name: "ModB".into(),
            enabled: true,
            assets: BTreeSet::new(),
            error: None,
            gamebanana_mod_id: Some(777),
        };
        let mods = vec![mod1, mod2];
        let order: Vec<String> = vec![];
        let graph = ConflictGraph::build(&mods, &order);
        let warnings = detect(&mods, &order, &graph);

        // Should NOT detect duplicate since both are in the same parent folder
        assert!(!warnings.contains(&Warning::DuplicateGamebananaMod {
            mod_id: 777,
            names: vec!["ModA".into(), "ModB".into()]
        }));
    }
}

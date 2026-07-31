use crate::model::ModFile;
use std::collections::BTreeMap;

pub struct ConflictGraph {
    pub providers: BTreeMap<String, Vec<String>>,
}

impl ConflictGraph {
    pub fn build(mods: &[ModFile], order: &[String]) -> ConflictGraph {
        let position = |name: &str| order.iter().position(|o| o == name);
        let mut usable: Vec<&ModFile> = mods
            .iter()
            .filter(|m| m.enabled && m.error.is_none())
            .collect();
        usable.sort_by(|a, b| match (position(&a.name), position(&b.name)) {
            (Some(x), Some(y)) => x.cmp(&y),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.name.cmp(&b.name),
        });
        let mut providers: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for m in usable {
            for asset in &m.assets {
                let list = providers.entry(asset.clone()).or_default();
                if !list.contains(&m.name) {
                    list.push(m.name.clone());
                }
            }
        }
        ConflictGraph { providers }
    }

    pub fn conflicting(&self) -> BTreeMap<&String, &Vec<String>> {
        self.providers.iter().filter(|(_, v)| v.len() >= 2).collect()
    }

    pub fn winner<'a>(&'a self, asset: &str) -> Option<&'a String> {
        self.providers.get(asset).and_then(|v| v.last())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::ModFile;
    use std::collections::BTreeSet;

    fn mk(name: &str, enabled: bool, assets: &[&str]) -> ModFile {
        ModFile {
            path: format!("/tmp/{name}.otr").into(),
            name: name.into(),
            enabled,
            assets: assets.iter().map(|s| s.to_string()).collect::<BTreeSet<_>>(),
            error: None,
            gamebanana_mod_id: None,
        }
    }

    #[test]
    fn builds_providers_in_load_order_and_finds_conflicts() {
        let mods = vec![
            mk("Big", true, &["a", "b", "c"]),
            mk("Small", true, &["b"]),
            mk("Off", false, &["b"]),
        ];
        let order = vec!["Big".to_string(), "Small".to_string()];
        let graph = ConflictGraph::build(&mods, &order);
        let conflicting = graph.conflicting();
        assert_eq!(conflicting.len(), 1);
        assert_eq!(conflicting[&"b".to_string()], &vec!["Big".to_string(), "Small".to_string()]);
        assert_eq!(graph.winner("b"), Some(&"Small".to_string()));
        assert_eq!(graph.winner("a"), Some(&"Big".to_string()));
    }
}

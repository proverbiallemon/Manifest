use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ModFile {
    pub path: PathBuf,
    pub name: String,
    pub enabled: bool,
    pub assets: BTreeSet<String>,
    pub error: Option<String>,
    pub gamebanana_mod_id: Option<u64>,
}

impl ModFile {
    pub fn order_name(&self) -> &str {
        &self.name
    }
}

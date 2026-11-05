//! The models Codex offers this account: it keeps the list up to date in its home directory.

use super::models::is_model_name;
use super::programs::home;
use serde::Deserialize;
use std::path::PathBuf;

const CACHE_FILE: &str = "models_cache.json";
/// Models with any other visibility are not meant to be picked by hand.
const PICKABLE: &str = "list";

#[derive(Deserialize)]
struct Cache {
    models: Vec<CachedModel>,
}

#[derive(Deserialize)]
struct CachedModel {
    slug: String,
    #[serde(default)]
    visibility: String,
    #[serde(default)]
    priority: i64,
}

/// `None` when Codex has not stored a usable list (not installed, never run, another format).
pub fn cached_models() -> Option<Vec<String>> {
    let dir = std::env::var_os("CODEX_HOME").map_or_else(|| home().join(".codex"), PathBuf::from);
    parse_cache(&std::fs::read(dir.join(CACHE_FILE)).ok()?)
}

/// Pickable models, the ones Codex recommends first.
fn parse_cache(raw: &[u8]) -> Option<Vec<String>> {
    let mut models = serde_json::from_slice::<Cache>(raw).ok()?.models;
    models.retain(|m| m.visibility == PICKABLE && is_model_name(&m.slug));
    models.sort_by_key(|m| m.priority);
    let slugs: Vec<String> = models.into_iter().map(|m| m.slug).collect();
    (!slugs.is_empty()).then_some(slugs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_pickable_models_by_priority() {
        let raw = br#"{"fetched_at":"x","models":[
            {"slug":"second","visibility":"list","priority":2,"extra":{"ignored":true}},
            {"slug":"hidden","visibility":"hide","priority":0},
            {"slug":"first","visibility":"list","priority":1}]}"#;
        assert_eq!(parse_cache(raw).unwrap(), ["first", "second"]);
    }

    #[test]
    fn no_usable_list_is_none() {
        assert!(parse_cache(b"not json").is_none());
        assert!(parse_cache(br#"{"other":[]}"#).is_none());
        assert!(parse_cache(br#"{"models":[{"slug":"x","visibility":"hide"}]}"#).is_none());
    }
}

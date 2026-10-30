use serde_json::Value;

/// Finds the first JSON object in free-form model output that has `key` at the top level.
pub fn extract_object_with_key(text: &str, key: &str) -> Option<Value> {
    let has_key = |v: &Value| v.get(key).is_some();
    if let Ok(v) = serde_json::from_str::<Value>(text.trim())
        && has_key(&v)
    {
        return Some(v);
    }
    for (i, _) in text.match_indices('{') {
        let mut stream = serde_json::Deserializer::from_str(&text[i..]).into_iter::<Value>();
        if let Some(Ok(v)) = stream.next()
            && has_key(&v)
        {
            return Some(v);
        }
    }
    None
}

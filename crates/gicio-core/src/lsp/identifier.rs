/// The identifier around UTF-16 column `character` of `line`, for servers without `prepareRename`.
pub fn identifier_at(line: &str, character: u32) -> Option<String> {
    let units: Vec<u16> = line.encode_utf16().collect();
    let is_ident = |u: &u16| char::from_u32(*u as u32).is_some_and(|c| c.is_alphanumeric() || c == '_');
    let at = (character as usize).min(units.len());
    let start = units[..at].iter().rposition(|u| !is_ident(u)).map_or(0, |i| i + 1);
    let end = units[at..].iter().position(|u| !is_ident(u)).map_or(units.len(), |i| at + i);
    let name = String::from_utf16_lossy(&units[start..end]);
    name.chars().next().is_some_and(|c| c.is_alphabetic() || c == '_').then_some(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_word_under_the_cursor() {
        let line = "    var total = Sum(items);";
        assert_eq!(identifier_at(line, 8).as_deref(), Some("total"));
        assert_eq!(identifier_at(line, 13).as_deref(), Some("total"));
        assert_eq!(identifier_at(line, 17).as_deref(), Some("Sum"));
    }

    #[test]
    fn nothing_on_punctuation_or_numbers() {
        assert_eq!(identifier_at("a = (1);", 4), None);
        assert_eq!(identifier_at("x = 42;", 5), None);
        assert_eq!(identifier_at("", 0), None);
    }
}

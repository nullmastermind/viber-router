use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedKey {
    pub group_key: String,
    pub dynamic_keys: HashMap<i32, String>,
}

const DELIMITER: &str = "-rsv-";

pub fn parse_api_key(header: &str) -> ParsedKey {
    let parts: Vec<&str> = header.split(DELIMITER).collect();

    if parts.len() == 1 {
        return ParsedKey {
            group_key: header.to_string(),
            dynamic_keys: HashMap::new(),
        };
    }

    let group_key = parts[0].to_string();
    let mut dynamic_keys = HashMap::new();

    for segment in &parts[1..] {
        let Some(dash_pos) = segment.find('-') else {
            // Segment has no dash (short_id only, no key) → treat entire header as plain key
            return ParsedKey {
                group_key: header.to_string(),
                dynamic_keys: HashMap::new(),
            };
        };

        let short_id_str = &segment[..dash_pos];
        let key = &segment[dash_pos + 1..];

        let Ok(short_id) = short_id_str.parse::<i32>() else {
            // Non-numeric short_id → treat entire header as plain key
            return ParsedKey {
                group_key: header.to_string(),
                dynamic_keys: HashMap::new(),
            };
        };

        if key.is_empty() {
            // No key after short_id → treat entire header as plain key
            return ParsedKey {
                group_key: header.to_string(),
                dynamic_keys: HashMap::new(),
            };
        }

        dynamic_keys.insert(short_id, key.to_string());
    }

    ParsedKey {
        group_key,
        dynamic_keys,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plain_group_key() {
        let result = parse_api_key("sk-vibervn-abc123");
        assert_eq!(result.group_key, "sk-vibervn-abc123");
        assert!(result.dynamic_keys.is_empty());
    }

    #[test]
    fn test_single_dynamic_key() {
        let result = parse_api_key("sk-vibervn-abc123-rsv-1-sk-openai-xyz");
        assert_eq!(result.group_key, "sk-vibervn-abc123");
        assert_eq!(result.dynamic_keys.len(), 1);
        assert_eq!(result.dynamic_keys[&1], "sk-openai-xyz");
    }

    #[test]
    fn test_multiple_dynamic_keys() {
        let result = parse_api_key("sk-vibervn-abc123-rsv-1-sk-openai-xyz-rsv-3-sk-ant-abc");
        assert_eq!(result.group_key, "sk-vibervn-abc123");
        assert_eq!(result.dynamic_keys.len(), 2);
        assert_eq!(result.dynamic_keys[&1], "sk-openai-xyz");
        assert_eq!(result.dynamic_keys[&3], "sk-ant-abc");
    }

    #[test]
    fn test_malformed_non_numeric_short_id() {
        let input = "sk-vibervn-abc123-rsv-notanumber-sk-key";
        let result = parse_api_key(input);
        assert_eq!(result.group_key, input);
        assert!(result.dynamic_keys.is_empty());
    }

    #[test]
    fn test_segment_with_no_key_after_short_id() {
        let input = "sk-vibervn-abc123-rsv-1";
        let result = parse_api_key(input);
        assert_eq!(result.group_key, input);
        assert!(result.dynamic_keys.is_empty());
    }
}

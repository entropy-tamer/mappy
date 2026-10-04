//! Tag format parsers

use anyhow::Result;
use serde_json;

/// Trait for parsing tag formats
pub trait TagParser: Send + Sync {
    /// Parse input string into a vector of tags
    fn parse(&self, input: &str) -> Result<Vec<String>>;

    /// Get the format name
    fn format_name(&self) -> &'static str;
}

/// Space-separated tag parser
pub struct SpaceSeparatedParser;

impl SpaceSeparatedParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SpaceSeparatedParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TagParser for SpaceSeparatedParser {
    fn parse(&self, input: &str) -> Result<Vec<String>> {
        Ok(input
            .split_whitespace()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }

    fn format_name(&self) -> &'static str {
        "space-separated"
    }
}

/// Comma-separated tag parser
pub struct CommaSeparatedParser;

impl CommaSeparatedParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CommaSeparatedParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TagParser for CommaSeparatedParser {
    fn parse(&self, input: &str) -> Result<Vec<String>> {
        Ok(input
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    }

    fn format_name(&self) -> &'static str {
        "comma-separated"
    }
}

/// JSON array parser
pub struct JsonParser;

impl JsonParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonParser {
    fn default() -> Self {
        Self::new()
    }
}

impl TagParser for JsonParser {
    fn parse(&self, input: &str) -> Result<Vec<String>> {
        let tags: Vec<String> = serde_json::from_str(input)?;
        Ok(tags)
    }

    fn format_name(&self) -> &'static str {
        "json"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_separated() {
        let parser = SpaceSeparatedParser::new();
        let tags = parser.parse("tag1 tag2 tag3").unwrap();
        assert_eq!(tags, vec!["tag1", "tag2", "tag3"]);
    }

    #[test]
    fn test_space_separated_empty() {
        let parser = SpaceSeparatedParser::new();
        let tags = parser.parse("").unwrap();
        assert_eq!(tags, Vec::<String>::new());
    }

    #[test]
    fn test_space_separated_with_extra_spaces() {
        let parser = SpaceSeparatedParser::new();
        let tags = parser.parse("  tag1   tag2  tag3  ").unwrap();
        assert_eq!(tags, vec!["tag1", "tag2", "tag3"]);
    }

    #[test]
    fn test_space_separated_single_tag() {
        let parser = SpaceSeparatedParser::new();
        let tags = parser.parse("tag1").unwrap();
        assert_eq!(tags, vec!["tag1"]);
    }

    #[test]
    fn test_space_separated_format_name() {
        let parser = SpaceSeparatedParser::new();
        assert_eq!(parser.format_name(), "space-separated");
    }

    #[test]
    fn test_space_separated_default() {
        let parser = SpaceSeparatedParser::default();
        assert_eq!(parser.format_name(), "space-separated");
    }

    #[test]
    fn test_comma_separated() {
        let parser = CommaSeparatedParser::new();
        let tags = parser.parse("tag1,tag2,tag3").unwrap();
        assert_eq!(tags, vec!["tag1", "tag2", "tag3"]);
    }

    #[test]
    fn test_comma_separated_empty() {
        let parser = CommaSeparatedParser::new();
        let tags = parser.parse("").unwrap();
        assert_eq!(tags, Vec::<String>::new());
    }

    #[test]
    fn test_comma_separated_with_spaces() {
        let parser = CommaSeparatedParser::new();
        let tags = parser.parse(" tag1 , tag2 , tag3 ").unwrap();
        assert_eq!(tags, vec!["tag1", "tag2", "tag3"]);
    }

    #[test]
    fn test_comma_separated_single_tag() {
        let parser = CommaSeparatedParser::new();
        let tags = parser.parse("tag1").unwrap();
        assert_eq!(tags, vec!["tag1"]);
    }

    #[test]
    fn test_comma_separated_format_name() {
        let parser = CommaSeparatedParser::new();
        assert_eq!(parser.format_name(), "comma-separated");
    }

    #[test]
    fn test_comma_separated_default() {
        let parser = CommaSeparatedParser::default();
        assert_eq!(parser.format_name(), "comma-separated");
    }

    #[test]
    fn test_json() {
        let parser = JsonParser::new();
        let tags = parser.parse(r#"["tag1","tag2","tag3"]"#).unwrap();
        assert_eq!(tags, vec!["tag1", "tag2", "tag3"]);
    }

    #[test]
    fn test_json_empty() {
        let parser = JsonParser::new();
        let tags = parser.parse("[]").unwrap();
        assert_eq!(tags, Vec::<String>::new());
    }

    #[test]
    fn test_json_single_tag() {
        let parser = JsonParser::new();
        let tags = parser.parse(r#"["tag1"]"#).unwrap();
        assert_eq!(tags, vec!["tag1"]);
    }

    #[test]
    fn test_json_invalid() {
        let parser = JsonParser::new();
        let result = parser.parse("not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_json_format_name() {
        let parser = JsonParser::new();
        assert_eq!(parser.format_name(), "json");
    }

    #[test]
    fn test_json_default() {
        let parser = JsonParser::default();
        assert_eq!(parser.format_name(), "json");
    }

    #[test]
    fn test_json_with_special_chars() {
        let parser = JsonParser::new();
        let tags = parser
            .parse(r#"["tag-with-dash","tag_with_underscore"]"#)
            .unwrap();
        assert_eq!(tags, vec!["tag-with-dash", "tag_with_underscore"]);
    }

    #[test]
    fn test_space_separated_with_special_chars() {
        let parser = SpaceSeparatedParser::new();
        let tags = parser
            .parse("tag-with-dash tag_with_underscore tag.with.dots")
            .unwrap();
        assert_eq!(
            tags,
            vec!["tag-with-dash", "tag_with_underscore", "tag.with.dots"]
        );
    }

    #[test]
    fn test_comma_separated_with_special_chars() {
        let parser = CommaSeparatedParser::new();
        let tags = parser
            .parse("tag-with-dash,tag_with_underscore,tag.with.dots")
            .unwrap();
        assert_eq!(
            tags,
            vec!["tag-with-dash", "tag_with_underscore", "tag.with.dots"]
        );
    }

    #[test]
    fn test_tag_parser_trait() {
        let parser: Box<dyn TagParser> = Box::new(SpaceSeparatedParser::new());
        let tags = parser.parse("tag1 tag2").unwrap();
        assert_eq!(tags, vec!["tag1", "tag2"]);
        assert_eq!(parser.format_name(), "space-separated");
    }
}

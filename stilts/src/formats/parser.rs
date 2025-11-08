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
    fn test_comma_separated() {
        let parser = CommaSeparatedParser::new();
        let tags = parser.parse("tag1,tag2,tag3").unwrap();
        assert_eq!(tags, vec!["tag1", "tag2", "tag3"]);
    }
    
    #[test]
    fn test_json() {
        let parser = JsonParser::new();
        let tags = parser.parse(r#"["tag1","tag2","tag3"]"#).unwrap();
        assert_eq!(tags, vec!["tag1", "tag2", "tag3"]);
    }
}


//! Tag format serializers

use anyhow::Result;
use serde_json;

/// Trait for serializing tags to different formats
pub trait TagSerializer: Send + Sync {
    /// Serialize tags to a string
    fn serialize(&self, tags: &[String]) -> Result<String>;

    /// Get the format name
    fn format_name(&self) -> &'static str;
}

/// Space-separated serializer
pub struct SpaceSeparatedSerializer;

impl Default for SpaceSeparatedSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl SpaceSeparatedSerializer {
    pub fn new() -> Self {
        Self
    }
}

impl TagSerializer for SpaceSeparatedSerializer {
    fn serialize(&self, tags: &[String]) -> Result<String> {
        Ok(tags.join(" "))
    }

    fn format_name(&self) -> &'static str {
        "space-separated"
    }
}

/// Comma-separated serializer
pub struct CommaSeparatedSerializer;

impl Default for CommaSeparatedSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl CommaSeparatedSerializer {
    pub fn new() -> Self {
        Self
    }
}

impl TagSerializer for CommaSeparatedSerializer {
    fn serialize(&self, tags: &[String]) -> Result<String> {
        Ok(tags.join(","))
    }

    fn format_name(&self) -> &'static str {
        "comma-separated"
    }
}

/// JSON serializer
pub struct JsonSerializer;

impl Default for JsonSerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonSerializer {
    pub fn new() -> Self {
        Self
    }
}

impl TagSerializer for JsonSerializer {
    fn serialize(&self, tags: &[String]) -> Result<String> {
        Ok(serde_json::to_string(tags)?)
    }

    fn format_name(&self) -> &'static str {
        "json"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_separated_serializer() {
        let serializer = SpaceSeparatedSerializer::new();
        let tags = vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()];
        let result = serializer.serialize(&tags);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "tag1 tag2 tag3");
    }

    #[test]
    fn test_space_separated_serializer_single_tag() {
        let serializer = SpaceSeparatedSerializer::new();
        let tags = vec!["single".to_string()];
        let result = serializer.serialize(&tags);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "single");
    }

    #[test]
    fn test_space_separated_serializer_empty() {
        let serializer = SpaceSeparatedSerializer::new();
        let tags: Vec<String> = vec![];
        let result = serializer.serialize(&tags);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_space_separated_serializer_format_name() {
        let serializer = SpaceSeparatedSerializer::new();
        assert_eq!(serializer.format_name(), "space-separated");
    }

    #[test]
    fn test_space_separated_serializer_default() {
        let serializer = SpaceSeparatedSerializer::default();
        let tags = vec!["tag1".to_string(), "tag2".to_string()];
        let result = serializer.serialize(&tags);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "tag1 tag2");
    }

    #[test]
    fn test_comma_separated_serializer() {
        let serializer = CommaSeparatedSerializer::new();
        let tags = vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()];
        let result = serializer.serialize(&tags);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "tag1,tag2,tag3");
    }

    #[test]
    fn test_comma_separated_serializer_single_tag() {
        let serializer = CommaSeparatedSerializer::new();
        let tags = vec!["single".to_string()];
        let result = serializer.serialize(&tags);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "single");
    }

    #[test]
    fn test_comma_separated_serializer_empty() {
        let serializer = CommaSeparatedSerializer::new();
        let tags: Vec<String> = vec![];
        let result = serializer.serialize(&tags);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_comma_separated_serializer_format_name() {
        let serializer = CommaSeparatedSerializer::new();
        assert_eq!(serializer.format_name(), "comma-separated");
    }

    #[test]
    fn test_comma_separated_serializer_default() {
        let serializer = CommaSeparatedSerializer::default();
        let tags = vec!["tag1".to_string(), "tag2".to_string()];
        let result = serializer.serialize(&tags);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "tag1,tag2");
    }

    #[test]
    fn test_json_serializer() {
        let serializer = JsonSerializer::new();
        let tags = vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()];
        let result = serializer.serialize(&tags);
        assert!(result.is_ok());
        let json_str = result.unwrap();
        assert!(json_str.contains("tag1"));
        assert!(json_str.contains("tag2"));
        assert!(json_str.contains("tag3"));
    }

    #[test]
    fn test_json_serializer_single_tag() {
        let serializer = JsonSerializer::new();
        let tags = vec!["single".to_string()];
        let result = serializer.serialize(&tags);
        assert!(result.is_ok());
        let json_str = result.unwrap();
        assert!(json_str.contains("single"));
    }

    #[test]
    fn test_json_serializer_empty() {
        let serializer = JsonSerializer::new();
        let tags: Vec<String> = vec![];
        let result = serializer.serialize(&tags);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "[]");
    }

    #[test]
    fn test_json_serializer_format_name() {
        let serializer = JsonSerializer::new();
        assert_eq!(serializer.format_name(), "json");
    }

    #[test]
    fn test_json_serializer_default() {
        let serializer = JsonSerializer::default();
        let tags = vec!["tag1".to_string(), "tag2".to_string()];
        let result = serializer.serialize(&tags);
        assert!(result.is_ok());
        let json_str = result.unwrap();
        assert!(json_str.contains("tag1"));
        assert!(json_str.contains("tag2"));
    }

    #[test]
    fn test_json_serializer_special_characters() {
        let serializer = JsonSerializer::new();
        let tags = vec!["tag with spaces".to_string(), "tag\"with\"quotes".to_string()];
        let result = serializer.serialize(&tags);
        assert!(result.is_ok());
        let json_str = result.unwrap();
        assert!(json_str.contains("tag with spaces"));
    }

    #[test]
    fn test_tag_serializer_trait() {
        let serializer: Box<dyn TagSerializer> = Box::new(SpaceSeparatedSerializer::new());
        let tags = vec!["tag1".to_string(), "tag2".to_string()];
        let result = serializer.serialize(&tags);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "tag1 tag2");
        assert_eq!(serializer.format_name(), "space-separated");
    }
}

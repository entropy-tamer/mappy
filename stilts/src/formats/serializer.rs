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


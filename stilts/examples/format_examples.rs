#![allow(clippy::cast_precision_loss)] // Acceptable for benchmark/example calculations
//! Format parsing examples

use stilts::formats::{CommaSeparatedParser, JsonParser, SpaceSeparatedParser, TagParser};

fn main() -> anyhow::Result<()> {
    // Space-separated
    let parser = SpaceSeparatedParser::new();
    let tags = parser.parse("tag1 tag2 tag3")?;
    println!("Space-separated: {:?}", tags);

    // Comma-separated
    let parser = CommaSeparatedParser::new();
    let tags = parser.parse("tag1,tag2,tag3")?;
    println!("Comma-separated: {:?}", tags);

    // JSON
    let parser = JsonParser::new();
    let tags = parser.parse(r#"["tag1","tag2","tag3"]"#)?;
    println!("JSON: {:?}", tags);

    Ok(())
}

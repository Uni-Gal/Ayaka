use crate::*;
use serde::Deserialize;
use serde_with::rust::maps_duplicate_key_is_error;
use std::collections::HashMap;

/// Represents a line in a prograph.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Line {
    /// An empty line, usually fallbacks to the base language one.
    Empty,
    /// A text line.
    Text(Text),
    /// Some `switches`.
    Switch {
        /// The switch items.
        switches: Vec<String>,
    },
    /// Custom line types.
    #[serde(with = "maps_duplicate_key_is_error")]
    Custom(HashMap<String, RawValue>),
}

#[cfg(test)]
mod test {
    use crate::{
        text::test::{str, text},
        *,
    };
    use std::collections::HashMap;

    #[test]
    fn parse() {
        let lines = r#"
- abc
- exec: $a
- switches:
  - a
  - b
- video: 0
-
        "#;
        let lines: Vec<Line> = serde_yaml::from_str(lines).unwrap();
        assert_eq!(lines.len(), 5);
        assert_eq!(lines[0], Line::Text(text(vec![str("abc")])));
        assert_eq!(
            lines[1],
            Line::Custom(HashMap::from([(
                "exec".to_string(),
                RawValue::Str("$a".to_string())
            )]))
        );
        assert_eq!(
            lines[2],
            Line::Switch {
                switches: vec!["a".to_string(), "b".to_string()]
            }
        );
        assert_eq!(
            lines[3],
            Line::Custom(HashMap::from([("video".to_string(), RawValue::Num(0))]))
        );
        assert_eq!(lines[4], Line::Empty);
    }
}

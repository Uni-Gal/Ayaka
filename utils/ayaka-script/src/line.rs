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
    /// An `exec` line, to execute scripts.
    Exec {
        /// The program to execute.
        exec: Program,
    },
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
    use std::collections::HashMap;

    use crate::*;

    #[test]
    fn parse() {
        let lines = r#"
- abc
- exec: $a
- switches:
  - a
  - b
- exec: $?
- video: 0
-
        "#;
        let lines: Vec<Line> = serde_yaml::from_str(lines).unwrap();
        assert_eq!(lines.len(), 6);
        assert_eq!(
            lines[0],
            Line::Text(Text(vec![SubText::Str("abc".to_string())]))
        );
        assert_eq!(
            lines[1],
            Line::Exec {
                exec: Program(vec![Expr::Ref(Ref::Ctx("a".to_string()))])
            }
        );
        assert_eq!(
            lines[2],
            Line::Switch {
                switches: vec!["a".to_string(), "b".to_string()]
            }
        );
        assert_eq!(
            lines[3],
            Line::Exec {
                exec: Program(vec![Expr::Ref(Ref::Ctx("?".to_string()))])
            }
        );
        assert_eq!(
            lines[4],
            Line::Custom(HashMap::from([("video".to_string(), RawValue::Num(0))]))
        );
        assert_eq!(lines[5], Line::Empty);
    }
}

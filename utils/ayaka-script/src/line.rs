use crate::*;
use serde::Deserialize;
use serde_with::rust::maps_duplicate_key_is_error;
use std::{collections::HashMap, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum Line {
    Empty,
    Text(Text),
    Exec {
        exec: Program,
    },
    Switch {
        switches: Vec<SwitchItem>,
    },
    #[serde(with = "maps_duplicate_key_is_error")]
    Custom(HashMap<String, RawValue>),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub struct SwitchItem {
    pub text: String,
    pub enabled: Option<Program>,
    pub action: Program,
}

impl FromStr for SwitchItem {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splits = s.split('|');
        let text = splits.next().unwrap_or_default().to_string();
        let enabled = if let Some(s) = splits.next() {
            if s.is_empty() {
                None
            } else {
                Some(s.parse()?)
            }
        } else {
            None
        };
        let action = splits.next().unwrap_or_default().parse()?;
        Ok(Self {
            text,
            enabled,
            action,
        })
    }
}

impl TryFrom<String> for SwitchItem {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
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
  - a||$n = 1
  - b|false|$n = 0
- video: 0
-
        "#;
        let lines: Vec<Line> = serde_yaml::from_str(lines).unwrap();
        assert_eq!(lines.len(), 5);
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
                switches: vec![
                    SwitchItem {
                        text: "a".to_string(),
                        enabled: None,
                        action: Program(vec![Expr::Binary(
                            Box::new(Expr::Ref(Ref::Ctx("n".to_string()))),
                            BinaryOp::Assign,
                            Box::new(Expr::Const(RawValue::Num(1)))
                        )])
                    },
                    SwitchItem {
                        text: "b".to_string(),
                        enabled: Some(Program(vec![Expr::Const(RawValue::Bool(false))])),
                        action: Program(vec![Expr::Binary(
                            Box::new(Expr::Ref(Ref::Ctx("n".to_string()))),
                            BinaryOp::Assign,
                            Box::new(Expr::Const(RawValue::Num(0)))
                        )])
                    }
                ]
            }
        );
        assert_eq!(
            lines[3],
            Line::Custom({
                let mut map = HashMap::new();
                map.insert("video".to_string(), RawValue::Num(0));
                map
            })
        );
        assert_eq!(lines[4], Line::Empty);
    }
}

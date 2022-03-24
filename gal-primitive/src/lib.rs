use gal_script::Program;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Value {
    Bool(bool),
    Num(i64),
    Str(String),
    Expr(Arc<Program>),
}

#[derive(Debug)]
pub struct Game {
    pub title: String,
    pub author: String,
    pub paras: Vec<Paragraph>,
}

impl Game {
    pub fn find_para(&self, tag: &str) -> Option<&Paragraph> {
        for p in &self.paras {
            if p.tag == tag {
                return Some(p);
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct Paragraph {
    pub tag: String,
    pub title: String,
    pub actions: Vec<Action>,
    pub next: Value,
}

#[derive(Debug)]
pub enum Action {
    Text(Value),
    Switch(Vec<SwitchItem>),
}

#[derive(Debug)]
pub struct SwitchItem {
    pub text: String,
    pub enabled: Value,
    pub action: Program,
}

#[derive(Debug, Default)]
pub struct RawContext {
    pub cur_para: String,
    pub cur_act: usize,
}

pub type VarMap = HashMap<String, Value>;

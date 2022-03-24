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
    pub title: Value,
    pub author: Value,
    pub paras: Vec<Paragraph>,
}

impl Game {
    pub fn find_para(&self, tag: &str) -> Option<&Paragraph> {
        for p in self.paras.iter() {
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
    pub title: Value,
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
    pub action: Program,
}

#[derive(Debug)]
pub struct RawContext {
    pub cur_para: String,
    pub cur_act: usize,
    pub vars: HashMap<String, Value>,
}

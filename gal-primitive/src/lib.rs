use gal_script::Expr;
use std::collections::HashMap;

pub enum Value {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Expression(Expr),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bool(b) => b.fmt(f),
            Self::Int(i) => i.fmt(f),
            Self::Float(fl) => fl.fmt(f),
            Self::String(s) => s.fmt(f),
            Self::Expression(_) => unimplemented!(),
        }
    }
}

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

pub struct Paragraph {
    pub tag: String,
    pub title: Value,
    pub actions: Vec<Action>,
    pub next: Value,
}

pub enum Action {
    Text(String),
    Switch(Vec<Value>, Expr),
}

pub struct RawContext {
    pub cur_para: String,
    pub cur_act: usize,
    pub vars: HashMap<String, Value>,
}

impl RawContext {
    pub fn find_var(&self, n: &str) -> Option<&Value> {
        self.vars.get(n)
    }
}

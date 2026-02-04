use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::rc::Rc;

#[derive(Debug)]
pub struct FileHandle {
    pub file: File,
    pub eof: bool,
    pub error: bool,
}

#[derive(Debug, Clone)]
pub enum Object {
    Integer(i64),
    String(String),
    Boolean(bool),
    Function(
        Vec<String>,
        Box<crate::ast::Statement>,
        Rc<RefCell<Environment>>,
    ), // params, body, env
    Builtin(fn(Vec<Object>) -> Object),
    File(Rc<RefCell<FileHandle>>),
    Null,
    ReturnValue(Box<Object>),
    Error(String),
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Integer(l), Object::Integer(r)) => l == r,
            (Object::String(l), Object::String(r)) => l == r,
            (Object::Boolean(l), Object::Boolean(r)) => l == r,
            (Object::Function(p1, b1, _), Object::Function(p2, b2, _)) => p1 == p2 && b1 == b2, // ignoring env
            (Object::Builtin(_), Object::Builtin(_)) => false, // Functions are not comparable easily
            (Object::File(_), Object::File(_)) => false,       // Files are not comparable easily
            (Object::Null, Object::Null) => true,
            (Object::ReturnValue(l), Object::ReturnValue(r)) => l == r,
            (Object::Error(l), Object::Error(r)) => l == r,
            _ => false,
        }
    }
}

impl Object {
    pub fn inspect(&self) -> String {
        match self {
            Object::Integer(val) => format!("{}", val),
            Object::String(val) => format!("{}", val),
            Object::Boolean(val) => format!("{}", val),
            Object::Function(params, _, _) => format!("fn({}) {{ ... }}", params.join(", ")),
            Object::Builtin(_) => "builtin function".to_string(),
            Object::File(_) => "file".to_string(),
            Object::Null => "null".to_string(),
            Object::ReturnValue(val) => val.inspect(),
            Object::Error(msg) => format!("ERROR: {}", msg),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Environment {
    store: HashMap<String, Object>,
    outer: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            store: HashMap::new(),
            outer: None,
        }
    }

    pub fn new_enclosed(outer: Rc<RefCell<Environment>>) -> Self {
        Environment {
            store: HashMap::new(),
            outer: Some(outer),
        }
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        match self.store.get(name) {
            Some(obj) => Some(obj.clone()),
            None => match &self.outer {
                Some(outer) => outer.borrow().get(name),
                None => None,
            },
        }
    }

    pub fn set(&mut self, name: String, val: Object) -> Object {
        self.store.insert(name, val.clone());
        val
    }
}

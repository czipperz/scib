use types::*;
use builtins::*;
use parse::parse;
use lex::lex;
use eval::eval;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Result, Error, ErrorKind};
use std::rc::Rc;

pub struct Scib {
    definitions: HashMap<String, Rc<Value>>,
}

impl Scib {
    pub fn new() -> Self {
        let mut instance = Scib {
            definitions: HashMap::new(),
        };
        instance.set(String::from("setq"),
                     Rc::new(Value::Macro(Rc::new(
                         Macro {
                             params: Parameters {
                                 required: vec![String::from("_setq-label"),
                                                String::from("_setq-value")],
                                 optional: vec![],
                                 rest: None,
                             },
                             body: Body::Rust(setq_f),
                         }))));
        instance.set(String::from("="),
                     Rc::new(Value::Function(Rc::new(
                         Function {
                             params: Parameters {
                                 required: vec![String::from("_=-first")],
                                 optional: vec![],
                                 rest: Some(String::from("_=-rest")),
                             },
                             body: Body::Rust(equalsign_f),
                         }))));
        instance.set(String::from("+"),
                     Rc::new(Value::Function(Rc::new(
                         Function {
                             params: Parameters {
                                 required: vec![],
                                 optional: vec![],
                                 rest: Some(String::from("_+")),
                             },
                             body: Body::Rust(sum_f),
                         }))));
        instance.set(String::from("-"),
                     Rc::new(Value::Function(Rc::new(
                         Function {
                             params: Parameters {
                                 required: vec![String::from("_--positive")],
                                 optional: vec![],
                                 rest: Some(String::from("_--negatives")),
                             },
                             body: Body::Rust(difference_f),
                         }))));
        instance.set(String::from("*"),
                     Rc::new(Value::Function(Rc::new(
                         Function {
                             params: Parameters {
                                 required: vec![],
                                 optional: vec![],
                                 rest: Some(String::from("_*")),
                             },
                             body: Body::Rust(product_f),
                         }))));
        instance.set(String::from("/"),
                     Rc::new(Value::Function(Rc::new(
                         Function {
                             params: Parameters {
                                 required: vec![String::from("_/-numerator")],
                                 optional: vec![],
                                 rest: Some(String::from("_/-denominator")),
                             },
                             body: Body::Rust(quotient_f),
                         }))));
        instance.set(String::from("list"),
                     Rc::new(Value::Function(Rc::new(
                         Function {
                             params: Parameters {
                                 required: vec![],
                                 optional: vec![],
                                 rest: Some(String::from("_list-rest")),
                             },
                             body: Body::Rust(list_f),
                         }))));
        instance.set(String::from("progn"),
                     Rc::new(Value::Macro(Rc::new(
                         Macro {
                             params: Parameters {
                                 required: vec![],
                                 optional: vec![],
                                 rest: Some(String::from("_progn-rest")),
                             },
                             body: Body::Rust(progn_f),
                         }))));
        instance.set(String::from("if"),
                     Rc::new(Value::Macro(Rc::new(
                         Macro {
                             params: Parameters {
                                 required: vec![String::from("_if-cond"), String::from("_if-iftrue")],
                                 optional: vec![],
                                 rest: Some(String::from("_if-iffalse")),
                             },
                             body: Body::Rust(if_f),
                         }))));
        instance.set(String::from("defmacro"),
                     Rc::new(Value::Macro(Rc::new(
                         Macro {
                             params: Parameters {
                                 required: vec![String::from("_defmacro-name")],
                                 optional: vec![],
                                 rest: Some(String::from("_defmacro-value")),
                             },
                             body: Body::Rust(defmacro_f),
                         }))));
        instance.set(String::from("define"),
                     Rc::new(Value::Macro(Rc::new(
                         Macro {
                             params: Parameters {
                                 required: vec![String::from("_define-name")],
                                 optional: vec![],
                                 rest: Some(String::from("_define-value")),
                             },
                             body: Body::Rust(define_f),
                         }))));
        instance.eval("(defmacro (when cond &rest rest) `(if ,cond (progn ,@rest)))").unwrap();
        instance
    }

    pub fn eval_file(&mut self, file_name: &str) -> Result<Rc<Value>> {
        let file = try!(File::open(file_name));
        let chars = BufReader::new(file).bytes().map(|r| r.unwrap() as char).fuse();
        let exprs = try!(parse(try!(lex(chars))));
        let mut result = Rc::new(Value::Nil);
        for expr in exprs {
            result = try!(eval(self, &expr));
        }
        Ok(result)
    }

    pub fn eval(&mut self, string: &str) -> Result<Rc<Value>> {
        let exprs = try!(parse(try!(lex(string.chars().fuse()))));
        progn(self, &exprs)
    }

    pub fn lookup(&self, name: &str) -> Result<Rc<Value>> {
        if let Some(v) = self.definitions.get(name) { Ok(v.clone()) }
        else { Err(Error::new(ErrorKind::InvalidData, format!("Unbound label {}", name))) }
    }

    pub fn get<'a>(&'a mut self, name: &str) -> Option<&'a Rc<Value>> {
        self.definitions.get(name)
    }

    pub fn set(&mut self, name: String, value: Rc<Value>) -> Option<Rc<Value>> {
        self.definitions.insert(name, value)
    }

    pub fn unbind(&mut self, name: &str) -> Option<Rc<Value>> {
        self.definitions.remove(name)
    }
}

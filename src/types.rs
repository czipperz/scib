use std::rc::Rc;
use std::io::{Result, Error, ErrorKind};
use instance::Scib;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Value {
    Nil,
    True,
    Number(f64),
    String(String),
    Label(String),
    List(Vec<Rc<Value>>),
    Quote(Rc<Value>),
    Backquote(Rc<Value>),
    Unquote(Rc<Value>),
    UnquoteList(Rc<Value>),
    Function(Rc<Function>),
    Macro(Rc<Function>),
}

impl Value {
    pub fn as_label(&self) -> Result<&String> {
        match *self {
            Value::Label(ref l) => Ok(l),
            _ => Err(Error::new(ErrorKind::InvalidInput,
                                format!("Expected label, found '{:?}'", self))),
        }
    }

    pub fn as_list(&self) -> Result<&Vec<Rc<Value>>> {
        match *self {
            Value::List(ref l) => Ok(l),
            _ => Err(Error::new(ErrorKind::InvalidInput,
                                format!("Expected list, found '{:?}'", self))),
        }
    }

    pub fn unwrap_list(&self) -> &Vec<Rc<Value>> {
        match *self {
            Value::List(ref l) => l,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub params: Parameters,
    pub body: Body,
}
pub type Macro = Function;

pub enum Body {
    Lisp(Vec<Rc<Value>>),
    Rust(fn(&mut Scib) -> Result<Rc<Value>>),
}

impl PartialEq for Body {
    fn eq(&self, other: &Body) -> bool {
        match (self, other) {
            (&Body::Lisp(ref l1), &Body::Lisp(ref l2)) => l1 == l2,
            _ => false,
        }
    }
}

impl fmt::Debug for Body {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
impl fmt::Display for Body {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Body::Lisp(ref body) => {
                let mut first = true;
                for v in body {
                    try!(write!(f, "{}{:?}", if first { "" } else { " " }, v));
                    first = false;
                }
            },
            &Body::Rust(_) => {
                try!(write!(f, "..."));
            },
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct Parameters {
    pub required: Vec<String>,
    pub optional: Vec<String>,
    pub rest: Option<String>,
}

impl fmt::Display for Parameters {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "("));
        let mut first = true;
        for r in &self.required {
            try!(write!(f, "{}{}", if first { "" } else { " " }, r));
            first = false;
        }
        if !self.optional.is_empty() {
            if first {
                first = false;
            } else {
                try!(write!(f, " "));
            }
            try!(write!(f, "&optional"));
            for o in &self.optional {
                try!(write!(f, " {}", o));
            }
        }
        if self.rest.is_some() {
            if first {
                first = false;
            } else {
                try!(write!(f, " "));
            }
            try!(write!(f, "&rest {}", self.rest.as_ref().unwrap()));
        }
        try!(write!(f, ")"));
        Ok(())
    }
}

impl Parameters {
    pub fn check_params_len(&self, len: usize) -> Result<()> {
        let len = len - 1;
        if len < self.required.len() {
            Err(Error::new(ErrorKind::InvalidInput,
                           format!("Not enough arguments to function call (requires {} {})",
                                   if self.optional.is_empty() && self.rest.is_none() {
                                       "exactly"
                                   } else {
                                       "at least"
                                   },
                                   self.required.len())))
        } else if self.rest.is_none() && len > self.required.len() + self.optional.len() {
            Err(Error::new(ErrorKind::InvalidInput,
                           format!("Too many arguments to function call (requires {} {})",
                                   if self.optional.is_empty() {
                                       "exactly"
                                   } else {
                                       "at most"
                                   },
                                   self.required.len() + self.optional.len())))
        } else {
            Ok(())
        }
    }

    pub fn bind_params<I: Iterator<Item = Rc<Value>>>(&self, iter: I) -> Vec<(String, Rc<Value>)> {
        let mut iter = iter.fuse();
        assert!(iter.next().is_some());
        let mut v: Vec<(String, Rc<Value>)> =
            Vec::with_capacity(self.required.len() + self.optional.len() +
                               if self.rest.is_some() { 1 } else { 0 });
        for r in &self.required {
            v.push((r.clone(), iter.next().unwrap()));
        }
        for r in &self.optional {
            v.push((r.clone(), iter.next().unwrap_or_else(|| Rc::new(Value::Nil))));
        }
        match self.rest {
            Some(ref rest) => v.push((rest.clone(), Rc::new(Value::List(iter.collect())))),
            None => assert!(iter.next().is_none()),
        }
        v
    }
}

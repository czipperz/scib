use std::rc::Rc;
use types::*;
use eval::eval;
use instance::Scib;
use std::io::{Result, Error, ErrorKind};

pub fn let_vars<'a, I: Iterator<Item=(String, Rc<Value>)>>(
    scib: &mut Scib, args: I, to_eval: &Body) -> Result<Rc<Value>> {
    let mut old_bound = Vec::new();
    for (name, value) in args {
        if let Some(old_value) = scib.set(name.clone(), value.clone()) {
            old_bound.push((name, old_value));
        }
    }
    let result =
        match to_eval {
            &Body::Lisp(ref l) => {
                let mut result = Rc::new(Value::Nil);
                for v in l {
                    result = try!(eval(scib, &v));
                }
                result
            },
            &Body::Rust(f) => try!(f(scib)),
        };
    for (name, value) in old_bound {
        scib.set(name.clone(), value);
    }
    Ok(result)
}

pub fn let_f(scib: &mut Scib) -> Result<Rc<Value>> {
    let binds_list = scib.unbind("_let-binds").unwrap();
    let body = Body::Lisp(
        match Rc::try_unwrap(scib.unbind("_let-body").unwrap()) {
            Ok(body) =>
                match body {
                    Value::List(l) => l,
                    _ => unreachable!(),
                },
            Err(body) =>
                match *body {
                    Value::List(ref l) => l.clone(),
                    _ => unreachable!(),
                },
        });
    let binds_unparsed =
        match *binds_list {
            Value::List(ref l) => l,
            _ => return Err(Error::new(ErrorKind::InvalidInput,
                                       format!("let requires a list of bindings as its first parameter, found '{:?}'", *binds_list))),
        };
    let mut binds: Vec<(String, Rc<Value>)> = Vec::with_capacity(binds_unparsed.len());
    for bind in binds_unparsed {
        match **bind {
            Value::List(ref l) => {
                if l.len() == 2 {
                    let n = match *l[0] {
                        Value::Label(ref l) => l.clone(),
                        _ => return Err(Error::new(ErrorKind::InvalidInput,
                                                   format!("let requires a binding to have a label as it's name"))),
                    };
                    binds.push((n, l[1].clone()));
                } else {
                    return Err(Error::new(ErrorKind::InvalidInput,
                                          format!("let requires a binding to have a name and a value only")));
                }
            },
            Value::Label(ref l) => {
                binds.push((l.clone(), Rc::new(Value::Nil)));
            },
            _ => return Err(Error::new(ErrorKind::InvalidInput,
                                       format!("let requires each binding to fit the form '(name value)' or 'name', found {:?}", *bind))),
        }
    }

    let_vars(scib, binds.iter().cloned(), &body)
}

pub fn setq_f(scib: &mut Scib) -> Result<Rc<Value>> {
    let label = match *scib.unbind("_setq-label").unwrap() {
        Value::Label(ref l) => l.clone(),
        ref label => return Err(Error::new(ErrorKind::InvalidInput,
                                           format!("setq's first argument must be a label, found '{:?}'", label))),
    };
    let value = scib.unbind("_setq-value").unwrap();
    let value = try!(eval(scib, &value));
    scib.set(label, value.clone());
    Ok(Rc::new(Value::Quote(value)))
}

pub fn equalsign_f(scib: &mut Scib) -> Result<Rc<Value>> {
    let rest = match *scib.unbind("_=-rest").unwrap() {
        Value::List(ref l) => l.clone(),
        _ => panic!(),
    };
    let first = scib.unbind("_=-first").unwrap();
    if rest.iter().all(|v| first == *v) {
        Ok(Rc::new(Value::True))
    } else {
        Ok(Rc::new(Value::Nil))
    }
}

pub fn sum_f(scib: &mut Scib) -> Result<Rc<Value>> {
    let values = match *scib.unbind("_+").unwrap() {
        Value::List(ref l) => l.clone(),
        ref label => panic!(),
    };
    let mut res = 0.0;
    for value in values {
        res += match *value {
            Value::Number(n) => n,
            ref label => return Err(Error::new(ErrorKind::InvalidInput,
                                               format!("+'s arguments must all be numbers, found '{:?}'", label))),
        };
    }
    Ok(Rc::new(Value::Number(res)))
}

pub fn difference_f(scib: &mut Scib) -> Result<Rc<Value>> {
    let mut res = match *scib.unbind("_--positive").unwrap() {
        Value::Number(n) => n,
        ref label => return Err(Error::new(ErrorKind::InvalidInput,
                                           format!("-'s arguments must all be numbers, found '{:?}'", label))),
    };
    let mut values = match *scib.unbind("_--negatives").unwrap() {
        Value::List(ref l) => l.clone(),
        ref label => panic!(),
    };
    for value in values {
        res -= match *value {
            Value::Number(n) => n,
            ref label => return Err(Error::new(ErrorKind::InvalidInput,
                                               format!("-'s arguments must all be numbers, found '{:?}'", label))),
        };
    }
    Ok(Rc::new(Value::Number(res)))
}

pub fn product_f(scib: &mut Scib) -> Result<Rc<Value>> {
    let values = match *scib.unbind("_*").unwrap() {
        Value::List(ref l) => l.clone(),
        ref label => panic!(),
    };
    let mut res = 1.0;
    for value in values {
        res *= match *value {
            Value::Number(n) => n,
            ref label => return Err(Error::new(ErrorKind::InvalidInput,
                                               format!("+'s arguments must all be numbers, found '{:?}'", label))),
        };
    }
    Ok(Rc::new(Value::Number(res)))
}

pub fn quotient_f(scib: &mut Scib) -> Result<Rc<Value>> {
    let mut res = match *scib.unbind("_/-numerator").unwrap() {
        Value::Number(n) => n,
        ref label => return Err(Error::new(ErrorKind::InvalidInput,
                                           format!("/'s arguments must all be numbers, found '{:?}'", label))),
    };
    let values = match *scib.unbind("_/-denominator").unwrap() {
        Value::List(ref l) => l.clone(),
        ref label => panic!(),
    };
    for value in values {
        res /= match *value {
            Value::Number(n) => n,
            ref label => return Err(Error::new(ErrorKind::InvalidInput,
                                               format!("/'s arguments must all be numbers, found '{:?}'", label))),
        };
    }
    Ok(Rc::new(Value::Number(res)))
}

pub fn list_f(scib: &mut Scib) -> Result<Rc<Value>> {
    Ok(scib.unbind("_list-rest").unwrap())
}

pub fn progn_f(scib: &mut Scib) -> Result<Rc<Value>> {
    let rest = match *scib.unbind("_progn-rest").unwrap() {
        Value::List(ref l) => l.clone(),
        _ => panic!(),
    };
    Ok(Rc::new(Value::Quote(try!(progn(scib, &rest)))))
}

pub fn progn(scib: &mut Scib, exprs: &Vec<Rc<Value>>) -> Result<Rc<Value>> {
    let mut res = Rc::new(Value::Nil);
    for expr in exprs {
        res = try!(eval(scib, expr));
    }
    Ok(res)
}

pub fn if_f(scib: &mut Scib) -> Result<Rc<Value>> {
    let cond = match *scib.unbind("_if-cond").unwrap() {
        Value::Nil => false,
        _ => true,
    };
    let iftrue = scib.unbind("_if-iftrue").unwrap();
    let iffalse = match *scib.unbind("_if-iffalse").unwrap() {
        Value::List(ref l) => l.clone(),
        _ => panic!(),
    };
    if cond {
        Ok(iftrue)
    } else {
        Ok(Rc::new(Value::Quote(try!(progn(scib, &iffalse)))))
    }
}

fn define_parse_params(l: &Vec<Rc<Value>>) -> Result<(String, Parameters)> {
    let mut required = Vec::new();
    let mut optional = Vec::new();
    let mut rest = None;
    let mut iter = l.iter();
    let name =
        try!(
            try!(iter.next().ok_or(
                Error::new(ErrorKind::InvalidInput,
                           format!("A name is required."))))
                .as_label())
        .clone();
    while let Some(param) = iter.next() {
        let param = try!(param.as_label());
        if param == "&optional" {
            while let Some(param) = iter.next() {
                let param = try!(param.as_label());
                if param == "&rest" {
                    rest = Some(try!(try!(iter.next().ok_or(
                        Error::new(ErrorKind::InvalidInput,
                                   format!("&rest must be named."))))
                                     .as_label()).clone());
                    if iter.next().is_some() {
                        return Err(Error::new(ErrorKind::InvalidInput,
                                              format!("&rest cannot be named multiple times.")));
                    }
                    break;
                } else {
                    optional.push(param.clone());
                }
            }
            if optional.is_empty() {
                return Err(Error::new(ErrorKind::InvalidInput,
                                      format!("No optional arguments given.")));
            }
            break;
        } else if param == "&rest" {
            rest = Some(try!(try!(iter.next().ok_or(Error::new(ErrorKind::InvalidInput,
                                                               format!("&rest must be named."))))
                             .as_label()).clone());
            if iter.next().is_some() {
                return Err(Error::new(ErrorKind::InvalidInput,
                                      format!("&rest cannot be named multiple times.")));
            }
            break;
        } else {
            required.push(param.clone());
        }
    }
    Ok((name, Parameters {
        required, optional, rest,
    }))
}

pub fn define_f(scib: &mut Scib) -> Result<Rc<Value>> {
    let value = scib.unbind("_define-value").unwrap().unwrap_list().clone();
    match *scib.unbind("_define-name").unwrap() {
        Value::Label(ref name) => {
            let value = try!(progn(scib, &value));
            scib.set(name.clone(), value.clone());
            Ok(value)
        },
        Value::List(ref l) => {
            let (name, params) = try!(define_parse_params(l));
            let value = Rc::new(Value::Function(Rc::new(
                Function {
                    params,
                    body: Body::Lisp(value),
                })));
            scib.set(name, value.clone());
            Ok(value)
        },
        _ => Err(Error::new(ErrorKind::InvalidInput,
                            format!("Definition name must be a list or label."))),
    }
}

pub fn defmacro_f(scib: &mut Scib) -> Result<Rc<Value>> {
    let value = scib.unbind("_defmacro-value").unwrap().unwrap_list().clone();
    match *scib.unbind("_defmacro-name").unwrap() {
        Value::List(ref l) => {
            let (name, params) = try!(define_parse_params(l));
            let value = Rc::new(Value::Macro(Rc::new(
                Macro {
                    params,
                    body: Body::Lisp(value),
                })));
            scib.set(name, value.clone());
            Ok(value)
        },
        _ => Err(Error::new(ErrorKind::InvalidInput,
                            format!("Macro parameters must be a list."))),
    }
}

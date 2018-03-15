use types::*;
use instance::*;
use builtins::*;
use std::rc::Rc;
use std::io::{Result, Error, ErrorKind};

fn eval_function(scib: &mut Scib, f: &Function, unevaled_args: &Vec<Rc<Value>>) -> Result<Rc<Value>> {
}

fn eval_macro(scib: &mut Scib, m: &Macro, unevaled_args: &Vec<Rc<Value>>) -> Result<Rc<Value>> {
}

fn eval_function_or_macro(scib: &mut Scib, unevaled_args: &Vec<Rc<Value>>, output: &mut Vec<Rc<Value>>) -> Result<()> {
    let o = Vec::with_capacity(1);
    try!(eval(scib, &unevaled_args[0], &mut o));
    let first = o.remove(0);
    if !o.is_empty() {
        
    }
    match *first {
        Value::Function(ref f) => {
            try!(f.params.check_params_len(unevaled_args.len()));
            let mut evaled_args = Vec::with_capacity(unevaled_args.len());
            for unevaled_arg in unevaled_args {
                evaled_args.push(try!(eval(scib, unevaled_arg)));
            }
            let_vars(scib, f.params.bind_params(evaled_args.into_iter()).into_iter(), &f.body)
        },
        Value::Macro(ref m) => {
            try!(m.params.check_params_len(unevaled_args.len()));
            let result = try!(let_vars(scib, m.params.bind_params(unevaled_args.iter().cloned()).into_iter(), &m.body));
            eval(scib, &result)
        },
        _ => panic!(),
    }
}

pub fn eval_backquote(scib: &mut Scib, v: &Rc<Value>, in_backquote: i32) -> Result<Rc<Value>> {
    match **v {
        Value::True |
        Value::Nil |
        Value::Number(_) |
        Value::String(_) |
        Value::Function(_) |
        Value::Macro(_) |
        Value::Label(_) => Ok(v.clone()),
        Value::Unquote(ref v) => {
            if in_backquote == 1 {
                eval(scib, v)
            } else {
                eval_backquote(scib, v, in_backquote - 1)
            }
        },
        Value::UnquoteList(ref v) => {
            if in_backquote == 1 {
                eval(scib, v)
            }
        },
        Value::List(ref list) => {
            Ok(Rc::new(Value::List(try!(list.iter().map(
                |v| eval_backquote(scib, v, in_backquote))
                                   .collect()))))
        },
        Value::Quote(ref v) => {
            Ok(Rc::new(Value::Quote(try!(eval_backquote(scib, v, in_backquote)))))
        },
        Value::Backquote(ref v) => {
            Ok(Rc::new(Value::Quote(try!(eval_backquote(scib, v, in_backquote + 1)))))
        }
    }
}

pub fn eval(scib: &mut Scib, v: &Rc<Value>, output: &mut Vec<Rc<Value>>) -> Result<()> {
    output.push(match **v {
        Value::True |
        Value::Nil |
        Value::Number(_) |
        Value::String(_) |
        Value::Function(_) |
        Value::Macro(_) => {
            v.clone()
        },
        Value::Label(ref label) => {
            try!(scib.lookup(label))
        },
        Value::List(ref list) => {
            if list.is_empty() {
                Rc::new(Value::Nil)
            } else {
                return eval_function_or_macro(scib, list, output)
            }
        },
        Value::Backquote(ref v) => {
            try!(eval_backquote(scib, v, 1))
        },
        Value::Unquote(ref v) => {
            Err(Error::new(ErrorKind::InvalidInput,
                           format!("Unquote without accompanying backquote")))
        },
        Value::Quote(ref v) => {
            v.clone()
        },
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lex::lex;
    use parse::parse;

    fn unwrap_1<T>(mut v: Vec<T>) -> T {
        assert!(v.len() == 1);
        v.remove(0)
    }

    #[test]
    fn test_eval_1() {
        let mut instance = Scib::new();
        assert_eq!(Value::Number(123.0),
                   *eval(&mut instance,
                         &unwrap_1(parse(lex("123".chars().fuse()).unwrap())
                                   .unwrap()))
                   .unwrap());
    }

    #[test]
    fn test_eval_2() {
        let mut instance = Scib::new();
        assert_eq!(Value::String(String::from("HI")),
                   *eval(&mut instance,
                         &unwrap_1(parse(lex("\"HI\"".chars().fuse()).unwrap())
                                   .unwrap()))
                   .unwrap());
    }

    #[test]
    fn test_eval_3() {
        let mut instance = Scib::new();
        assert_eq!(Value::Number(123.0),
                   *eval(&mut instance,
                         &unwrap_1(parse(lex("(setq xo 123)".chars().fuse()).unwrap())
                                   .unwrap()))
                   .unwrap());
        assert_eq!(Value::Number(123.0),
                   **instance.get("xo").unwrap());
    }

    #[test]
    fn test_eval_4() {
        let mut instance = Scib::new();
        assert_eq!(Value::Label(String::from("y")),
                   *eval(&mut instance,
                         &unwrap_1(parse(lex("(setq x 'y)".chars().fuse()).unwrap())
                                   .unwrap()))
                   .unwrap());
        assert_eq!(Value::Label(String::from("y")),
                   **instance.get("x").unwrap());
    }

    #[test]
    fn test_eval_5() {
        let mut instance = Scib::new();
        assert_eq!(Value::List(vec![Rc::new(Value::Number(1.0)), Rc::new(Value::Number(3.0))]),
                   *instance.eval("(setq x 2)(setq y 3)(setq x 1)(list x y)").unwrap());
    }

    #[test]
    fn test_eval_6() {
        let mut instance = Scib::new();
        assert_eq!(Value::Number(7.0),
                   *instance.eval("(progn(setq x 1)(setq y 2)(setq z 3)(+ x (* y z)))").unwrap());
    }

    #[test]
    fn test_eval_7() {
        let mut instance = Scib::new();
        assert_eq!(Value::Function(Rc::new(Function {
            params: Parameters {
                required: vec![String::from("x")],
                optional: vec![],
                rest: None,
            },
            body: Body::Lisp(vec![
                Rc::new(Value::List(vec![
                    Rc::new(Value::Label(String::from("+"))),
                    Rc::new(Value::Number(1.0)),
                    Rc::new(Value::Label(String::from("x"))),
                ]))]),
        })),
                   *instance.eval("(define (f x) (+ 1 x))").unwrap());
        assert_eq!(Value::Number(23.0),
                   *instance.eval("(f 22)").unwrap());
    }

    #[test]
    fn test_eval_8() {
        let mut instance = Scib::new();
        assert_eq!(Value::Number(13.0),
                   *instance.eval("(- (/ 30 2 3) -8)").unwrap());
    }

    #[test]
    fn test_eval_9() {
        let mut instance = Scib::new();
        assert_eq!(Value::Nil,
                   *instance.eval("(when (= 1 3) 13 23)").unwrap());
    }
}

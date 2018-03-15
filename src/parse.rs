use lex::Token;
use types::*;
use std::rc::Rc;
use std::io::{Result, Error, ErrorKind};

fn parse_token<I: Iterator<Item = Token>>(token: Token, tokens: &mut I, in_paren: bool, in_backquote: i32) -> Result<Option<Rc<Value>>> {
    match token {
        Token::Value(v) => {
            Ok(Some(Rc::new(v)))
        },
        Token::Backquote => {
            fn err() -> Error { Error::new(ErrorKind::InvalidInput,
                                           format!("Backquote without accompanying quoted data")) };
            let v = try!(tokens.next().ok_or_else(err));
            let e = try!(try!(parse_token(v, tokens, in_paren, in_backquote + 1)).ok_or_else(err));
            Ok(Some(Rc::new(Value::Backquote(e))))
        },
        Token::Unquote => {
            if in_backquote <= 0 {
                return Err(Error::new(ErrorKind::InvalidInput,
                                      format!("Unquote without accompanying backquote")));
            }
            fn err() -> Error { Error::new(ErrorKind::InvalidInput,
                                           format!("Unquote without accompanying quoted data")) };
            let v = try!(tokens.next().ok_or_else(err));
            let e = try!(try!(parse_token(v, tokens, in_paren, in_backquote - 1)).ok_or_else(err));
            Ok(Some(Rc::new(Value::Unquote(e))))
        },
        Token::UnquoteList => {
            if in_backquote <= 0 {
                return Err(Error::new(ErrorKind::InvalidInput,
                                      format!("Unquote without accompanying backquote")));
            }
            fn err() -> Error { Error::new(ErrorKind::InvalidInput,
                                           format!("Unquote without accompanying quoted data")) };
            let v = try!(tokens.next().ok_or_else(err));
            let e = try!(try!(parse_token(v, tokens, in_paren, in_backquote - 1)).ok_or_else(err));
            Ok(Some(Rc::new(Value::UnquoteList(e))))
        },
        Token::Quote => {
            fn err() -> Error { Error::new(ErrorKind::InvalidInput,
                                           format!("Quote without accompanying quoted data")) };
            let v = try!(tokens.next().ok_or_else(err));
            let e = try!(try!(parse_token(v, tokens, in_paren, in_backquote)).ok_or_else(err));
            Ok(Some(Rc::new(Value::Quote(e))))
        },
        Token::OpenParen => {
            let mut e = Vec::new();
            try!(parse_(tokens, &mut e, true, in_backquote));
            Ok(Some(Rc::new(Value::List(e))))
        },
        Token::CloseParen =>
            if in_paren {
                Ok(None)
            } else {
                Err(Error::new(ErrorKind::InvalidInput,
                               format!("Closing parenthesis without accompanying open parenthesis.")))
            },
    }
    
}

fn parse_<I: Iterator<Item = Token>>(tokens: &mut I, exprs: &mut Vec<Rc<Value>>, in_paren: bool, in_backquote: i32) -> Result<()> {
    while let Some(token) = tokens.next() {
        match try!(parse_token(token, tokens, in_paren, in_backquote)) {
            Some(t) => exprs.push(t),
            None => return Ok(()),
        }
    }
    if in_paren {
        Err(Error::new(ErrorKind::InvalidInput, "Open parenthesis without accompanying closing parenthesis.".to_owned()))
    } else {
        Ok(())
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Rc<Value>>> {
    let mut exprs = Vec::new();
    try!(parse_(&mut tokens.into_iter(), &mut exprs, false, 0));
    Ok(exprs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_1() {
        assert_eq!(
            vec![Rc::new(Value::List(vec![]))],
            parse(vec![Token::OpenParen, Token::CloseParen]).unwrap());
    }

    #[test]
    fn test_parse_2() {
        assert_eq!(
            vec![Rc::new(Value::List(vec![Rc::new(Value::Label("abc".to_owned()))]))],
            parse(vec![Token::OpenParen,
                       Token::Value(Value::Label("abc".to_owned())),
                       Token::CloseParen]).unwrap());
    }

    #[test]
    fn test_parse_3() {
        assert_eq!(
            vec![Rc::new(Value::List(vec![Rc::new(Value::Label("abc".to_owned()))])),
                 Rc::new(Value::List(vec![Rc::new(Value::Label("abc".to_owned()))]))],
            parse(vec![Token::OpenParen,
                       Token::Value(Value::Label("abc".to_owned())),
                       Token::CloseParen,
                       Token::OpenParen,
                       Token::Value(Value::Label("abc".to_owned())),
                       Token::CloseParen]).unwrap());
    }

    #[test]
    fn test_parse_4() {
        assert_eq!(
            vec![
                Rc::new(Value::List(vec![
                    Rc::new(Value::Number(13.0)),
                    Rc::new(Value::List(vec![
                        Rc::new(Value::Number(13.0))])),
                    Rc::new(Value::Number(13.0))])),
                Rc::new(Value::Label("abc".to_string())),
            ],
            parse(vec![
                Token::OpenParen,
                Token::Value(Value::Number(13.0)),
                Token::OpenParen,
                Token::Value(Value::Number(13.0)),
                Token::CloseParen,
                Token::Value(Value::Number(13.0)),
                Token::CloseParen,
                Token::Value(Value::Label("abc".to_string()))]).unwrap());
    }

    #[test]
    fn test_parse_5() {
        assert_eq!(
            vec![
                Rc::new(Value::Quote(
                    Rc::new(Value::List(vec![
                        Rc::new(Value::Number(13.0)),
                        Rc::new(Value::List(vec![
                            Rc::new(Value::Number(13.0))])),
                        Rc::new(Value::Number(13.0))])))),
                Rc::new(Value::Quote(
                    Rc::new(Value::Label("abc".to_string())))),
            ],
            parse(vec![
                Token::Quote,
                Token::OpenParen,
                Token::Value(Value::Number(13.0)),
                Token::OpenParen,
                Token::Value(Value::Number(13.0)),
                Token::CloseParen,
                Token::Value(Value::Number(13.0)),
                Token::CloseParen,
                Token::Quote,
                Token::Value(Value::Label("abc".to_string()))]).unwrap());
    }

    #[test]
    fn test_parse_6() {
        use lex::lex;
        assert_eq!(vec![
            Rc::new(Value::List(vec![
                Rc::new(Value::Label("-".to_string())),
                Rc::new(Value::List(vec![
                    Rc::new(Value::Label("/".to_string())),
                    Rc::new(Value::Number(30.0)),
                    Rc::new(Value::Number(2.0)),
                    Rc::new(Value::Number(3.0))])),
                Rc::new(Value::Number(-8.0))]))],
            parse(lex("(- (/ 30 2 3) -8)".chars().fuse()).unwrap()).unwrap());
    }

    #[test]
    fn test_parse_panic_1() {
        assert!(parse(vec![Token::CloseParen]).is_err());
    }

    #[test]
    fn test_parse_panic_2() {
        assert!(parse(vec![Token::OpenParen]).is_err());
    }

    #[test]
    fn test_parse_panic_3() {
        assert!(parse(vec![Token::OpenParen, Token::OpenParen, Token::CloseParen]).is_err());
    }

    #[test]
    fn test_parse_panic_4() {
        assert!(parse(vec![Token::OpenParen,
                           Token::Value(Value::Number(13.0)),
                           Token::OpenParen,
                           Token::Value(Value::Number(13.0)),
                           Token::CloseParen,
                           Token::Value(Value::Number(13.0))]).is_err());
    }

    #[test]
    fn test_parse_panic_5() {
        assert!(parse(vec![Token::Value(Value::Number(13.0)),
                           Token::Quote]).is_err());
    }

    #[test]
    fn test_parse_panic_6() {
        assert!(parse(vec![Token::OpenParen,
                           Token::Value(Value::Number(13.0)),
                           Token::Quote,
                           Token::CloseParen]).is_err());
    }
}

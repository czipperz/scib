use types::Value;
use std::io;
use std::io::{Error, ErrorKind};

#[derive(Debug, PartialEq)]
pub enum Token {
    Value(Value),
    Quote,
    Backquote,
    Unquote,
    UnquoteList,
    OpenParen,
    CloseParen,
}

fn is_separator_char(c: char) -> bool { c == '(' || c == ')' || c.is_whitespace() }

fn is_label_character(ch: char) -> bool {
    !(ch.is_whitespace() || ch == '`' || ch == '\'' || ch == '(' || ch == ')' || ch == '"')
}

fn label_to_token(s: String) -> Token {
    use std::str::FromStr;
    Token::Value(
        if s == "t" {
            Value::True
        } else if s == "nil" {
            Value::Nil
        } else if let Ok(n) = f64::from_str(s.as_str()) {
            Value::Number(n)
        } else {
            Value::Label(s)
        })
}

pub fn lex<I: Iterator<Item = char>>(mut iter: I) -> io::Result<Vec<Token>> {
    let mut vec = Vec::new();
    let mut ch = match iter.next() {
        Some(ch) => ch,
        None => return Ok(vec),
    };
    'outer: loop {
        if ch == '(' {
            vec.push(Token::OpenParen);
        } else if ch == ')' {
            vec.push(Token::CloseParen);
        } else if ch == '"' {
            let mut s = String::new();
            while let Some(cn) = iter.next() {
                if cn == '\\' {
                    s.push(match iter.next() {
                        Some('\\') => '\\',
                        Some('"') => '"',
                        Some('t') => '\t',
                        Some('n') => '\n',
                        Some(x) => return Err(Error::new(ErrorKind::InvalidInput,
                                                         format!("Invalid escape sequence `\\{}'.", x))),
                        None => return Err(Error::new(ErrorKind::InvalidInput,
                                                      format!("Nothing to escape and the string isn't closed."))),
                    });
                } else if cn == '"' {
                    vec.push(Token::Value(Value::String(s)));
                    match iter.next() {
                        Some(c) => {
                            ch = c;
                            continue 'outer;
                        },
                        None => break 'outer,
                    }
                } else {
                    s.push(cn);
                }
            }
            return Err(Error::new(ErrorKind::InvalidInput,
                                  format!("String not ended")));
        } else if ch == '\'' {
            vec.push(Token::Quote);
        } else if ch == '`' {
            vec.push(Token::Backquote);
        } else if ch == ',' {
            match iter.next() {
                Some('@') => vec.push(Token::UnquoteList),
                Some(c) => {
                    vec.push(Token::Unquote);
                    ch = c;
                    continue;
                },
                None => break,
            }
        } else if ch.is_whitespace() {
        } else if is_label_character(ch) {
            let mut s = String::new();
            s.push(ch);
            let mut c = iter.next();
            while c.is_some() {
                let cn = c.unwrap();
                if is_label_character(cn) {
                    s.push(cn);
                } else if is_separator_char(cn) {
                    vec.push(label_to_token(s));
                    ch = cn;
                    continue 'outer;
                } else {
                    return Err(Error::new(ErrorKind::InvalidInput,
                                          format!("Invalid character encountered after a number or label '{}'.  Consider adding a space.", cn)));
                }
                c = iter.next();
            }
            // end of file
            vec.push(label_to_token(s));
            break;
        } else {
            unreachable!();
        }
        match iter.next() {
            Some(c) => ch = c,
            None => break,
        }
    }
    Ok(vec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lex_1() {
        assert_eq!(
            vec![Token::OpenParen, Token::Value(Value::Label("setq".to_string())), Token::Value(Value::Number(13.0)), Token::CloseParen],
            lex("(setq 13)".chars().fuse()).unwrap());
    }

    #[test]
    fn test_lex_2() {
        assert_eq!(
            vec![Token::OpenParen, Token::Value(Value::Label("setq".to_string())), Token::Value(Value::Number(0.13)), Token::CloseParen],
            lex("(setq .13)".chars().fuse()).unwrap());
    }

    #[test]
    fn test_lex_3() {
        assert_eq!(
            vec![Token::OpenParen, Token::Value(Value::Label("setq".to_string())), Token::Value(Value::Number(13.123)), Token::CloseParen],
            lex("(setq 13.123)".chars().fuse()).unwrap());
    }

    #[test]
    fn test_lex_4() {
        assert_eq!(
            vec![Token::OpenParen, Token::Value(Value::Label("setq".to_string())), Token::Value(Value::Number(13.0)), Token::CloseParen],
            lex("(setq 13.)".chars().fuse()).unwrap());
    }

    #[test]
    fn test_lex_5() {
        assert_eq!(
            vec![Token::OpenParen, Token::Value(Value::Label("setq".to_string())), Token::Value(Value::Number(13.0)), Token::Value(Value::Label("abcd".to_string())), Token::CloseParen],
            lex("(setq 13. abcd)".chars().fuse()).unwrap());
    }

    #[test]
    fn test_lex_6() {
        assert_eq!(
            vec![Token::Value(Value::Number(13.0))],
            lex("13".chars().fuse()).unwrap());
    }

    #[test]
    fn test_lex_7() {
        assert_eq!(
            vec![Token::OpenParen, Token::Value(Value::Label("setq".to_string())), Token::Value(Value::Number(13.0)), Token::OpenParen, Token::Value(Value::Number(0.123)), Token::CloseParen, Token::CloseParen],
            lex("(setq 13. (.123))".chars().fuse()).unwrap());
    }

    #[test]
    fn test_lex_8() {
        assert_eq!(
            vec![Token::Value(Value::True), Token::Value(Value::Nil)],
            lex("t nil".chars().fuse()).unwrap());
    }

    #[test]
    fn test_lex_string_1() {
        assert_eq!(
            vec![Token::Value(Value::String("ab12390noeu0voaeut,.hp\"oeuhtn".to_owned()))],
            lex("\"ab12390noeu0voaeut,.hp\\\"oeuhtn\"".chars().fuse()).unwrap());
    }

    #[test]
    fn test_lex_string_2() {
        assert_eq!(
            vec![Token::Value(Value::String("\t\n\\n".to_owned()))],
            lex("\"\\t\\n\\\\n\"".chars().fuse()).unwrap());
    }

    #[test]
    fn test_lex_string_3() {
        assert_eq!(
            vec![Token::Value(Value::String("".to_owned()))],
            lex("\"\"".chars().fuse()).unwrap());
    }

    #[test]
    fn test_lex_string_4() {
        assert_eq!(
            vec![Token::Value(Value::String("abcdefg".to_owned()))],
            lex("\"abcdefg\"".chars().fuse()).unwrap());
    }

    #[test]
    fn test_lex_label() {
        assert_eq!(
            vec![Token::Value(Value::Label("ns:xx/oeu-aoeu++".to_owned()))],
            lex("ns:xx/oeu-aoeu++".chars().fuse()).unwrap());
    }

    #[test]
    fn test_lex_panic_1() {
        assert!(lex("13`()".chars().fuse()).is_err());
    }

    #[test]
    fn test_lex_panic_2() {
        assert!(lex("13.'a)".chars().fuse()).is_err());
    }

    #[test]
    fn test_lex_panic_3() {
        assert!(lex("\"\\a\"".chars().fuse()).is_err());
    }

    #[test]
    fn test_lex_panic_4() {
        assert!(lex("\"\\\"".chars().fuse()).is_err());
    }
}

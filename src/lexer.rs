use std::iter::Peekable;
use std::str::Chars;

use crate::parser::ParseError;

#[derive(Debug)]
struct UnmatchedDelimiterError;

#[derive(Debug)]
pub enum Token {
    Word(String),
    SubShell(String),
    Variable(String),
    Pipe,
    PipeBoth,
    RedirOut,
    RedirErr,
    RedirBoth,
    AndThen,
    AndThenIf,
}

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            chars: input.chars().peekable(),
        }
    }

    fn skip_whitespace(&mut self) {
        while self.chars.next_if(|c| c.is_whitespace()).is_some() {}
    }

    fn lex_word(&mut self) -> Result<Token, ParseError> {
        let mut word = String::new();
        let mut in_single_quotes = false;
        let mut in_double_quotes = false;

        while let Some(&c) = self.chars.peek() {
            if in_single_quotes {
                self.chars.next();
                if c == '\'' {
                    in_single_quotes = false;
                } else {
                    word.push(c);
                }
            } else if in_double_quotes {
                self.chars.next();
                if c == '"' {
                    in_double_quotes = false;
                } else {
                    word.push(c);
                }
            } else if c.is_whitespace() || c == '|' {
                break;
            } else if c == '\'' {
                self.chars.next();
                in_single_quotes = true;
            } else if c == '"' {
                self.chars.next();
                in_double_quotes = true;
            } else if c == ';' || c == '>' || c == '&' {
                break;
            } else if c.is_ascii_digit()
                && self
                    .chars
                    .clone()
                    .next_if(|&next_c| next_c == '>')
                    .is_some()
            {
                // Only break on digits if they're followed by '>', like "2>"
                break;
            } else {
                self.chars.next();
                word.push(c);
            }
        }

        if in_single_quotes || in_double_quotes {
            return Err(ParseError::UnterminatedStringLiteral);
        }

        if !word.is_empty() {
            Ok(Token::Word(word))
        } else {
            Err(ParseError::NotFound)
        }
    }

    fn lex_and_then(&mut self) -> Option<Token> {
        let mut iter = self.chars.clone();

        if let Some(&c) = iter.peek() {
            if c == '&' {
                iter.next();

                if let Some(&next_c) = iter.peek() {
                    if next_c == '&' {
                        self.chars.next();
                        self.chars.next();
                        return Some(Token::AndThenIf);
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else if c == ';' {
                self.chars.next();
                return Some(Token::AndThen);
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    fn lex_redirection(&mut self) -> Option<Token> {
        let mut iter = self.chars.clone();
        let mut redir = String::new();

        if let Some(&c) = iter.peek() {
            if c == '1' || c == '2' || c == '&' {
                redir.push(c);
                iter.next();
            }
        }

        if let Some(&c) = iter.peek() {
            if c == '>' {
                redir.push(c);
                iter.next();

                if let Some(&next_c) = iter.peek() {
                    if next_c == '>' {
                        redir.push(next_c);
                        iter.next();
                    }
                }
            } else {
                return None;
            }
        } else {
            return None;
        }

        match redir.as_str() {
            ">" | "1>" | ">>" | "1>>" => {
                for _ in 0..redir.len() {
                    self.chars.next();
                }
                Some(Token::RedirOut)
            }
            "2>" | "2>>" => {
                for _ in 0..redir.len() {
                    self.chars.next();
                }
                Some(Token::RedirErr)
            }
            "&>" | "&>>" => {
                for _ in 0..redir.len() {
                    self.chars.next();
                }
                Some(Token::RedirBoth)
            }
            _ => None,
        }
    }

    fn lex_pipe(&mut self) -> Option<Token> {
        let mut iter = self.chars.clone();

        if let Some(c) = iter.next() {
            if c == '|' {
                if let Some(&next_c) = iter.peek() {
                    if next_c == '&' {
                        self.chars.next();
                        self.chars.next();
                        return Some(Token::PipeBoth);
                    }
                }
                self.chars.next();
                return Some(Token::Pipe);
            }
        }
        None
    }

    // add a custom error type maybe
    fn subshell_inner(&mut self) -> Result<String, ParseError> {
        let mut inner_string = String::new();
        let mut open_parens = 1;
        let mut matched = false;

        // We don't want that first '('
        self.chars.next();

        while let Some(&c) = self.chars.peek() {
            match c {
                ')' => {
                    if open_parens == 0 {
                        break;
                    } else if open_parens == 1 {
                        self.chars.next();
                        matched = true;
                        break;
                    } else {
                        open_parens -= 1;
                    }
                }
                '(' => {
                    open_parens += 1;
                }
                _ => {}
            }
            self.chars.next();
            inner_string.push(c);
        }
        if matched {
            Ok(inner_string)
        } else {
            Err(ParseError::UnmatchedDelimiterError)
        }
    }

    fn lex_subshell(&mut self) -> Result<Token, ParseError> {
        let mut iter = self.chars.clone();

        if let Some(c) = iter.next() {
            if c == '$' {
                if let Some(&next_c) = iter.peek() {
                    if next_c == '(' {
                        self.chars.next();
                        let inner_string = self.subshell_inner()?;
                        return Ok(Token::SubShell(inner_string));
                    }
                }
            } else if c == '(' {
                let inner_string = self.subshell_inner()?;
                return Ok(Token::SubShell(inner_string));
            }
        }
        Err(ParseError::NotFound)
    }

    /*
     * name -  A  word  consisting  only  of alphanumeric characters and underscores,
     *         and beginning with an alphabetic character or an  underscore.  Also
     *         referred to as an identifier
     */

    fn lex_variable(&mut self) -> Result<Token, ParseError> {
        if let Some(&c) = self.chars.peek() {
            if c == '$' {
                self.chars.next();

                if !self
                    .chars
                    .peek()
                    .is_some_and(|&ch| ch.is_alphabetic() || ch == '_')
                {
                    return Err(ParseError::InvalidVariable);
                }

                let variable_name: String = self
                    .chars
                    .clone()
                    .take_while(|&ch| ch.is_alphanumeric() || ch == '_')
                    .collect();

                for _ in 0..variable_name.len() {
                    self.chars.next();
                }
                return Ok(Token::Variable(variable_name));
            }
        }
        Err(ParseError::NotFound)
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.skip_whitespace();

        if let Some(token) = self.lex_redirection() {
            return Some(Ok(token));
        }

        if let Some(token) = self.lex_pipe() {
            return Some(Ok(token));
        }

        if let Some(token) = self.lex_and_then() {
            return Some(Ok(token));
        }

        match self.lex_subshell() {
            Ok(token) => return Some(Ok(token)),
            Err(ParseError::NotFound) => (),
            Err(e) => return Some(Err(e)),
        }

        match self.lex_variable() {
            Ok(token) => return Some(Ok(token)),
            Err(ParseError::NotFound) => (),
            Err(e) => return Some(Err(e)),
        }

        match self.lex_word() {
            Ok(token) => return Some(Ok(token)),
            Err(ParseError::NotFound) => None,
            Err(e) => return Some(Err(e)),
        }
    }
}

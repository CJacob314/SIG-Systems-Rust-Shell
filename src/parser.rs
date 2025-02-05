use std::path::PathBuf;
use std::{hint::unreachable_unchecked, iter::Peekable};

use crate::lexer::{Lexer, Token};

#[derive(Debug)]
pub enum ParseError {
    Empty,
    MissingFileName,
    UnmatchedDelimiterError,
    InvalidVariable,
    UnterminatedStringLiteral,
    NotFound,
}

#[derive(Debug)]
pub struct ParseErrors {
    errors: Vec<ParseError>,
}

impl ParseErrors {
    fn into_iter(self) -> impl Iterator<Item = ParseError> {
        self.errors.into_iter()
    }
}

#[derive(Debug, PartialEq)]
pub enum Arg {
    Word(String),
    Variable(String),
    Subshell(Command),
}

#[derive(Debug)]
pub struct Parser<I: Iterator<Item = Result<Token, ParseError>>> {
    tokens: Peekable<I>,
}

#[derive(Debug, PartialEq)]
pub struct Command {
    pub argv: Vec<Arg>,
    pub pipe_to: Option<PipeTo>,
    pub redirect_to: Vec<FileRedir>,
    pub and_then: Option<AndThen>,
}

#[derive(Debug, PartialEq)]
pub struct PipeTo {
    pub pipe_type: RedirType,
    pub target: Box<Command>,
}

#[derive(Debug, PartialEq)]
pub struct AndThen {
    pub conditional: bool,
    pub target: Box<Command>,
}

#[derive(Debug, PartialEq)]
pub enum RedirType {
    Stdout,
    Stderr,
    Both,
}

#[derive(Debug, PartialEq)]
pub struct FileRedir {
    pub redirect_type: RedirType,
    pub target: PathBuf,
}

impl<I: Iterator<Item = Result<Token, ParseError>>> Parser<I> {
    pub fn new(tokens: I) -> Self {
        Parser {
            tokens: tokens.peekable(),
        }
    }
    fn parse_command(&mut self) -> Result<Command, ParseErrors> {
        let mut errors = Vec::new();
        let mut argv = Vec::new();
        let mut pipe_to = None;
        let mut redirect_to = Vec::new();
        let mut and_then = None;

        while let Some(token) = self.tokens.peek() {
            match token {
                Ok(tok) => match tok {
                    Token::Word(_) => match self.tokens.next() {
                        Some(Ok(Token::Word(word))) => argv.push(Arg::Word(word)),
                        Some(Err(e)) => errors.push(e),
                        _ => unsafe { unreachable_unchecked() },
                    },
                    Token::RedirOut | Token::RedirErr | Token::RedirBoth => {
                        let redir_type = self.parse_redir_type();
                        if let Some(Ok(Token::Word(path))) = self.tokens.next() {
                            redirect_to.push(FileRedir {
                                redirect_type: redir_type,
                                target: PathBuf::from(path),
                            });
                        } else {
                            errors.push(ParseError::MissingFileName);
                        }
                    }
                    Token::Pipe | Token::PipeBoth => {
                        let pipe_token = self.tokens.next();
                        let pipe_type = match pipe_token {
                            Some(Ok(Token::Pipe)) => RedirType::Stdout,
                            Some(Ok(Token::PipeBoth)) => RedirType::Both,
                            _ => RedirType::Stdout,
                        };
                        match self.parse_command() {
                            Ok(next_command) => {
                                pipe_to = Some(PipeTo {
                                    pipe_type,
                                    target: Box::new(next_command),
                                });
                            }
                            Err(errs) => {
                                errors.extend(errs.into_iter());
                            }
                        }
                        break;
                    }
                    Token::AndThen => {
                        self.tokens.next();
                        match self.parse_command() {
                            Ok(next_command) => {
                                and_then = Some(AndThen {
                                    target: Box::new(next_command),
                                    conditional: false,
                                });
                            }
                            Err(errs) => {
                                errors.extend(errs.into_iter());
                            }
                        }
                        break;
                    }
                    Token::AndThenIf => {
                        self.tokens.next();
                        match self.parse_command() {
                            Ok(next_command) => {
                                and_then = Some(AndThen {
                                    target: Box::new(next_command),
                                    conditional: true,
                                });
                            }
                            Err(errs) => {
                                errors.extend(errs.into_iter());
                            }
                        }
                        break;
                    }
                    Token::SubShell(command) => {
                        match Command::parse(command) {
                            Ok(command) => argv.push(Arg::Subshell(command)),
                            Err(errs) => errors.extend(errs.into_iter()),
                        }
                        self.tokens.next();
                    }
                    Token::Variable(s) => {
                        argv.push(Arg::Variable({
                            match self.tokens.next().unwrap() {
                                Ok(Token::Variable(v)) => v,
                                _ => unsafe { unreachable_unchecked() },
                            }
                        }));
                    }
                },
                Err(_) => {
                    errors.push(match self.tokens.next() {
                        Some(Err(e)) => e,
                        _ => panic!("what"),
                    });
                }
            }
        }

        if !errors.is_empty() || argv.is_empty() {
            Err(ParseErrors { errors })
        } else {
            Ok(Command {
                argv,
                pipe_to,
                and_then,
                redirect_to,
            })
        }
    }

    fn parse_redir_type(&mut self) -> RedirType {
        match self.tokens.next() {
            Some(Ok(Token::RedirOut)) => RedirType::Stdout,
            Some(Ok(Token::RedirErr)) => RedirType::Stderr,
            Some(Ok(Token::RedirBoth)) => RedirType::Both,
            _ => panic!("peek is Token::Redir, but matched none of Redir Variants"),
        }
    }
}

impl Command {
    pub fn parse(input: impl AsRef<str>) -> Result<Self, ParseErrors> {
        let lexer = Lexer::new(input.as_ref());
        let mut parser = Parser::new(lexer);
        parser.parse_command()
    }
}

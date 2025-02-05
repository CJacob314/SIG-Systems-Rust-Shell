// HACK: This test suite was written by an LLM

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::lexer::*;
    use crate::parser::*;

    fn parse_command(input: &str) -> Option<Command> {
        match Command::parse(input) {
            Ok(s) => Some(s),
            Err(_) => None,
        }
    }

    #[test]
    fn test_word_parsing() {
        let input = "echo hello";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Word("hello".to_string())
            ]
        );
    }

    #[test]
    fn test_subshell_parsing() {
        let input = "echo $(ls -l)";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Subshell(Command {
                    argv: vec![Arg::Word("ls".to_string()), Arg::Word("-l".to_string())],
                    pipe_to: None,
                    redirect_to: Vec::new(),
                    and_then: None,
                })
            ]
        );
    }

    #[test]
    fn test_variable_parsing() {
        let input = "echo $HOME";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Variable("HOME".to_string())
            ]
        );
    }

    #[test]
    fn test_redirection_stdout() {
        let input = "echo hello > output.txt";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Word("hello".to_string())
            ]
        );

        assert_eq!(
            command.redirect_to,
            vec![FileRedir {
                redirect_type: RedirType::Stdout,
                target: PathBuf::from("output.txt")
            }]
        );
    }

    #[test]
    fn test_redirection_stderr() {
        let input = "echo hello 2> error.txt";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Word("hello".to_string())
            ]
        );

        assert_eq!(
            command.redirect_to,
            vec![FileRedir {
                redirect_type: RedirType::Stderr,
                target: PathBuf::from("error.txt")
            }]
        );
    }

    #[test]
    fn test_redirection_both() {
        let input = "echo hello &> output.txt";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Word("hello".to_string())
            ]
        );

        assert_eq!(
            command.redirect_to,
            vec![FileRedir {
                redirect_type: RedirType::Both,
                target: PathBuf::from("output.txt")
            }]
        );
    }

    #[test]
    fn test_pipe() {
        let input = "echo hello | grep world";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Word("hello".to_string())
            ]
        );

        assert_eq!(
            command.pipe_to,
            Some(PipeTo {
                pipe_type: RedirType::Stdout,
                target: Box::new(Command {
                    argv: vec![
                        Arg::Word("grep".to_string()),
                        Arg::Word("world".to_string())
                    ],
                    pipe_to: None,
                    redirect_to: Vec::new(),
                    and_then: None,
                })
            })
        );
    }

    #[test]
    fn test_pipe_both() {
        let input = "echo hello |& grep world";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Word("hello".to_string())
            ]
        );

        assert_eq!(
            command.pipe_to,
            Some(PipeTo {
                pipe_type: RedirType::Both,
                target: Box::new(Command {
                    argv: vec![
                        Arg::Word("grep".to_string()),
                        Arg::Word("world".to_string())
                    ],
                    pipe_to: None,
                    redirect_to: Vec::new(),
                    and_then: None,
                })
            })
        );
    }

    #[test]
    fn test_and_then_if() {
        let input = "echo hello && echo world";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Word("hello".to_string())
            ]
        );

        assert_eq!(
            command.and_then,
            Some(AndThen {
                target: Box::new(Command {
                    argv: vec![
                        Arg::Word("echo".to_string()),
                        Arg::Word("world".to_string())
                    ],
                    pipe_to: None,
                    redirect_to: Vec::new(),
                    and_then: None,
                }),
                conditional: true
            })
        );
    }

    #[test]
    fn test_and_then() {
        let input = "echo hello ; echo world";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Word("hello".to_string())
            ]
        );

        assert_eq!(
            command.and_then,
            Some(AndThen {
                target: Box::new(Command {
                    argv: vec![
                        Arg::Word("echo".to_string()),
                        Arg::Word("world".to_string())
                    ],
                    pipe_to: None,
                    redirect_to: Vec::new(),
                    and_then: None,
                }),
                conditional: false
            })
        );
    }

    #[test]
    fn test_empty_input() {
        let input = "";
        let command = parse_command(input);
        assert_eq!(command, None);
    }

    #[test]
    fn test_unclosed_subshell() {
        let input = "echo $(ls -l";
        let command = parse_command(input);
        assert_eq!(command, None);
    }

    #[test]
    fn test_multiple_redirections() {
        let input = "echo hello > out.txt 2> err.txt";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Word("hello".to_string())
            ]
        );

        assert_eq!(
            command.redirect_to,
            vec![
                FileRedir {
                    redirect_type: RedirType::Stdout,
                    target: PathBuf::from("out.txt")
                },
                FileRedir {
                    redirect_type: RedirType::Stderr,
                    target: PathBuf::from("err.txt")
                }
            ]
        );
    }

    #[test]
    fn test_edge_case_variable_with_special_chars() {
        let input = "echo $HOME_VAR";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Variable("HOME_VAR".to_string())
            ]
        );
    }

    #[test]
    fn test_invalid_variable() {
        let input = "echo $1invalidVar";
        let command = parse_command(input);
        assert_eq!(command, None);
    }

    #[test]
    fn test_empty_word() {
        let input = "echo ";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(command.argv, vec![Arg::Word("echo".to_string())]);
        assert!(command.pipe_to.is_none());
        assert!(command.redirect_to.is_empty());
    }

    #[test]
    fn test_multiple_spaces_between_words() {
        let input = "echo    hello   world";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Word("hello".to_string()),
                Arg::Word("world".to_string())
            ]
        );
    }

    #[test]
    fn test_multiple_redirections_same_type() {
        let input = "echo hello > output.txt > another_output.txt";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Word("hello".to_string())
            ]
        );

        assert_eq!(
            command.redirect_to,
            vec![
                FileRedir {
                    redirect_type: RedirType::Stdout,
                    target: PathBuf::from("output.txt")
                },
                FileRedir {
                    redirect_type: RedirType::Stdout,
                    target: PathBuf::from("another_output.txt")
                }
            ]
        );
    }

    #[test]
    fn test_redirection_with_pipe() {
        let input = "echo hello > output.txt | grep world";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Word("hello".to_string())
            ]
        );

        assert_eq!(
            command.redirect_to,
            vec![FileRedir {
                redirect_type: RedirType::Stdout,
                target: PathBuf::from("output.txt")
            }]
        );

        assert_eq!(
            command.pipe_to,
            Some(PipeTo {
                pipe_type: RedirType::Stdout,
                target: Box::new(Command {
                    argv: vec![
                        Arg::Word("grep".to_string()),
                        Arg::Word("world".to_string())
                    ],
                    pipe_to: None,
                    redirect_to: Vec::new(),
                    and_then: None,
                })
            })
        );
    }

    #[test]
    fn test_redirection_with_and_then() {
        let input = "echo hello > output.txt && echo world";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Word("hello".to_string())
            ]
        );

        assert_eq!(
            command.redirect_to,
            vec![FileRedir {
                redirect_type: RedirType::Stdout,
                target: PathBuf::from("output.txt")
            }]
        );

        assert_eq!(
            command.and_then,
            Some(AndThen {
                target: Box::new(Command {
                    argv: vec![
                        Arg::Word("echo".to_string()),
                        Arg::Word("world".to_string())
                    ],
                    pipe_to: None,
                    redirect_to: Vec::new(),
                    and_then: None,
                }),
                conditional: true
            })
        );
    }

    #[test]
    fn test_variable_in_subshell() {
        let input = "echo $(echo $USER)";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Subshell(Command {
                    argv: vec![
                        Arg::Word("echo".to_string()),
                        Arg::Variable("USER".to_string())
                    ],
                    pipe_to: None,
                    redirect_to: Vec::new(),
                    and_then: None,
                })
            ]
        );
    }

    #[test]
    fn test_subshell_with_redirection() {
        let input = "echo $(ls) > output.txt";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Subshell(Command {
                    argv: vec![Arg::Word("ls".to_string())],
                    pipe_to: None,
                    redirect_to: Vec::new(),
                    and_then: None,
                })
            ]
        );

        assert_eq!(
            command.redirect_to,
            vec![FileRedir {
                redirect_type: RedirType::Stdout,
                target: PathBuf::from("output.txt")
            }]
        );
    }

    #[test]
    fn test_unclosed_quotes() {
        let input = "echo \"hello";
        let command = parse_command(input);
        assert_eq!(command, None);
    }

    #[test]
    fn test_subshell_missing_closing_paren() {
        let input = "echo $(ls";
        let command = parse_command(input);
        assert_eq!(command, None);
    }

    #[test]
    fn test_and_then_with_pipe() {
        let input = "echo hello && echo world | grep test";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Word("hello".to_string())
            ]
        );

        assert_eq!(
            command.and_then,
            Some(AndThen {
                target: Box::new(Command {
                    argv: vec![
                        Arg::Word("echo".to_string()),
                        Arg::Word("world".to_string())
                    ],
                    pipe_to: Some(PipeTo {
                        pipe_type: RedirType::Stdout,
                        target: Box::new(Command {
                            argv: vec![
                                Arg::Word("grep".to_string()),
                                Arg::Word("test".to_string())
                            ],
                            pipe_to: None,
                            redirect_to: Vec::new(),
                            and_then: None,
                        })
                    }),
                    redirect_to: Vec::new(),
                    and_then: None,
                }),
                conditional: true
            })
        );
    }

    #[test]
    fn test_pipe_with_multiple_commands() {
        let input = "echo hello | grep world | sort";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Word("hello".to_string())
            ]
        );

        assert_eq!(
            command.pipe_to,
            Some(PipeTo {
                pipe_type: RedirType::Stdout,
                target: Box::new(Command {
                    argv: vec![
                        Arg::Word("grep".to_string()),
                        Arg::Word("world".to_string())
                    ],
                    pipe_to: Some(PipeTo {
                        pipe_type: RedirType::Stdout,
                        target: Box::new(Command {
                            argv: vec![Arg::Word("sort".to_string())],
                            pipe_to: None,
                            redirect_to: Vec::new(),
                            and_then: None,
                        })
                    }),
                    redirect_to: Vec::new(),
                    and_then: None,
                })
            })
        );
    }

    #[test]
    fn test_empty_subshell() {
        let input = "echo $(echo)";
        let command = parse_command(input).expect("Failed to parse command");

        assert_eq!(
            command.argv,
            vec![
                Arg::Word("echo".to_string()),
                Arg::Subshell(Command {
                    argv: vec![Arg::Word("echo".to_string())],
                    pipe_to: None,
                    redirect_to: Vec::new(),
                    and_then: None,
                })
            ]
        );
    }
}

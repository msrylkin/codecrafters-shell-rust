pub struct ArgsParser {
    input: String,
    state: ArgsState,
    current_handler: CharHandler,
}

impl ArgsParser {
    pub fn new(input: String) -> Self {
        ArgsParser {
            input,
            state: ArgsState::new(),
            current_handler: CharHandler::Unqouted,
        }
    }

    pub fn parse(mut self) -> Vec<String> {
        for c in self.input.chars() {
            self.current_handler = process_char(c, self.current_handler, &mut self.state)
        }

        let args = self.state.finish();

        args
        // let mut args_state = ArgsState::new();
        // let mut char_handler = CharHandler::Unqouted;

        // input.chars().for_each(|c| {
        //     char_handler = process_char(c, char_handler.clone(), &mut args_state);
        // });

        // let args_vec = args_state.finish();
        // let mut args_vec_iter = args_vec.iter();

        // let command = args_vec_iter.next();
        // let command_args: Vec<String> = args_vec_iter.map(|x| x.to_string()).collect();
    }
}

#[derive(Clone)]
pub enum CharHandler {
    SingleQoute,
    DoubleQoute,
    Unqouted,
    Preserve(Box<CharHandler>),
}

impl ArgsState {
    pub fn new() -> Self {
        Self {
            args: vec![],
            res_string: String::new(),
        }
    }

    pub fn finish(mut self) -> Vec<String> {
        self.flush();

        self.args
    }

    fn push_char(&mut self, c: char) {
        self.res_string.push(c);
    }

    fn flush(&mut self) {
        if !self.res_string.is_empty() {
            self.args.push(self.res_string.clone());
            self.res_string = String::new();
        }
    }
}

pub struct ArgsState {
    args: Vec<String>,
    res_string: String,
}

pub fn process_char(
    c: char,
    handler: CharHandler,
    state: &mut ArgsState,
) -> CharHandler {
    match c {
        '\\' => process_backslash(handler, state),
        '\'' => process_single_qoute(handler, state),
        '"' => process_double_qoute(handler, state),
        c if c.is_whitespace() => process_whitespace(c, handler, state),
        c => process_symbol(c, handler, state),
    }
}

fn process_backslash(
    handler: CharHandler,
    state: &mut ArgsState
) -> CharHandler {
    match handler {
        CharHandler::Unqouted | CharHandler::DoubleQoute => CharHandler::Preserve(Box::new(handler)),
        CharHandler::Preserve(parent_handler) => {
            state.push_char('\\');

            *parent_handler
        },
        _ => {
            state.push_char('\\');

            handler
        }
    }
}

fn process_single_qoute(
    handler: CharHandler,
    state: &mut ArgsState,
) -> CharHandler {
    match handler {
        CharHandler::DoubleQoute => {
            state.push_char('\'');

            CharHandler::DoubleQoute
        },
        CharHandler::SingleQoute => CharHandler::Unqouted,
        CharHandler::Unqouted => CharHandler::SingleQoute,
        CharHandler::Preserve(inner_handler) => {
            match *inner_handler {
                CharHandler::DoubleQoute => {
                    state.push_char('\\');
                    state.push_char('\'');
                },
                _ => {
                    state.push_char('\'');
                } 
            };

            *inner_handler
        },
    }
}

fn process_symbol(
    c: char,
    handler: CharHandler,
    state: &mut ArgsState,
) -> CharHandler {
    match handler {
        CharHandler::Preserve(inner_handler) => {
            match *inner_handler {
                CharHandler::DoubleQoute => {
                    state.push_char('\\');
                    state.push_char(c);
                },
                _ => {
                    state.push_char(c);
                } 
            };

            *inner_handler
        },
        _ => {
            state.push_char(c);

            handler
        }
    }
}

fn process_double_qoute(
    handler: CharHandler,
    state: &mut ArgsState,
) -> CharHandler {
    match handler {
        CharHandler::Preserve(inner_handler) => {
            state.push_char('"');

            *inner_handler
        },
        CharHandler::SingleQoute => {
            state.push_char('"');

            handler
        },
        CharHandler::DoubleQoute => {
            CharHandler::Unqouted
        },
        CharHandler::Unqouted => {
            CharHandler::DoubleQoute
        }
    }
}

fn process_whitespace(
    c: char,
    handler: CharHandler,
    state: &mut ArgsState,
) -> CharHandler {
    match handler {
        CharHandler::Preserve(inner_handler) => {
            state.push_char(c);

            *inner_handler
        },
        CharHandler::SingleQoute | CharHandler::DoubleQoute => {
            state.push_char(c);

            handler
        },
        CharHandler::Unqouted => {
            state.flush();

            handler
        }
    }
}
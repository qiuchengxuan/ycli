use std::io::{self, Write as _};
use std::str;

use gethostname::gethostname;
use swagger_utils::path::uri::SegmentIter;
use swagger_utils::path::{Method, Paths};

mod cmd_tree;
mod line_reader;

use self::cmd_tree::{dump_all, dump_cmd_tree, globbing, validate_token, CmdTree};
use self::line_reader::{LineReader, LineType};
use crate::command_tree::node::{Node, COMMIT, CONFIG, SAVE};
use crate::command_tree::ROOT;

#[derive(PartialEq, Copy, Clone)]
enum Mode {
    View,
    Config,
}

impl Into<char> for Mode {
    fn into(self) -> char {
        match self {
            Mode::View => '>',
            Mode::Config => '#',
        }
    }
}

pub struct CLI<'a> {
    hostname: String,
    path: Vec<&'a str>,
    mode: Mode,
    view_tree: CmdTree<'a>,
    config_tree: CmdTree<'a>,
    prompt: String,
}

impl<'a> CLI<'a> {
    pub fn new(paths: &'a Paths) -> CLI<'a> {
        let mut config_tree = CmdTree::new();
        config_tree.insert_root(ROOT);

        let mut view_tree = CmdTree::new();
        view_tree.insert_root(ROOT);

        for (uri, v) in paths.iter() {
            for (method, operation) in v.iter() {
                let tree = match method {
                    Method::Get => &mut view_tree,
                    Method::Put | Method::Patch => &mut config_tree,
                    _ => continue,
                };
                let mut index = tree.get_root_index().unwrap();
                'outer: for segment in SegmentIter::from((uri, operation)) {
                    let node: Node = segment.into();
                    for child_index in tree.children(index) {
                        if tree.get(child_index).unwrap() == &node {
                            index = child_index;
                            continue 'outer;
                        }
                    }
                    index = tree.insert(node, index)
                }
                debug!("Insert uri {} for method {:?}", uri, method);
            }
        }

        let root = view_tree.get_root_index().unwrap();
        view_tree.insert(CONFIG, root);
        debug!("{}", dump_cmd_tree(&view_tree, root));

        let root = config_tree.get_root_index().unwrap();
        config_tree.insert(COMMIT, root);
        config_tree.insert(SAVE, root);
        debug!("{}", dump_cmd_tree(&config_tree, root));

        let hostname = gethostname().into_string().unwrap_or("localhost".into());
        let mode_char: char = Mode::View.into();
        let prompt = format!("{}{} ", hostname, mode_char);
        CLI {
            hostname: hostname,
            path: Vec::new(),
            mode: Mode::View,
            view_tree: view_tree,
            config_tree: config_tree,
            prompt: prompt,
        }
    }

    fn update_prompt(&mut self) {
        let mode_char: char = self.mode.into();
        self.prompt = if self.path.len() > 0 {
            format!("{} {}{} ", self.hostname, self.path.join(" "), mode_char)
        } else {
            format!("{}{} ", self.hostname, mode_char)
        }
    }

    fn complete(&self, line: &str) -> io::Result<()> {
        print!("\r{}{}", self.prompt, line);
        io::stdout().flush()
    }

    fn build_help(&self, line: &str, on_error_padding: usize) -> String {
        let on_error = |token: &str, error: &str| {
            let index = line.find(token).unwrap();
            format!("{}^~~~ {}", " ".repeat(index + on_error_padding), error)
        };

        let tree = match self.mode {
            Mode::View => &self.view_tree,
            Mode::Config => &self.config_tree,
        };
        let mut index = tree.get_root_index().unwrap();
        let tokens: Vec<&str> = line.trim().split_whitespace().collect();
        for token_index in 0..tokens.len() {
            let token = &tokens[token_index];
            let error = match validate_token(tree, token, index) {
                Ok(new_index) => {
                    index = new_index;
                    continue;
                }
                Err(error) => error,
            };
            if token_index == tokens.len() - 1 {
                let globbing = globbing(tree, token, index);
                if globbing.len() > 0 {
                    return globbing;
                }
            }
            return on_error(token, &error);
        }
        dump_all(tree, index)
    }

    fn help(&self, line: &str) -> io::Result<()> {
        println!("");
        println!("{}", self.build_help(line, self.prompt.len()));
        print!("\r{}{}", self.prompt, line);
        io::stdout().flush()
    }

    pub fn run(&mut self) -> io::Result<()> {
        print!("{}", self.prompt);
        io::stdout().flush()?;
        let mut reader = LineReader::new();
        loop {
            let (line, line_type) = reader.read_line()?;
            debug!("line: {:?}", line);

            if line_type == LineType::Complete {
                self.complete(&line)?;
                continue;
            } else if line_type == LineType::Help {
                self.help(&line)?;
                continue;
            }

            match (line.trim(), self.mode) {
                ("config", Mode::View) => {
                    self.mode = Mode::Config;
                    self.update_prompt();
                }
                ("quit", Mode::View) => return Ok(()),
                ("exit", Mode::View) => return Ok(()),
                ("quit", Mode::Config) => {
                    if self.path.len() > 0 {
                        self.path.pop();
                    } else {
                        self.mode = Mode::View
                    }
                    self.update_prompt();
                }
                ("exit", Mode::Config) => {
                    if self.path.len() > 0 {
                        self.path.clear();
                    } else {
                        self.mode = Mode::View
                    }
                    self.update_prompt();
                }
                ("", _) => self.update_prompt(),
                _ => println!("Unrecognized command: {}", line),
            }
            print!("{}", self.prompt);
            io::stdout().flush()?;
            reader.clear();
        }
    }
}

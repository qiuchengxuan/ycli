use std::fmt::Write;

use swagger_utils::validator::Validator;
use vec_tree::{Index, VecTree};

use crate::command_tree::node::Node;

pub type CmdTree<'a> = VecTree<Node<'a>>;

pub fn dump_cmd_tree(tree: &CmdTree, index: Index) -> String {
    let (id, _) = index.into_raw_parts();
    let mut buf: String = format!("{}@{}: ", tree.get(index).unwrap(), id);
    let mut has_child = false;
    for child_index in tree.children(index) {
        let (id, _) = child_index.into_raw_parts();
        let node = tree.get(child_index).unwrap();
        let _ = write!(buf, "{}@{}, ", node, id);
        has_child = true;
    }
    if !has_child {
        return "".into();
    }
    buf.truncate(buf.len() - 2);
    let _ = write!(buf, "\n");
    for child_index in tree.children(index) {
        let _ = write!(buf, "{}", dump_cmd_tree(tree, child_index));
    }
    buf
}

pub fn validate_token(tree: &CmdTree, token: &str, index: Index) -> Result<Index, String> {
    let mut error: String = "None matched".into();
    for child_index in tree.children(index) {
        let result = match tree.get(child_index).unwrap() {
            Node::Fixed(name, _) => {
                if name == &token {
                    return Ok(child_index);
                }
                continue;
            }
            Node::Text(validator, _) => validator.validate(token),
            Node::Number(validator, _) => match token.parse::<i64>() {
                Ok(number) => validator.validate(&number),
                Err(_) => return Err("Not a number".into()),
            },
        };
        match result {
            Some(err) => error = err,
            None => return Ok(child_index),
        }
    }
    Err(error)
}

pub fn globbing(tree: &CmdTree, token: &str, index: Index) -> String {
    let mut string = String::new();
    for child_index in tree.children(index) {
        match tree.get(child_index).unwrap() {
            Node::Fixed(name, description) => {
                if name.starts_with(token) {
                    let _ = write!(string, " {:<10} {}\n", name, description);
                }
            }
            _ => (),
        };
    }
    return string;
}

pub fn dump_all(tree: &CmdTree, index: Index) -> String {
    let mut string = String::new();
    for child_index in tree.children(index) {
        match tree.get(child_index).unwrap() {
            Node::Fixed(name, description) => {
                let _ = write!(string, " {:<10} {}\n", name, description);
            }
            Node::Text(validator, description) => {
                let _ = write!(string, " {:<10} {}\n", validator.to_string(), description);
            }
            Node::Number(validator, description) => {
                let _ = write!(string, " {:<10} {}\n", validator.to_string(), description);
            }
        }
    }
    return string;
}

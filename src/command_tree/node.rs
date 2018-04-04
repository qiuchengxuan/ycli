use std::fmt;

use swagger_utils::path::uri::Segment;
use swagger_utils::validator::integer::IntegerValidator;
use swagger_utils::validator::string::StringValidator;

#[derive(Clone)]
pub enum Node<'a> {
    Fixed(&'a str, &'a str),
    Text(StringValidator, &'a str),
    Number(IntegerValidator, &'a str),
}

impl<'a> fmt::Display for Node<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Node::Fixed(name, _) => write!(f, "{}", name),
            Node::Text(v, _) => write!(f, "{}", v),
            Node::Number(v, _) => write!(f, "{}", v),
        }
    }
}

impl<'a> PartialEq for Node<'a> {
    fn eq(&self, other: &Node) -> bool {
        match (self, other) {
            (Node::Fixed(name, _), Node::Fixed(other_name, _)) => name == other_name,
            (Node::Text(_, _), Node::Text(_, _)) => true,
            (Node::Number(_, _), Node::Number(_, _)) => true,
            _ => false,
        }
    }
}

impl<'a> From<Segment<'a>> for Node<'a> {
    fn from(segment: Segment<'a>) -> Self {
        match segment {
            Segment::Fixed(name) => Node::Fixed(name, "..."),
            Segment::Text(v, description) => Node::Text(v.clone(), description),
            Segment::Number(v, description) => Node::Number(v.clone(), description),
        }
    }
}

pub const CONFIG: Node = Node::Fixed("config", "Enter config mode");
pub const COMMIT: Node = Node::Fixed("commit", "Commit config");
pub const SAVE: Node = Node::Fixed("save", "Save config");

use vec_tree::{Index, VecTree};

pub mod node;

use self::node::Node;

pub const ROOT: Node = Node::Fixed("/", "root");

pub trait CommandQuerier<'a> {
    fn query(&self, index: Index) -> Vec<(String, &'a str)>;
}

impl<'a> CommandQuerier<'a> for VecTree<Node<'a>> {
    fn query(&self, index: Index) -> Vec<(String, &'a str)> {
        let mut results = Vec::<(String, &'a str)>::new();
        for child_index in self.children(index) {
            let tuple = match self.get(child_index).unwrap() {
                Node::Fixed(name, summary) => ((*name).into(), *summary),
                Node::Text(validator, description) => {
                    (format!("<{}>", validator.format), *description)
                }
                Node::Number(validator, description) => {
                    let name = match (validator.minimum, validator.maximum) {
                        (std::i64::MIN, std::i64::MAX) => "<number>".into(),
                        (std::i64::MIN, _) => format!("<-inf~{}>", validator.maximum),
                        (_, std::i64::MAX) => format!("<{}~inf)>", validator.minimum),
                        _ => format!("<{}~{}>", validator.minimum, validator.maximum),
                    };
                    (name, *description)
                }
            };
            results.push(tuple);
        }
        results.push(("quit".into(), "Quit to upper level".into()));
        results.push(("exit".into(), "Exit CLI".into()));
        results
    }
}

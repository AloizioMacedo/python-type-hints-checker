use clap::Parser;

const PARAMETERS_KIND: u16 = 147;
const _TYPED_PARAMETER: u16 = 206;
const _TYPED_DEFAULT_PARAMETER: u16 = 183;
const IDENTIFIER: u16 = 1;
const DEFAULT_PARAMETER: u16 = 182;

/// Checks missing type hints in function definitions for Python files.
#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    path: String,
}

pub fn get_tree_from_file(
    parser: &mut tree_sitter::Parser,
    path: &str,
) -> (tree_sitter::Tree, Vec<u8>) {
    let contents =
        std::fs::read_to_string(path).unwrap_or_else(|_| panic!("File in {path} should exist."));
    let contents_to_return = contents.as_bytes().to_vec();

    (parser.parse(contents, None).unwrap(), contents_to_return)
}

pub fn create_python_parser() -> tree_sitter::Parser {
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(tree_sitter_python::language())
        .expect("Error loading Python grammar");

    parser
}

#[derive(Debug)]
pub struct Position {
    _start: tree_sitter::Point,
    _end: tree_sitter::Point,
    _missing_type: MissingType,
}

#[derive(Debug)]
enum MissingType {
    MissingReturn,
    MissingParameter,
}

pub fn find_missing_types_positions(source_code: &[u8], tree: tree_sitter::Tree) -> Vec<Position> {
    let walk = tree.walk();
    let mut results = Vec::new();

    for node in tree_sitter_traversal::traverse(walk, tree_sitter_traversal::Order::Pre) {
        if node.kind() == "function_definition" {
            let mut cursor = node.walk();

            let mut has_return_type = false;
            for child in node.children(&mut cursor) {
                if child.kind() == "type" {
                    has_return_type = true;
                }

                if child.kind_id() == PARAMETERS_KIND {
                    let mut cursor = child.walk();
                    for inner_child in child.children(&mut cursor) {
                        if matches!(inner_child.kind_id(), IDENTIFIER | DEFAULT_PARAMETER) {
                            if let Ok("self") = inner_child.utf8_text(source_code) {
                                continue;
                            }

                            let start = inner_child.start_position();
                            let end = inner_child.end_position();

                            results.push(Position {
                                _start: start,
                                _end: end,
                                _missing_type: MissingType::MissingParameter,
                            });
                        }
                    }
                }
            }
            if !has_return_type {
                results.push(Position {
                    _start: node.start_position(),
                    _end: node.end_position(),
                    _missing_type: MissingType::MissingReturn,
                });
            }
        }
    }
    results
}

fn main() {
    let args = Args::parse();

    let mut parser = create_python_parser();

    let (tree, source_code) = get_tree_from_file(&mut parser, &args.path);
    let positions = find_missing_types_positions(&source_code, tree);

    println!("{:?}", positions);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tree_from_test_file() {
        let mut parser = create_python_parser();
        let tree = get_tree_from_file(&mut parser, "test_file.py");

        println!("{:?}", tree);
    }

    #[test]
    fn find_args_test() {
        let mut parser = create_python_parser();
        let (tree, source_code) = get_tree_from_file(&mut parser, "test_file.py");
        println!("{:?}", find_missing_types_positions(&source_code, tree));
    }
}

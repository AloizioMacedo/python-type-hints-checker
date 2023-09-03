use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use clap::Parser;
use rayon::prelude::{ParallelBridge, ParallelIterator};

const PARAMETERS_KIND: u16 = 147;
const _TYPED_PARAMETER: u16 = 206;
const _TYPED_DEFAULT_PARAMETER: u16 = 183;
const IDENTIFIER: u16 = 1;
const DEFAULT_PARAMETER: u16 = 182;

/// Checks missing type hints in function definitions for Python files.
#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// File or directory to check
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
    Return(String),
    Parameter(String),
}

pub fn find_missing_types_positions(source_code: &[u8], tree: tree_sitter::Tree) -> Vec<Position> {
    let walk = tree.walk();
    let mut results = Vec::new();

    for node in tree_sitter_traversal::traverse(walk, tree_sitter_traversal::Order::Pre) {
        if node.kind() == "function_definition" {
            let mut cursor = node.walk();

            let mut has_return_type = false;
            for child in node.children(&mut cursor) {
                // println!(
                //     "Kind: {:?}, Text: {:?}, Id: {:?}",
                //     child.kind(),
                //     child.utf8_text(source_code),
                //     child.kind_id()
                // );

                if child.kind() == "type" {
                    has_return_type = true;
                }

                if child.kind_id() == PARAMETERS_KIND {
                    let mut cursor = child.walk();
                    for inner_child in child.children(&mut cursor) {
                        if matches!(inner_child.kind_id(), IDENTIFIER | DEFAULT_PARAMETER) {
                            let utf8_text = inner_child.utf8_text(source_code);

                            if let Ok("self") = utf8_text {
                                continue;
                            }

                            let start = inner_child.start_position();
                            let end = inner_child.end_position();

                            results.push(Position {
                                _start: start,
                                _end: end,
                                _missing_type: MissingType::Parameter(
                                    utf8_text.expect("Parameter should have name").to_string(),
                                ),
                            });
                        }
                    }
                }
            }
            if !has_return_type {
                let identifier = node.child(1).expect("Function should have name.");
                let mut function_name = identifier
                    .utf8_text(source_code)
                    .expect("Function should have name.")
                    .to_string();

                if function_name == "def" {
                    let identifier = node.child(2).expect("Function should have name.");
                    function_name = identifier
                        .utf8_text(source_code)
                        .expect("Function should have name.")
                        .to_string();
                }

                if function_name == "main" {
                    continue;
                }

                results.push(Position {
                    _start: node.start_position(),
                    _end: node.end_position(),
                    _missing_type: MissingType::Return(function_name),
                });
            }
        }
    }
    results
}

fn get_message_from_positions(positions: &[Position]) -> String {
    let mut message = String::new();

    for position in positions {
        match &position._missing_type {
            MissingType::Return(name) => {
                message += &format!(
                    "Function '{name}' in line {} and column {} is missing a return type.\n",
                    position._start.row + 1,
                    position._start.column + 1
                )
            }
            MissingType::Parameter(name) => {
                message += &format!(
                    "Parameter '{name}' in line {} and column {} is missing a type hint.\n",
                    position._start.row + 1,
                    position._start.column + 1
                )
            }
        }
    }

    message
}

fn main() {
    let args = Args::parse();
    let path = args.path;

    let path = PathBuf::from(&path);
    if path.is_dir() {
        let message = Arc::new(Mutex::from(String::new()));

        let walkdir = walkdir::WalkDir::new(path);

        walkdir
            .into_iter()
            .flatten()
            .par_bridge()
            .for_each(|entry| add_to_message_from_file(entry, Arc::clone(&message)));

        print!("{}", message.as_ref().lock().unwrap());
    } else {
        print!("{}", get_message_from_file(path.as_path()));
    }
}

fn add_to_message_from_file(entry: walkdir::DirEntry, message: Arc<Mutex<String>>) {
    if !entry.metadata().expect("Should have metadata.").is_dir()
        && entry
            .file_name()
            .to_str()
            .expect("Should be valid path name.")
            .ends_with(".py")
    {
        let messages_from_file = get_message_from_file(entry.path());
        if messages_from_file.is_empty() {
            return;
        }

        let mut message = message
            .lock()
            .expect("Should be able to get a lock on the message.");

        *message += format!(
            "File: {}\n",
            entry.path().to_str().expect("Should be valid path name.")
        )
        .as_str();

        let messages_from_file = messages_from_file.split('\n');

        for line in messages_from_file {
            *message += &("    ".to_string() + line + "\n")
        }
    }
}

fn get_message_from_file(file: &Path) -> String {
    let mut parser = create_python_parser();

    let (tree, source_code) = get_tree_from_file(
        &mut parser,
        file.to_str().expect("Should be valid path name."),
    );
    let positions = find_missing_types_positions(&source_code, tree);

    get_message_from_positions(&positions)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tree_from_test_file() {
        let mut parser = create_python_parser();
        get_tree_from_file(&mut parser, "test_file.py");
    }

    #[test]
    fn find_args_test() {
        let mut parser = create_python_parser();
        let (tree, source_code) = get_tree_from_file(&mut parser, "test_file.py");
        println!("{:?}", find_missing_types_positions(&source_code, tree));
    }
}

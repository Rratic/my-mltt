use crate::parser::parser::*;

mod definitions;
mod parser;
mod syntax;

fn main() {
    let raw = "y : λ (x : T) . E";
    let mut parser = Parser::new(raw);
    println!("{:?}", parser.parse_program());
}

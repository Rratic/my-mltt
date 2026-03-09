use crate::parser::lexer::{Lexer, TokenType};

mod parser;

fn main() {
    let raw = "a /: T :≡ this_IS_a_ident猫猫0 666 => no -> a=b\t {} () *";
    let mut lexer = Lexer::new(raw);
    loop {
        let token = lexer.next_token();
        if token.class == TokenType::Eof {
            break;
        }
        println!("{:?}", token);
    }
}

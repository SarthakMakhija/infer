use crate::lexer::keywords::Keywords;
use crate::lexer::Lexer;
use crate::parser::stream::ParserStream;
use crate::parser::Parser;

pub(crate) mod ast;
mod lexer;
mod parser;

fn main() {
    let lexer = Lexer::new("var x = 100; var flag = true;", Keywords::new());
    let mut stream = ParserStream::new(lexer);
    let mut parser = Parser::new(&mut stream);

    let program = parser.parse().unwrap();
    println!("{:?}", program);
}

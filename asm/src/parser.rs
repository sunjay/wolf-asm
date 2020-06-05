mod span;
mod source_files;
mod scanner;
mod token;
mod lexer;

pub use span::*;
pub use source_files::*;

use std::fmt::Write;

use crate::ast;
use crate::diagnostics::Diagnostics;

use scanner::Scanner;
use token::{Token, TokenKind};
use lexer::Lexer;

type Input<'a> = &'a [Token];

/// On success, this represents the output and next input position after the output
///
/// On error, this represents what was expected and the actual item found, as well
/// as the input position of the actual item found
pub type IResult<'a, O> = Result<(Input<'a>, O), (Input<'a>, ParseError)>;

#[derive(Debug, Clone)]
pub struct ParseError {
    /// The token kinds expected to be found
    pub expected: Vec<TokenKind>,
    /// The token that was actually found
    pub actual: Token,
}

pub fn parse_program(source: FileSource, diag: &Diagnostics) -> ast::Program {
    let scanner = Scanner::new(source);
    let lexer = Lexer::new(scanner, diag);
    let tokens = collect_tokens(lexer);
    let module = program(&tokens);
    match module {
        Ok((input, module)) => {
            assert!(input.is_empty(), "bug: parser did not consume all input");
            module
        },
        Err((_, err)) => {
            let ParseError {expected, actual} = err;
            //TODO: Order expected tokens with: expected.sort_unstable();

            let mut message = String::new();
            match &expected[..] {
                [] => unreachable!("bug: no parser should produce zero expected tokens"),
                [tk] => write!(message, "expected {}", tk).unwrap(),
                [tk1, tk2] => write!(message, "expected {} or {}", tk1, tk2).unwrap(),
                kinds => {
                    write!(message, "expected one of ").unwrap();
                    for kind in &kinds[..kinds.len()-1] {
                        write!(message, "{}, ", kind).unwrap();
                    }
                    write!(message, "or {}", kinds[kinds.len()-1]).unwrap();
                },
            }
            write!(message, ", found: {}", actual.kind).unwrap();
            diag.span_error(actual.span, message).emit();

            // Error recovery: return an empty program
            ast::Program {
                stmts: Vec::new(),
            }
        },
    }
}

fn collect_tokens(mut lexer: Lexer) -> Vec<Token> {
    let mut tokens = Vec::new();

    loop {
        let token = lexer.next();
        if token.kind == TokenKind::Eof {
            tokens.push(token);
            break;
        }
        tokens.push(token);
    }

    tokens
}

fn program(input: Input) -> IResult<ast::Program> {
    todo!()
}

mod span;
mod source_files;
mod scanner;
mod token;
mod lexer;

pub use span::*;
pub use source_files::*;

use std::fmt;

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
type ParseResult<'a, O> = Result<(Input<'a>, O), (Input<'a>, ParseError<'a>)>;

trait TryParse<'a, I: 'a>: Sized {
    type Output;

    fn and_parse<T, F>(self, f: F) -> ParseResult<'a, (Self::Output, T)>
        where F: FnOnce(I) -> ParseResult<'a, T>;

    fn or_parse<F>(self, f: F) -> ParseResult<'a, Self::Output>
        where F: FnOnce(I) -> ParseResult<'a, Self::Output>;
}

impl<'a, O> TryParse<'a, Input<'a>> for ParseResult<'a, O> {
    type Output = O;

    fn and_parse<T, F>(self, f: F) -> ParseResult<'a, (Self::Output, T)>
        where F: FnOnce(Input<'a>) -> ParseResult<'a, T>
    {
        let (input, value) = self?;
        let (input, value2) = f(input)?;
        Ok((input, (value, value2)))
    }

    fn or_parse<F>(self, f: F) -> ParseResult<'a, Self::Output>
        where F: FnOnce(Input<'a>) -> ParseResult<'a, Self::Output>
    {
        todo!()
    }
}

#[derive(Debug, Clone)]
struct ParseError<'a> {
    /// The token kinds expected to be found
    pub expected: Vec<TokenKind>,
    /// The token that was actually found
    pub actual: &'a Token,
}

impl<'a> fmt::Display for ParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Self {expected, actual} = self;
        //TODO: Order expected tokens with: expected.sort_unstable();

        match &expected[..] {
            [] => unreachable!("bug: no parser should produce zero expected tokens"),
            [tk] => write!(f, "expected {}", tk)?,
            [tk1, tk2] => write!(f, "expected {} or {}", tk1, tk2)?,
            kinds => {
                write!(f, "expected one of ")?;
                for kind in &kinds[..kinds.len()-1] {
                    write!(f, "{}, ", kind)?;
                }
                write!(f, "or {}", kinds[kinds.len()-1])?;
            },
        }

        write!(f, ", found: {}", actual.kind)
    }
}

pub fn parse_program(source: FileSource, diag: &Diagnostics) -> ast::Program {
    let scanner = Scanner::new(source);
    let lexer = Lexer::new(scanner, diag);
    let tokens = collect_tokens(lexer);

    let (input, prog) = program(&tokens, diag);
    assert!(input.is_empty(), "bug: parser did not consume all input");
    prog
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

fn program<'a>(mut input: Input<'a>, diag: &Diagnostics) -> (Input<'a>, ast::Program) {
    let mut stmts = Vec::new();

    while input.get(0).map(|tk| tk.kind != TokenKind::Eof).unwrap_or(false) {
        input = extend_stmts(input, diag, &mut stmts);
    }

    (input, ast::Program {stmts})
}

/// Parses a single `stmt` rule in the grammar
///
/// Due to the structure of the AST, this may append to `stmts` multiple times
fn extend_stmts<'a>(
    mut input: Input<'a>,
    diag: &Diagnostics,
    stmts: &mut Vec<ast::Stmt>,
) -> Input<'a> {
    while let Ok((next_input, label)) = label(input) {
        stmts.push(ast::Stmt::Label(label));
        input = next_input;
    }

    match stmt_body(input).and_parse(|input| tk(input, TokenKind::Newline)) {
        Ok((input, (stmt, _))) => {
            stmts.extend(stmt);
            input
        },

        Err((mut input, err)) => {
            diag.span_error(err.actual.span, err.to_string()).emit();

            // Error recovery is done at a statement level. Read until the end of the line and keep trying
            // to parse the remainder of the file.
            while input.get(0).map(|tk| tk.kind != TokenKind::Newline).unwrap_or(false) {
                let (next_input, _) = advance(input);
                input = next_input;
            }
            // Advance past new line
            let (next_input, _) = advance(input);
            next_input
        },
    }
}

fn label(input: Input) -> ParseResult<ast::Ident> {
    todo!()
}

/// Parses the "body" of a statement (body = without labels and newline)
///
/// Returns `None` if the statement is empty (e.g. just a newline token)
fn stmt_body(input: Input) -> ParseResult<Option<ast::Stmt>> {
    todo!()
}

fn tk(input: Input, kind: TokenKind) -> ParseResult<&Token> {
    let (next_input, token) = advance(input);
    if token.kind == kind {
        // Only proceed with the next input if this succeeds
        Ok((next_input, token))
    } else {
        Err((input, ParseError {
            expected: vec![kind],
            actual: token,
        }))
    }
}

fn advance(input: Input) -> (Input, &Token) {
    (&input[1..], &input[0])
}

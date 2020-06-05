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

    /// Runs a function on the output of a parser (if it hasn't errored), returning the input as is
    fn map_output<T, F>(self, f: F) -> ParseResult<'a, T>
        where F: FnOnce(Self::Output) -> T;

    /// Runs the provided parser only if this result was successful
    ///
    /// The parser is run with the input immediately after this parser.
    fn and_parse<T, F>(self, f: F) -> ParseResult<'a, (Self::Output, T)>
        where F: FnOnce(I) -> ParseResult<'a, T>;

    /// Runs the provided parser only if this one did not succeed
    ///
    /// If both parsers produce an error, the error from the parser that proceeded the furthest is
    /// preferred. If both errors proceeded the same amount, the errors are merged.
    fn or_parse<F>(self, f: F) -> ParseResult<'a, Self::Output>
        where F: FnOnce() -> ParseResult<'a, Self::Output>;
}

impl<'a, O> TryParse<'a, Input<'a>> for ParseResult<'a, O> {
    type Output = O;

    fn map_output<T, F>(self, f: F) -> ParseResult<'a, T>
        where F: FnOnce(Self::Output) -> T
    {
        self.map(|(input, output)| (input, f(output)))
    }

    fn and_parse<T, F>(self, f: F) -> ParseResult<'a, (Self::Output, T)>
        where F: FnOnce(Input<'a>) -> ParseResult<'a, T>
    {
        let (input, value) = self?;
        let (input, value2) = f(input)?;
        Ok((input, (value, value2)))
    }

    fn or_parse<F>(self, f: F) -> ParseResult<'a, Self::Output>
        where F: FnOnce() -> ParseResult<'a, Self::Output>
    {
        use RelativePosition::*;
        match self {
            Ok((input, output)) => Ok((input, output)),
            Err((input1, err1)) => match f() {
                Ok((input, output)) => Ok((input, output)),
                Err((input2, err2)) => match relative_position_to(input2, input1) {
                    Behind => Err((input1, err1)),
                    Same => Err((input1, err1.merge(err2))),
                    Ahead => Err((input2, err2)),
                },
            },
        }
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

impl<'a> ParseError<'a> {
    pub fn merge(self, other: Self) -> Self {
        let Self {mut expected, actual} = self;
        let Self {expected: other_expected, actual: other_actual} = other;

        assert!(actual == other_actual,
            "bug: cannot merge errors where `actual` item is different");

        for item in other_expected {
            if !expected.contains(&item) {
                expected.push(item);
            }
        }

        Self {
            expected,
            actual,
        }
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
    ident(input)
        .and_parse(|input| tk(input, TokenKind::Colon))
        .map_output(|(label, _)| label)
}

/// Parses the "body" of a statement (body = without labels and newline)
///
/// Returns `None` if the statement is empty (e.g. just a newline token)
fn stmt_body(input: Input) -> ParseResult<Option<ast::Stmt>> {
    todo!()
}

fn ident(input: Input) -> ParseResult<ast::Ident> {
    tk(input, TokenKind::Ident).map_output(|token| ast::Ident {
        value: token.unwrap_ident().clone(),
        span: token.span,
    })
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelativePosition {
    /// The left input is behind the right input (left has advanced less than right)
    Behind,
    /// The left input is at the same position as the right input
    Same,
    /// The left input is ahead of the right input (left has advanced more than right)
    Ahead,
}

fn relative_position_to(input: Input, other: Input) -> RelativePosition {
    let self_ptr = input.as_ptr();
    let other_ptr = other.as_ptr();

    use std::cmp::Ordering::*;
    match self_ptr.cmp(&other_ptr) {
        Less => RelativePosition::Behind,
        Equal => RelativePosition::Same,
        Greater => RelativePosition::Ahead,
    }
}

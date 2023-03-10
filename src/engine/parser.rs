use std::{
    error::Error,
    fmt::{self, Display},
    mem::take,
};

#[derive(Debug)]
pub enum AST {
    Char(char),
    Plus(Box<AST>),
    Star(Box<AST>),
    Question(Box<AST>),
    Or(Box<AST>,Box<AST>),
    Seq(Vec<AST>),
    Hat,
    Dollar(Box<AST>),
}

#[derive(Debug)]
pub enum ParserError {
    InvalidEscape(usize, char),
    InvalidRightParan(usize),
    NoPrev(usize),
    NoRightParan,
    Empty,
}

impl Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParserError::InvalidEscape(pos, c) => {
                write!(f, "ParseError: invalid escape: pos = {pos}, char = '{c}'")
            }
            ParserError::InvalidRightParan(pos) => {
                write!(f, "ParseError: invalid right paran: pos = {pos}")
            }
            ParserError::NoPrev(pos) => {
                write!(f, "ParseError: no previous expression: pos = {pos}")
            }
            ParserError::NoRightParan => {
                write!(f, "ParseError: no right parenthesis")
            }
            ParserError::Empty => {
                write!(f, "ParseError: empty expression")
            }
        }
    }
}

impl Error for ParserError {}

fn parser_escape(pos: usize, c: char) -> Result<AST, ParserError> {
    match c {
        '\\' | '(' | ')' | '|' | '+' | '*' | '?' => Ok(AST::Char(c)),
        _ => {
            let err = ParserError::InvalidEscape(pos,c) ;
            Err(err)
        }
    }
}

enum NeedPrev {
    Plus,
    Star,
    Question,
    Dollar,
}

fn parse_need_prev_symbol (
    seq: &mut Vec<AST>,
    ast_type: NeedPrev,
    pos: usize,
) -> Result<(), ParserError> {
    if let Some(prev) = seq.pop() {
        let ast = match ast_type {
            NeedPrev::Plus => AST::Plus(Box::new(prev)),
            NeedPrev::Star => AST::Star(Box::new(prev)),
            NeedPrev::Question => AST::Question(Box::new(prev)),
            NeedPrev::Dollar => AST::Dollar(Box::new(prev)),
        };
        seq.push(ast);
        Ok(())
    } else {
        Err(ParserError::NoPrev(pos))
    }
}

fn fold_or(mut seq_or: Vec<AST>) -> Option<AST> {
    if seq_or.len() > 1 {
        let mut ast = seq_or.pop().unwrap();
        seq_or.reverse();
        for s in seq_or {
            ast = AST::Or(Box::new(s), Box::new(ast));
        }
        Some(ast)
    } else {
        seq_or.pop()
    }
}

pub fn parse(expr: &str) -> Result<AST, ParserError> {
    enum ParseState {
        Char,
        Escape,
    }

    let mut seq = Vec::new();
    let mut seq_or = Vec::new();
    let mut stack = Vec::new();
    let mut state = ParseState::Char;

    for (i, c) in expr.chars().enumerate() {
        match &state {
            ParseState::Char => {
                match c {
                    '+' => parse_need_prev_symbol(&mut seq, NeedPrev::Plus, i)?,
                    '*' => parse_need_prev_symbol(&mut seq, NeedPrev::Star, i)?,
                    '?' => parse_need_prev_symbol(&mut seq, NeedPrev::Question, i)?,
                    '$' => parse_need_prev_symbol(&mut seq, NeedPrev::Dollar, i)?,
                    '(' => {
                        let prev = take(&mut seq) ;
                        let prev_or = take(&mut seq_or);
                        stack.push((prev, prev_or));
                    }
                    ')' => {
                        if let Some((mut prev, prev_or)) = stack.pop() {
                            if !seq.is_empty() {
                                seq_or.push(AST::Seq(seq))
                            }
                            
                            if let Some(ast) = fold_or(seq_or) {
                                prev.push(ast)
                            }

                            seq = prev;
                            seq_or = prev_or;

                        } else {
                            return Err(*Box::new(ParserError::InvalidRightParan(i)))
                        }
                    }
                    '|' => {
                        if seq.is_empty() {
                            return Err(*Box::new(ParserError::NoPrev(i)));
                        } else {
                            let prev = take(&mut seq);
                            seq_or.push(AST::Seq(prev));
                        }
                    }
                    '\\' => state = ParseState::Escape,
                    '^' => seq.push(AST::Hat),
                    _ => seq.push(AST::Char(c)),
                };
            }
            ParseState::Escape => {
                let ast = parser_escape(i,c)?;
                seq.push(ast);
                state = ParseState::Char;
            }
        }
    }

    if !stack.is_empty() {
        return Err(*Box::new(ParserError::NoRightParan));
    }

    if !seq.is_empty() {
        seq_or.push(AST::Seq(seq));
    }

    if let Some(ast) = fold_or(seq_or) {
        Ok(ast)
    } else {
        Err(*Box::new(ParserError::Empty))
    }
}
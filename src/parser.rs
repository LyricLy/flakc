use colored::Colorize;
use crate::ast::{Ast, Inst::{*}};

fn show_span(s: &str, pos: usize) {
    let mut line = 1;
    let mut column = 1;
    let mut cur_line = String::new();
    for (i, c) in s.chars().enumerate() {
        let ending = i >= pos;
        if !ending {
            column += 1;
        }
        if c == '\n' {
            if ending {
                break;
            }
            cur_line.clear();
            line += 1;
            column = 1;
        } else {
            cur_line.push(c);
        }
    }
    eprintln!(" {} :{}:{}", "-->".blue(), line, column);
    eprintln!("{}", "     |".blue());
    eprintln!("{:>4} {} {}", line.to_string().blue(), "|".blue(), cur_line);
    eprintln!("{} {: <3$}{}", "     |".blue(), "", "~".red(), column-1);
}

fn report(s: &str, level: &'static str, msg: &'static str, pos: usize) {
    eprintln!("{}: {}", level.red().bold(), msg);
    show_span(s, pos);
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum DelimType {
    Paren,
    Brace,
    Bracket,
    Angle,
}
use DelimType::{*};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
enum TokenType {
    Open(DelimType),
    Close(DelimType),
    Junk,
}
use TokenType::{*};

#[derive(Debug, Clone, Copy)]
struct Token {
    ty: TokenType,
    pos: usize,
}

fn lex(s: &str) -> Option<Vec<Token>> {
    let mut ts = Vec::new();
    let mut line_is_false_comment = false;
    let mut line_is_comment = false;
    let mut last_was_hash = false;
    let mut block_comment_level: usize = 0;
    for (pos, c) in s.chars().enumerate() {
        if line_is_comment {
            if last_was_hash && c == '{' {
                line_is_comment = false;
                block_comment_level = 1;
            }
            if c == '\n' {
                line_is_comment = false;
            }
            last_was_hash = false;
            continue;
        }
        if block_comment_level > 0 {
            if c == '{' {
                block_comment_level += 1;
            } else if c == '}' {
                block_comment_level -= 1;
            }
            continue;
        }
        match c {
            '(' => ts.push(Token { ty: Open(Paren), pos }),
            ')' => ts.push(Token { ty: Close(Paren), pos }),
            '{' => ts.push(Token { ty: Open(Brace), pos }),
            '}' => ts.push(Token { ty: Close(Brace), pos }),
            '[' => ts.push(Token { ty: Open(Bracket), pos }),
            ']' => ts.push(Token { ty: Close(Bracket), pos }),
            '<' => ts.push(Token { ty: Open(Angle), pos }),
            '>' => ts.push(Token { ty: Close(Angle), pos }),
            '#' => {
                last_was_hash = true;
                line_is_comment = true;
            },
            _ => {
                if c == '\n' {
                    line_is_false_comment = false;
                } else if !c.is_whitespace() {
                    line_is_false_comment = true;
                }
                if !matches!(ts.last(), Some(Token { ty: Junk, pos: _ })) {
                    ts.push(Token { ty: Junk, pos });
                }
                continue;
            },
        }
        if line_is_false_comment {
            line_is_false_comment = false;
            report(s, "warning", "instructions appear after earlier junk characters on the same line", pos);
            eprintln!("{}: this may be an unintentional inclusion of instructions in prose intended to be a comment", "note".bold());
            eprintln!("{}: you can use # for a line comment", "help".green().bold());
            eprintln!("{}: if this is intentional, consider using a #{{block comment}} to enclose the junk characters", "help".green().bold())
        }
    }
    if block_comment_level > 0 {
        report(s, "error", "unclosed block comment somewhere (don't ask where, this is just pointing at the start of the program)", 0);
        return None;
    }
    Some(ts)
}

fn parse_tokens(ts: &mut &[Token], s: &str) -> Option<Ast> {
    let mut a = Vec::new();

    while !ts.is_empty() {
        match ts[0].ty {
            Open(t) => {
                let nilad = if ts.len() >= 3 && ts[1].ty == Junk && ts[2].ty == Close(t) {
                    report(s, "warning", "junk characters enclosed within nilad", ts[1].pos);
                    eprintln!("{}: this harms readability by making it less clear that this is a nilad", "note".bold());
                    *ts = &ts[3..];
                    true
                } else if ts.len() >= 2 && ts[1].ty == Close(t) {
                    *ts = &ts[2..];
                    true
                } else {
                    false
                };
                if nilad {
                    a.push(match t {
                        Paren => One,
                        Brace => Pop,
                        Bracket => Size,
                        Angle => Toggle,
                    });
                } else {
                    let prev_pos = ts[0].pos;
                    *ts = &ts[1..];
                    let ast = parse_tokens(ts, s)?;
                    if ts.is_empty() {
                        report(s, "error", "unclosed delimiter", prev_pos);
                        return None;
                    }
                    let post_pos = ts[0].pos;
                    let (attempt, len) = if ts[0].ty == Junk {
                        (ts[1].ty, 2)
                    } else {
                        (ts[0].ty, 1)
                    };
                    if attempt == Close(t) {
                        *ts = &ts[len..];
                    } else {
                        report(s, "error", "incorrect closing delimiter", post_pos+len-1);
                        return None;
                    }
                    a.push(match t {
                        Paren => Push(ast),
                        Brace => Loop(ast),
                        Bracket => Negate(ast),
                        Angle => Exec(ast),
                    })
                }
            },
            Close(_) => break,
            Junk => *ts = &ts[1..],
        }
    }
    Some(a)
}

pub fn parse(s: &str) -> Option<Ast> {
    let mut token_slice = &*lex(s)?;
    let r = parse_tokens(&mut token_slice, s)?;
    if !token_slice.is_empty() {
        report(s, "error", "unexpected closing delimiter", token_slice[0].pos);
        return None;
    }
    Some(r)
}

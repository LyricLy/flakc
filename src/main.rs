#![feature(let_else)]

mod ast;
mod parser;
mod gen;

use std::fs;

#[derive(argh::FromArgs)]
/// Compile Brain-Flak code.
struct Args {
    /// output C source code instead of a binary
    #[argh(switch, short = 'c')]
    output_c: bool,

    /// file to compile
    #[argh(positional)]
    input: String,

    /// name of output file
    #[argh(option, default = r#"String::from("a.out")"#, short = 'o')]
    output: String,
}

fn main() -> std::io::Result<()> {
    let args: Args = argh::from_env();

    let c_name = if args.output_c { &args.output } else { ".tmp.c" };
    let mut output = fs::File::create(c_name)?;

    let input = fs::read_to_string(args.input)?;
    let Some(tree) = parser::parse(&input) else { return Ok(()) };
    let code = ast::translate(tree);

    gen::compile(&mut output, code)?;

    if !args.output_c {
        std::process::Command::new("gcc")
            .args(["-O2", ".tmp.c", "-o", &args.output])
            .spawn()?
            .wait()?;
    }

    Ok(())
}

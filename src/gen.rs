use crate::ast::{Value, ValuePart, Effects, Effect, StackEffect, Expr};
use std::io::Write;

fn compile_value(b: &mut impl Write, v: Value) -> std::io::Result<()> {
    write!(b, "({}", v.const_val)?;
    for (part, mul) in v.parts {
        write!(b, "+")?;
        match part {
            ValuePart::CurStackElem(n) => write!(b, "s[p-{}]", n+1)?,
            ValuePart::OffStackElem(n) => write!(b, "o[d-{}]", n+1)?,
            ValuePart::CurStackSize => write!(b, "p")?,
            ValuePart::OffStackSize => write!(b, "d")?,
            ValuePart::LoopResult(i) => write!(b, "r{}", i)?,
        };
        if mul != 1 {
            write!(b, "*{}", mul)?;
        }
    }
    write!(b, ")")?;
    Ok(())
}

fn compile_single_stack_effect(b: &mut impl Write, pop: usize, push: Vec<Value>, is_off: bool, effect_index: usize) -> std::io::Result<isize> {
    let (stack, top, cap) = if !is_off {
        ("s", "p", "c")
    } else {
        ("o", "d", "v")
    };
    let offset = push.len() as isize - pop as isize;
    if offset > 0 {
        write!(b, "if({p}+{}>{c}){{{c}*=2;{s}=realloc({s},{c}*sizeof(l));}}", offset, s=stack, p=top, c=cap)?;
    }
    let l = push.len();
    for (i, elem) in push.into_iter().enumerate() {
        write!(b, "l t{}_{}=", i, effect_index)?;
        compile_value(b, elem)?;
        write!(b, ";")?;
    }
    for i in 0..l {
        write!(b, "{s}[{p}+{}]=t{}_{};", i as isize - pop as isize, i, effect_index, s=stack, p=top)?;
    }
    Ok(offset)
}

fn compile_effects(b: &mut impl Write, e: Effects) -> std::io::Result<()> {
    for (i, effect) in e.into_iter().enumerate() {
        match effect {
            Effect::Stack(StackEffect {
                cur_pop,
                cur_push,
                off_pop,
                off_push,
                toggle,
            }) => {
                let p_offset = compile_single_stack_effect(b, cur_pop, cur_push, false, i*2)?;
                let d_offset = compile_single_stack_effect(b, off_pop, off_push, true, i*2+1)?;

                if p_offset != 0 {
                    write!(b, "p+={};", p_offset)?;
                }
                if d_offset != 0 {
                    write!(b, "d+={};", d_offset)?;
                }
                if toggle {
                    write!(b, "{{size_t t=p;p=d;d=t;size_t g=c;c=v;v=g;l*h=s;s=o;o=h;}}")?;
                }
            },
            Effect::Loop(e) => {
                write!(b, "l r{}=0;while(p&&s[p-1]){{", i)?;
                write!(b, "r{}+=", i)?;
                compile_value(b, e.result)?;
                write!(b, ";")?;
                compile_effects(b, e.effects)?;
                write!(b, "}}")?;
            },
        }
    }
    Ok(())
}

pub fn compile(b: &mut impl Write, e: Expr) -> std::io::Result<()> {
    write!(b, "#include<stdlib.h>\n#include<string.h>\n#include<stdio.h>\n\
    typedef long long l;\
    int main(int argc,char**argv){{l*s=malloc(1024*sizeof(l)),*o=malloc(1024*sizeof(l));size_t p=argc-1,d=0;size_t c=1024,v=1024;\
    for(int i=1;i<argc;i++)s[i-1]=atoll(argv[i]);")?;
    let r = compile_effects(b, e.effects)?;
    write!(b, r#"for(size_t i=p-1;i!=-1;i--)printf("%lld\n", s[i]);}}"#)?;
    Ok(r)
}

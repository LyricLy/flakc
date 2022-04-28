use num_bigint::{BigInt, ToBigInt};

#[derive(Debug)]
pub enum Inst {
    One,
    Size,
    Pop,
    Toggle,
    Push(Ast),
    Negate(Ast),
    Loop(Ast),
    Exec(Ast),
}

pub type Ast = Vec<Inst>;


#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ValuePart {
    CurStackElem(usize),
    OffStackElem(usize),
    CurStackSize,
    OffStackSize,
    LoopResult(usize),
}

#[derive(Clone, Debug)]
pub struct Value {
    pub const_val: BigInt,
    pub parts: Vec<(ValuePart, isize)>,
}

impl Value {
    fn zero() -> Value {
        Value { const_val: 0.to_bigint().unwrap(), parts: Vec::new() }
    }

    fn negate(&mut self) {
        self.const_val *= -1;
        for v in self.parts.iter_mut() {
            v.1 *= -1;
        }
    }

    fn add_const(&mut self, other: isize) {
        self.const_val += other;
    }

    fn add_part_n(&mut self, part: ValuePart, n: isize) {
        for i in 0..self.parts.len() {
            if self.parts[i].0 == part {
                self.parts[i].1 += n;
                if self.parts[i].1 == 0 {
                    self.parts.swap_remove(i);
                }
                return;
            }
        }
        self.parts.push((part, n));
    }

    fn add_part(&mut self, part: ValuePart) {
        self.add_part_n(part, 1);
    }

    fn add(&mut self, other: Value) {
        self.const_val += other.const_val;
        for part in other.parts {
            self.add_part_n(part.0, part.1);
        }
    }
}

#[derive(Debug)]
pub struct StackEffect {
    pub cur_pop: usize,
    pub cur_push: Vec<Value>,
    pub off_pop: usize,
    pub off_push: Vec<Value>,
    pub toggle: bool,
}

impl StackEffect {
    fn new() -> StackEffect {
        StackEffect { cur_pop: 0, cur_push: Vec::new(), off_pop: 0, off_push: Vec::new(), toggle: false }
    }

    fn is_empty(&self) -> bool {
        matches!(self, StackEffect { cur_pop: 0, cur_push: a, off_pop: 0, off_push: b, toggle: false } if a.is_empty() && b.is_empty())
    }

    fn pop_push(&mut self) -> (&mut usize, &mut Vec<Value>) {
        if !self.toggle {
            (&mut self.cur_pop, &mut self.cur_push)
        } else {
            (&mut self.off_pop, &mut self.off_push)
        }
    }

    fn stack_elem(&self, t: usize) -> ValuePart {
        if !self.toggle {
            ValuePart::CurStackElem(t)
        } else {
            ValuePart::OffStackElem(t)
        }
    }

    fn stack_size(&self) -> ValuePart {
        if !self.toggle {
            ValuePart::CurStackSize
        } else {
            ValuePart::OffStackSize
        }
    }
}

#[derive(Debug)]
pub enum Effect {
    Stack(StackEffect),
    Loop(Expr),
}

pub type Effects = Vec<Effect>;

#[derive(Debug)]
pub struct Expr {
    pub effects: Effects,
    pub result: Value,
}

fn push_effect(effects: &mut Effects, effect: StackEffect) {
    if !effect.is_empty() {
        effects.push(Effect::Stack(effect));
    }
}

fn translate_with_effects(ast: Ast, effects: &mut Effects, cur_effect: &mut StackEffect) -> Value {
    let mut result = Value::zero();
    for inst in ast {
        match inst {
            Inst::One => result.add_const(1),
            Inst::Size => {
                result.add_part(cur_effect.stack_size());
                let (pop, push) = cur_effect.pop_push();
                result.add_const(push.len() as isize - *pop as isize);
            },
            Inst::Pop => {
                let (pop, push) = cur_effect.pop_push();
                if push.is_empty() {
                    let p = *pop;
                    let part = cur_effect.stack_elem(p);
                    result.add_part(part);
                    let (pop, _) = cur_effect.pop_push();
                    *pop += 1;
                } else {
                    result.add(push.pop().unwrap());
                }
            },
            Inst::Toggle => cur_effect.toggle = !cur_effect.toggle,
            Inst::Push(a) => {
                let r = translate_with_effects(a, effects, cur_effect);
                let (_, push) = cur_effect.pop_push();
                push.push(r.clone());
                result.add(r);
            },
            Inst::Negate(a) => {
                let mut r = translate_with_effects(a, effects, cur_effect);
                r.negate();
                result.add(r);
            },
            Inst::Loop(a) => {
                let c = std::mem::replace(cur_effect, StackEffect::new());
                push_effect(effects, c);
                effects.push(Effect::Loop(translate(a)));
                result.add_part(ValuePart::LoopResult(effects.len()-1));
            },
            Inst::Exec(a) => {
                translate_with_effects(a, effects, cur_effect);
            },
        }
    }
    result
}

pub fn translate(ast: Ast) -> Expr {
    let mut e = Vec::new();
    let mut ce = StackEffect::new();
    let r = translate_with_effects(ast, &mut e, &mut ce);
    push_effect(&mut e, ce);
    Expr { effects: e, result: r }
}

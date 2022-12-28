use super::Instruction;
use crate::helper::safe_add;
use std::{
    collections::VecDeque,
    error::Error,
    fmt::{self,Display},
};

#[derive(Debug)]
pub enum EvalError {
    PCOverFlow,
    SPOverFlow,
    InvalidPC,
    InvalidContext,
}

impl Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CodeGenError: {:?}", self)
    }
}

impl Error for EvalError {}

fn eval_depth(
    inst: &[Instruction],
    line: &[char],
    mut pc: usize,
    mut sp: usize,
) -> Result<bool, EvalError> {
    loop {
        let next = if let Some(i) = inst.get(pc) {
            i
        } else {
            return Err(*Box::new(EvalError::InvalidPC));
        };

        match next {
            Instruction::Char(c) => {
                if let Some(sp_c) = line.get(sp) {
                    if c == &'.' || c == sp_c {
                        safe_add(&mut pc, &1, || Box::new(EvalError::PCOverFlow));
                        safe_add(&mut sp, &1, || Box::new(EvalError::SPOverFlow));
                    } else {
                        return Ok(false);
                    }
                } else {
                    return Ok(false)
                }
            }
            Instruction::Hat => {
                safe_add(&mut pc, &1, || Box::new(EvalError::PCOverFlow)); 
            }
            Instruction::Dollar => {
                if let Some(_sp_c) = line.get(sp) {
                    println!("here0 {}", _sp_c);
                    return  Ok(false);                    
                } else {
                    println!("here");
                    safe_add(&mut pc, &1, || Box::new(EvalError::PCOverFlow));
                }
            }
            Instruction::Match => {
                return Ok(true)
            }
            Instruction::Jump(addr) => {
                pc = *addr;
            }
            Instruction::Split(addr1, addr2) => {
                if eval_depth(inst, line, *addr1, sp)? || eval_depth(inst, line, *addr2, sp)? {
                    return Ok(true)
                } else {
                    return Ok(false)
                }
            }
        }
    }
}

fn eval_width(inst: &[Instruction],line: &[char]) -> Result<bool, EvalError> {
    return Ok(false)
}

pub fn eval(inst: &[Instruction], start: usize, line: &[char], is_depth: bool) -> Result<bool, EvalError> {
    if is_depth {
        eval_depth(inst, line, 0, start)
    } else {
        eval_width(inst, line)
    }
}

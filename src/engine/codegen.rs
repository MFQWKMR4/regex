
use super::{parser::AST, Instruction};
use crate::helper::safe_add;
use std::{
    error::Error,
    fmt::{self, Display}
};

#[derive(Debug)]
pub enum CodeGenError {
    PCOverFlow,
    FailStar,
    FailOr,
    FailQuestion,
}

impl Display for CodeGenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CodeGenError: {:?}", self)
    }
}

impl Error for CodeGenError {}

#[derive(Default, Debug)]
struct Generator {
    pc: usize,
    insts: Vec<Instruction>,
}

impl Generator {

    fn inc_pc(&mut self) -> Result<(), CodeGenError> {
        safe_add(&mut self.pc, &1, || CodeGenError::PCOverFlow)
    }

    fn gen_expr(&mut self, ast: &AST) -> Result<(), CodeGenError> {
        match ast {
            AST::Char(c) => self.gen_char(*c)?,
            AST::Or(e1, e2) => self.gen_or(e1, e2)?,
            AST::Plus(e) => self.gen_plus(e)?,
            AST::Star(e) => self.gen_star(e)?,
            AST::Question(e) => self.gen_question(e)?,
            AST::Seq(v) => self.gen_seq(v)?,
            AST::Hat => self.gen_hat()?,
        }

        Ok(())
    }

    fn gen_plus(&mut self, expr: &AST) -> Result<(), CodeGenError> {
        // l1をさす
        let l1_addr = self.pc;

        self.gen_expr(expr)?;
        // splitを指す
        // l2を指すところまで進める
        self.inc_pc()?;

        let split = Instruction::Split(l1_addr, self.pc);
        self.insts.push(split);
        Ok(())
    }

    fn gen_star(&mut self, expr: &AST) -> Result<(), CodeGenError> {
        let l1_addr = self.pc;
        let split = Instruction::Split(0, 0);
        self.insts.push(split);
        self.inc_pc()?;
        // l2をさす

        let l2_addr = self.pc;
        self.gen_expr(expr)?;
        let jmp = Instruction::Jump(l1_addr);
        self.insts.push(jmp);
        self.inc_pc()?;
        // l3をさす

        let l3_addr = self.pc;
        // l1_addr = split_addrなので
        if let Some(Instruction::Split(l2, l3)) = self.insts.get_mut(l1_addr) {
            *l2 = l2_addr;
            *l3 = l3_addr;
        } else {
            return Err(*Box::new(CodeGenError::FailStar));
        }
        Ok(())
    }

    fn gen_question(&mut self, expr: &AST) -> Result<(), CodeGenError> {
        let split_addr = self.pc;
        let split = Instruction::Split(0, 0);
        self.insts.push(split);
        self.inc_pc()?;
        // l1をさす

        let l1_addr = self.pc;
        self.gen_expr(expr)?;
        // l2をさす

        let l2_addr = self.pc;
        if let Some(Instruction::Split(l1, l2)) = self.insts.get_mut(split_addr) {
            *l1 = l1_addr;
            *l2 = l2_addr;
        } else {
            return Err(*Box::new(CodeGenError::FailQuestion));
        }
        Ok(())
    }

    fn gen_seq(&mut self, exprs: &[AST]) -> Result<(), CodeGenError> {
        for e in exprs {
            self.gen_expr(e)?;
        }
        Ok(())
    }

    fn gen_char(&mut self, c: char) -> Result<(), CodeGenError> {
        let inst = Instruction::Char(c);
        self.insts.push(inst);
        self.inc_pc()?;
        Ok(())
    }

    fn gen_hat(&mut self) -> Result<(), CodeGenError> {
        let inst = Instruction::Hat;
        self.insts.push(inst);
        self.inc_pc()?;
        Ok(())
    }

    fn gen_or(&mut self, e1: &AST, e2: &AST) -> Result<(), CodeGenError> {
        let split_addr = self.pc;
        self.inc_pc()?;
        let split = Instruction::Split(self.pc, 0);
        self.insts.push(split);

        self.gen_expr(e1)?;

        let jmp_addr = self.pc;
        self.insts.push(Instruction::Jump(0));

        self.inc_pc()?;
        if let Some(Instruction::Split(_, l2)) = self.insts.get_mut(split_addr) {
            *l2 = self.pc;
        } else {
            return Err(*Box::new(CodeGenError::FailOr));
        }

        self.gen_expr(e2)?;

        if let Some(Instruction::Jump(l3)) = self.insts.get_mut(jmp_addr) {
            *l3 = self.pc;
        } else {
            return Err(*Box::new(CodeGenError::FailOr));
        }

        Ok(())
    }

    fn gen_code(&mut self, ast: &AST) -> Result<(), CodeGenError> {
        self.gen_expr(ast)?;
        self.inc_pc()?;
        self.insts.push(Instruction::Match);
        Ok(())
    }

}

pub fn get_code(ast: &AST) -> Result<Vec<Instruction>, CodeGenError> {
    let mut generator = Generator::default();
    generator.gen_code(ast)?;
    Ok(generator.insts)
}



use anyhow::Ok;
use itertools::Itertools;
use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    character::complete::{i32, line_ending, one_of, space1},
    multi::separated_list1,
    sequence::tuple,
    Finish, Parser,
};
use std::{
    fs::read_to_string,
    ops::{Index, IndexMut},
};
#[derive(Debug, Clone, Copy)]
enum Instruction {
    Inp(Var),
    Add(Var, VarOrValue),
    Mul(Var, VarOrValue),
    Div(Var, VarOrValue),
    Mod(Var, VarOrValue),
    Eql(Var, VarOrValue),
}

#[derive(Debug, Clone, Copy)]
enum Var {
    W,
    X,
    Y,
    Z,
}
#[derive(Debug, Clone, Copy)]
enum VarOrValue {
    Var(Var),
    Value(i32),
}

type Program = Vec<Instruction>;
#[derive(Debug, Clone)]
struct Alu {
    programs: Vec<Program>,
}
impl Alu {
    fn new(program: &str) -> Self {
        let instructions = parse(program).unwrap();
        let mut programs = vec![];
        for i in instructions.into_iter() {
            match i {
                Instruction::Inp(_) => {
                    programs.push(vec![i]);
                }
                _ => programs.last_mut().unwrap().push(i),
            };
        }
        Self { programs }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct Memory {
    data: [i32; 4],
}
impl From<[i32; 4]> for Memory {
    fn from(data: [i32; 4]) -> Self {
        Self { data }
    }
}
impl Index<Var> for Memory {
    type Output = i32;

    fn index(&self, index: Var) -> &Self::Output {
        match index {
            Var::W => &self.data[0],
            Var::X => &self.data[1],
            Var::Y => &self.data[2],
            Var::Z => &self.data[3],
        }
    }
}
impl IndexMut<Var> for Memory {
    fn index_mut(&mut self, index: Var) -> &mut Self::Output {
        match index {
            Var::W => &mut self.data[0],
            Var::X => &mut self.data[1],
            Var::Y => &mut self.data[2],
            Var::Z => &mut self.data[3],
        }
    }
}
impl Memory {
    fn get(&self, v: VarOrValue) -> i32 {
        match v {
            VarOrValue::Var(var) => self[var],
            VarOrValue::Value(val) => val,
        }
    }
    fn eval<I>(mut self, inst: Instruction, input: &mut I) -> Option<Self>
    where
        I: Iterator<Item = i32>,
    {
        match inst {
            Instruction::Inp(v) => {
                self[v] = input.next()?;
            }
            Instruction::Add(op1, op2) => {
                self[op1] += self.get(op2);
            }
            Instruction::Mul(op1, op2) => {
                self[op1] *= self.get(op2);
            }
            Instruction::Div(op1, op2) => {
                (self.get(op2) != 0).then(|| self[op1] /= self.get(op2))?
            }
            Instruction::Mod(op1, op2) => {
                (self[op1] >= 0 && self.get(op2) > 0).then(|| self[op1] %= self.get(op2))?
            }
            Instruction::Eql(op1, op2) => {
                self[op1] = if self[op1] == self.get(op2) { 1 } else { 0 };
            }
        };
        Some(self)
    }
    fn exec(self, program: &Program, input: &[i32]) -> Option<Self> {
        let mut input_it = input.iter().cloned();
        program
            .iter()
            .copied()
            .fold(Some(self), |acc, instruction| {
                acc?.eval(instruction, &mut input_it)
            })
    }
}
fn parse(input: &str) -> anyhow::Result<Program> {
    let kind = take(3usize);
    let var = |i| {
        one_of("wxyz")
            .map(|c| match c {
                'w' => Var::W,
                'x' => Var::X,
                'y' => Var::Y,
                'z' => Var::Z,
                _ => unreachable!(),
            })
            .parse(i)
    };
    let val = i32.map(VarOrValue::Value);
    let var_or_val = alt((var.map(VarOrValue::Var), val));
    let inp = tuple((tag("inp"), space1, var)).map(|(_, _, var)| Instruction::Inp(var));
    let other =
        tuple((kind, space1, var, space1, var_or_val)).map(|(kind, _, var, _, var_or_val)| {
            match kind {
                "add" => Instruction::Add(var, var_or_val),
                "mul" => Instruction::Mul(var, var_or_val),
                "div" => Instruction::Div(var, var_or_val),
                "mod" => Instruction::Mod(var, var_or_val),
                "eql" => Instruction::Eql(var, var_or_val),
                _ => unimplemented!(),
            }
        });
    let mut parser = separated_list1(line_ending, alt((inp, other)));
    parser
        .parse(input)
        .finish()
        .map(|(_input, parsed)| parsed)
        .map_err(|e: nom::error::VerboseError<&str>| anyhow::anyhow!("parser error: {:?}", e))
}
fn monad(program: &Program, numbers: Vec<(u64, Memory)>) -> Vec<(u64, Memory)> {
    use Var::*;

    let (valid, other): (Vec<_>, Vec<_>) = numbers
        .into_iter()
        .cartesian_product(1..10)
        .filter_map(|((number, mem), input)| {
            let result_mem = mem.exec(program, &[input])?;
            Some((number * 10 + result_mem[W] as u64, result_mem))
        })
        .partition(|(_, mem)| mem[X] == 0);

    if !valid.is_empty() {
        valid
    } else {
        other
    }
}
fn main() -> anyhow::Result<()> {
    let alu = Alu::new(&read_to_string("day-24/input.txt")?);
    let mut model_number = vec![(0, Memory::default())];
    for program in &alu.programs {
        model_number = monad(program, model_number);
    }
    println!("part1 {:?}", model_number.last().unwrap().0);
    println!("part2 {:?}", model_number.first().unwrap().0);
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    const INPUT1: &str = r#"
inp x
mul x -1
    "#;
    const INPUT2: &str = r#"
inp z
inp x
mul z 3
eql z x
    "#;
    const INPUT3: &str = r#"
inp w
add z w
mod z 2
div w 2
add y w
mod y 2
div w 2
add x w
mod x 2
div w 2
mod w 2
    "#;
    const INPUT4: &str = r#"
inp x
div x y
    "#;
    #[test]
    fn exec() -> anyhow::Result<()> {
        let prog1 = parse(INPUT1.trim())?;
        let prog2 = parse(INPUT2.trim())?;
        let prog3 = parse(INPUT3.trim())?;
        let prog4 = parse(INPUT4.trim())?;
        assert_eq!(
            Memory::default().exec(&prog1, &[2]),
            Some(Memory::from([0, -2, 0, 0]))
        );
        assert_eq!(
            Memory::default().exec(&prog2, &[1, 1]),
            Some(Memory::from([0, 1, 0, 0]))
        );
        assert_eq!(
            Memory::default().exec(&prog2, &[1, 3]),
            Some(Memory::from([0, 3, 0, 1]))
        );
        assert_eq!(
            Memory::default().exec(&prog3, &[15]),
            Some(Memory::from([1, 1, 1, 1]))
        );
        assert_eq!(Memory::default().exec(&prog4, &[1]), None);
        Ok(())
    }
}

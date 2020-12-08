use lazy_static::lazy_static;
use regex::Regex;
use std::{path, io, fs, result};
use std::io::BufRead;
use std::collections::HashSet;

lazy_static!{
    static ref INSTRUCTION_REGEX: Regex = Regex::new(r"(?P<opcode>(nop|acc|jmp))\s(?P<arg>[+-]\d+)").expect("illegal instruction regex");
}

#[derive(Copy, Clone)]
enum Instruction {
    Nop(i64),
    Acc(i64),
    Jmp(i64),
}

type Program = Vec<Instruction>;

fn flip_after(current: &[Instruction], n: usize) -> (usize, Program) {
    let mut new_program = current.to_vec();
    for i in n..new_program.len() {
        match new_program.get(i).expect("instruction out of range") {
            Instruction::Nop(n) => {
                new_program[i] = Instruction::Jmp(*n);
                return (i+1, new_program);
            },
            Instruction::Jmp(n) => {
                new_program[i] = Instruction::Nop(*n);
                return (i + 1, new_program);
            },
            _ => (),
        }
    }
    (new_program.len(), new_program)
}

struct Computer {
    program_counter: usize,
    accumulator: i64,
}

impl Computer {
    fn new() -> Computer {
        Computer{
            program_counter: 0,
            accumulator: 0,
        }
    }

    fn execute(&mut self, program: &[Instruction]) -> result::Result<i64, i64> {
        self.program_counter = 0;
        self.accumulator = 0;
        let mut executed: HashSet<usize> = HashSet::new();
        while self.program_counter < program.len() {
            if executed.contains(&self.program_counter) {
                return Err(self.accumulator)
            }
            executed.insert(self.program_counter);
            let instruction = program.get(self.program_counter).expect("instruction out of range");
            match instruction {
                Instruction::Nop(_) => self.program_counter += 1,
                Instruction::Acc(n) => {
                    self.accumulator += *n;
                    self.program_counter += 1;
                }
                Instruction::Jmp(n) => self.program_counter = (self.program_counter as i64 + *n) as usize,
            }

        }
        Ok(self.accumulator)
    }
}

fn read_lines<P: AsRef<path::Path>>(filename: P) -> io::Result<io::Lines<io::BufReader<fs::File>>> {
    let file = fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn read_all_instructions(lines: impl Iterator<Item=io::Result<String>>) -> Program {
    let mut all: Vec<Instruction> = Vec::new();
    for line_res in lines {
        let line = line_res.expect("invalid string");
        all.push(parse_instruction(&line));
    }
    all
}

fn parse_instruction(code: &str) -> Instruction {
    let caps = INSTRUCTION_REGEX.captures(code).expect("invalid code");
    let argument = caps["arg"].parse::<i64>().expect("invalid argument");
    match &caps["opcode"] {
        "nop" => Instruction::Nop(argument),
        "acc" => Instruction::Acc(argument),
        "jmp" => Instruction::Jmp(argument),
        opcode => panic!("invalid opcode >>{}<<", opcode.to_string()),
    }
}

fn main() {
    let lines = read_lines("input.txt").expect("error reading input");
    let program = read_all_instructions(lines);
    let mut computer = Computer::new();
    let result = computer.execute(&program);
    if let Err(accumulator) = result {
        println!("The accumulator before looping is {}", accumulator);
    } else {
        panic!("returned OK");
    }

    let (mut n, mut new_program) = flip_after(&program, 0);
    let mut new_result = computer.execute(&new_program);
    while new_result.is_err() {
        let flipped = flip_after(&program, n);
        n = flipped.0;
        new_program = flipped.1;
        if n >= new_program.len() {
            panic!("cannot find valid program");
        }
        new_result = computer.execute(&new_program);
    }
    let success = new_result.unwrap();
    println!("The final result is {}", success);
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    const TEST_INSTRUCTIONS: &'static str = indoc!{"\
        nop +0
        acc +1
        jmp +4
        acc +3
        jmp -3
        acc -99
        acc +1
        jmp -4
        acc +6"};

    fn to_line_results(data: &'static str) -> impl Iterator<Item = io::Result<String>> {
        data.split('\n').map(|s| Ok(s.to_string()))
    }

    #[test]
    fn can_parse_an_instruction() {
        let nop = parse_instruction("nop +0");
        assert!(matches!(nop, Instruction::Nop));

        let acc = parse_instruction("acc +1");
        assert!(matches!(acc, Instruction::Acc(1)));

        let jmp = parse_instruction("jmp +4");
        assert!(matches!(jmp, Instruction::Jmp(4)));
    }

    #[test]
    fn can_read_all_instructions() {
        let code = to_line_results(TEST_INSTRUCTIONS);
        let instructions = read_all_instructions(code);
        assert_eq!(9, instructions.len());
    }

    #[test]
    fn returns_the_correct_error_on_loop_detection() {
        let program = read_all_instructions(to_line_results(TEST_INSTRUCTIONS));
        let mut computer = Computer::new();
        assert!(matches!(computer.execute(program), Err(5)));
    }
}

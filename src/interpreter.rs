use std::convert::TryInto;

use crate::codegen::INSTRUCTIONS;
use crate::instruction::*;
use crate::rv64_i::MEMORY_SIZE;
use crate::PC;

pub trait InterpreterTrait {
  fn create(initial_registers: Vec<String>) -> Self;
  fn memory_size(&self) -> u32;
  fn set_code(&mut self, code: String);
  fn running(&self) -> bool;
  fn set_running(&mut self, running: bool);
  fn errors(&self) -> &Vec<String>;
  fn warnings(&self) -> &Vec<String>;
  fn registers_repr(&self) -> Vec<(String, String, String)>;
  fn memory_byte_repr(&self, start: usize, len: usize) -> Vec<String>;
  fn memory_ascii_repr(&self, start: usize, len: usize) -> Vec<String>;
  fn toggle_breakpoint(&mut self, line_num: u32);
  fn breakpoints(&self) -> Vec<bool>;
  fn set_frequency(&mut self, frequency: Option<u32>);
  fn next_inst_line_num(&self) -> u32;
}

pub struct Interpreter {
  code: String,
  instructions: Vec<Instruction>,
  registers: [Register; 32],
  memory: [u8; crate::rv64_i::MEMORY_SIZE],
  pc: PC,
  errors: Vec<String>,
  warnings: Vec<String>,
  // Some(0) means single step
  // None means "as fast as possible"
  frequency: Option<u32>,
  running: bool,
}

impl InterpreterTrait for Interpreter {
  fn create(initial_registers: Vec<String>) -> Interpreter {
    let mut interpreter = Interpreter {
      code: "".to_string(),
      instructions: Vec::new(),
      registers: [Register { value: 0 }; 32],
      memory: [0; MEMORY_SIZE],
      pc: PC::new(),
      errors: Vec::new(),
      warnings: Vec::new(),
      frequency: Some(0),
      running: false,
    };

    for (i, r) in initial_registers.iter().enumerate() {
      interpreter.registers[i + 1/* Skip 0 register because it's fixed*/] =
        Register {
          value: parse_int::parse::<u64>(r).expect("Successful conversion"),
        };
    }
    interpreter
  }

  fn memory_size(&self) -> u32 {
    MEMORY_SIZE as u32
  }

  fn set_code(&mut self, code: String) {
    if code.ne(&self.code) {
      self.code = code;
      self.parse();
    }
  }

  fn running(&self) -> bool {
    self.running
  }

  fn set_running(&mut self, running: bool) {
    self.running = running;
  }

  fn errors(&self) -> &Vec<String> {
    &self.errors
  }

  fn warnings(&self) -> &Vec<String> {
    &self.warnings
  }

  fn registers_repr(&self) -> Vec<(String, String, String)> {
    let mut representations: Vec<(String, String, String)> = Vec::new();
    for register in self.registers {
      let repr = (
        format!("{}", register),
        format!("0x{:016X}", register),
        format!("0b{:064b}", register),
      );
      representations.push(repr);
    }
    representations
  }

  fn memory_byte_repr(&self, start: usize, len: usize) -> Vec<String> {
    let mut strings: Vec<String> = Vec::new();
    for b in start..start + len {
      strings.push(format!("{:02x}", self.memory[b as usize]));
    }
    strings
  }

  fn memory_ascii_repr(&self, start: usize, len: usize) -> Vec<String> {
    vec![self.memory[start..(start + len)]
      .iter()
      .map(|num| {
        if *num >= 32 && *num <= 126 {
          (*num as char).to_string()
        } else {
          '.'.to_string()
        }
      })
      .collect()]
  }

  fn toggle_breakpoint(&mut self, line_num: u32) {
    for mut instruction in self.instructions.iter_mut() {
      if instruction.line_num == line_num {
        instruction.breakpoint = !instruction.breakpoint;
        log!("{:?}", instruction);
      }
    }
  }

  fn breakpoints(&self) -> Vec<bool> {
    let max_line_num = self
      .instructions
      .iter()
      .map(|i| i.line_num)
      .max()
      .unwrap_or(0) as usize;

    // Have to create and then set, because of blank lines
    let mut is_break: Vec<bool> = Vec::with_capacity(max_line_num);
    for _i in 0..max_line_num {
      is_break.push(false);
    }
    for instruction in &self.instructions {
      is_break[(instruction.line_num - 1/* 1 indexed */) as usize] =
        instruction.breakpoint;
    }
    is_break
  }

  fn set_frequency(&mut self, frequency: Option<u32>) {
    self.frequency = frequency;
  }

  fn next_inst_line_num(&self) -> u32 {
    let next_inst_num = (self.pc.get().value / 4) as usize;
    if next_inst_num < self.instructions.len() {
      self.instructions[next_inst_num].line_num - 1 /* 1 indexed */
    } else {
      0
    }
  }
}

impl Interpreter {
  pub fn parse(&mut self) {
    for (ln, line) in self.code.lines().enumerate() {
      let line_num: u32 = (ln + 1).try_into().unwrap(); // Source is 1 indexed
      let instruction: &str = line.split("//").nth(0).unwrap().trim();
      if instruction == "" {
        continue;
      }

      let opt_inst: Option<&InstructionSource> =
        INSTRUCTIONS.get(instruction.split_whitespace().nth(0).unwrap());
      if opt_inst.is_none() {
        self.errors.push(format!(
          "Invalid instruction on line {}: {}",
          line_num, instruction
        ));
        continue;
      }
      let inst: &InstructionSource = opt_inst.unwrap();
      let args = inst.parse(instruction);
      if args.is_none() {
        return;
      }
      let impl_func = (inst.implementation)(args.unwrap());
      let actual_instruction = Instruction {
        source: inst,
        line_num: line_num,
        breakpoint: false,
        implementation: impl_func,
      };
      self.instructions.push(actual_instruction);
    }
  }

  pub fn run(&mut self) {
    self.running = true;
    // 4 bytes/instruction
    let max_pc: u64 = self.instructions.len() as u64 * 4;
    while self.pc.get().value < max_pc {
      self.step();
    }
    self.running = false;
  }

  pub fn step(&mut self) {
    log!("{:?}; {}", self.registers, self.pc.get().value);
    self.pc.changed = false;
    let inst = &self.instructions[(self.pc.get().value / 4) as usize];
    log!("{:?}", inst);
    (inst.implementation)(&mut self.registers, &mut self.pc, &mut self.memory);
    if !self.pc.changed {
      self.pc.inc(Register { value: 4 });
    }
    self.registers[0] = Register { value: 0 };
  }

  pub fn stop(&mut self) {
    self.running = false;
  }
  /*
   * Commented out because this is now totally wrong
   * pub fn reset(&mut self) {
   *   self.stop();
   *   *self = Interpreter::new();
   *   self.start();
   * }
   */
}

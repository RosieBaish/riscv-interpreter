use std::convert::TryInto;

use crate::codegen::INSTRUCTIONS;
use crate::instruction::*;

pub struct Interpreter {
  pub code: String,
  pub instructions: Vec<Instruction>,
  pub registers: [Register; 32],
  pub memory: [u8; crate::rv64_i::MEMORY_SIZE],
  pub pc: PC,
  pub errors: Vec<String>,
  pub warnings: Vec<String>,
  // Some(0) means single step
  // None means "as fast as possible"
  pub frequency: Option<u32>,
  pub running: bool,
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

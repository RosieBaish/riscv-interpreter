use std::convert::TryInto;

use wasm_bindgen::prelude::*;

use crate::codegen::INSTRUCTIONS;
use crate::instruction::*;

#[wasm_bindgen]
pub struct Interpreter {
  #[wasm_bindgen(skip)]
  pub code: String,
  #[wasm_bindgen(skip)]
  pub instructions: Vec<Instruction>,
  #[wasm_bindgen(skip)]
  pub registers: [Register; 32],
  #[wasm_bindgen(skip)]
  pub memory: [u8; crate::rv64_i::MEMORY_SIZE],
  #[wasm_bindgen(skip)]
  pub pc: PC,
  #[wasm_bindgen(skip)]
  pub errors: Vec<String>,
  #[wasm_bindgen(skip)]
  pub warnings: Vec<String>,
  // Some(0) means single step
  // None means "as fast as possible"
  #[wasm_bindgen(skip)]
  pub frequency: Option<u32>,
  #[wasm_bindgen(skip)]
  pub running: bool,
  #[wasm_bindgen(skip)]
  pub started: bool,
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
        implementation: impl_func,
      };
      self.instructions.push(actual_instruction);
    }
  }
}

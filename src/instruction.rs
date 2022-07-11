use std::fmt;

use crate::rv64_i::MEMORY_SIZE;

#[allow(dead_code)] // Dead code analysis doesn't check in generated code.
pub enum ImplementationArg {
  Register(usize),
  Imm12([bool; 12]),
  Imm20([bool; 20]),
  Shamt(u64),
}

pub struct PC {
  pub value: u64,
  pub changed: bool,
}

impl PC {
  pub fn new() -> PC {
    PC {
      value: 0,
      changed: false,
    }
  }

  pub fn set(&mut self, val: u64) {
    self.value = val;
    self.changed = true;
  }

  pub fn get(&self) -> u64 {
    self.value
  }

  pub fn inc(&mut self, val: u64) {
    self.value = self.value.wrapping_add(val);
    self.changed = true;
  }
}

#[allow(dead_code)] // Dead code analysis doesn't check in generated code.
pub struct InstructionSource {
  pub mnemonic: &'static str,
  pub expansion: &'static str,
  pub syntax: &'static [&'static str],
  pub description: &'static str,
  pub implementation_str: &'static str,
  pub implementation:
    fn(
      Vec<ImplementationArg>,
    ) -> Box<dyn Fn(&mut [u64; 32], &mut PC, &mut [u8; MEMORY_SIZE])>,
}

impl fmt::Debug for InstructionSource {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("InstructionSource")
      .field("mnemonic", &self.mnemonic)
      .field("syntax", &self.syntax)
      .field("implementation", &self.implementation_str)
      .finish()
  }
}

#[allow(dead_code)] // TODO - connect source and line_num to front end.
pub struct Instruction {
  pub source: &'static InstructionSource,
  pub line_num: u32,
  pub implementation:
    Box<dyn Fn(&mut [u64; 32], &mut PC, &mut [u8; MEMORY_SIZE])>,
}

impl fmt::Debug for Instruction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Instruction")
      .field("source", &self.source)
      .field("line_num", &self.line_num)
      .finish()
  }
}

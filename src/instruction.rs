use std::convert::From;
use std::fmt;
use std::ops;

use crate::rv64_i::MEMORY_SIZE;

#[allow(dead_code)] // Dead code analysis doesn't check in generated code.
pub enum ImplementationArg {
  Register(usize),
  Imm12([bool; 12]),
  Imm20([bool; 20]),
  Shamt(u64),
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Register {
  pub value: u64,
}

impl ops::Add<Register> for Register {
  type Output = Register;

  fn add(self, rhs: Register) -> Register {
    Register {
      value: self.value.wrapping_add(rhs.value),
    }
  }
}

impl ops::Add<u64> for Register {
  type Output = Register;

  fn add(self, rhs: u64) -> Register {
    Register {
      value: self.value.wrapping_add(rhs),
    }
  }
}

impl ops::Add<Register> for u64 {
  type Output = Register;

  fn add(self, rhs: Register) -> Register {
    Register {
      value: self.wrapping_add(rhs.value),
    }
  }
}

impl ops::Sub<Register> for Register {
  type Output = Register;

  fn sub(self, rhs: Register) -> Register {
    Register {
      value: self.value.wrapping_sub(rhs.value),
    }
  }
}

impl ops::Shl<Register> for Register {
  type Output = Register;

  fn shl(self, rhs: Register) -> Register {
    Register {
      value: self.value << rhs.value,
    }
  }
}

impl ops::Shl<u64> for Register {
  type Output = Register;

  fn shl(self, rhs: u64) -> Register {
    Register {
      value: self.value << rhs,
    }
  }
}

impl ops::Shl<Register> for u64 {
  type Output = Register;

  fn shl(self, rhs: Register) -> Register {
    Register {
      value: self << rhs.value,
    }
  }
}

impl ops::Shr<Register> for Register {
  type Output = Register;

  fn shr(self, rhs: Register) -> Register {
    Register {
      value: self.value >> rhs.value,
    }
  }
}

impl ops::Shr<u64> for Register {
  type Output = Register;

  fn shr(self, rhs: u64) -> Register {
    Register {
      value: self.value >> rhs,
    }
  }
}

impl ops::Shr<Register> for u64 {
  type Output = Register;

  fn shr(self, rhs: Register) -> Register {
    Register {
      value: self >> rhs.value,
    }
  }
}

impl ops::BitXor<Register> for Register {
  type Output = Register;

  fn bitxor(self, rhs: Register) -> Register {
    Register {
      value: self.value ^ rhs.value,
    }
  }
}

impl ops::BitXor<u64> for Register {
  type Output = Register;

  fn bitxor(self, rhs: u64) -> Register {
    Register {
      value: self.value ^ rhs,
    }
  }
}

impl ops::BitOr<Register> for Register {
  type Output = Register;

  fn bitor(self, rhs: Register) -> Register {
    Register {
      value: self.value | rhs.value,
    }
  }
}

impl ops::BitOr<u64> for Register {
  type Output = Register;

  fn bitor(self, rhs: u64) -> Register {
    Register {
      value: self.value | rhs,
    }
  }
}

impl ops::BitAnd<Register> for Register {
  type Output = Register;

  fn bitand(self, rhs: Register) -> Register {
    Register {
      value: self.value & rhs.value,
    }
  }
}

impl ops::BitAnd<u64> for Register {
  type Output = Register;

  fn bitand(self, rhs: u64) -> Register {
    Register {
      value: self.value & rhs,
    }
  }
}

impl From<bool> for Register {
  fn from(item: bool) -> Self {
    Register { value: item as u64 }
  }
}

impl std::fmt::Display for Register {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Display::fmt(&self.value, f)
  }
}

impl std::fmt::UpperHex for Register {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::UpperHex::fmt(&self.value, f)
  }
}

impl std::fmt::Binary for Register {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    fmt::Binary::fmt(&self.value, f)
  }
}

pub struct PC {
  value: u64,
  pub changed: bool,
}

impl PC {
  pub fn new() -> PC {
    PC {
      value: 0,
      changed: false,
    }
  }

  pub fn set(&mut self, val: Register) {
    self.value = val.value;
    self.changed = true;
  }

  pub fn get(&self) -> Register {
    Register { value: self.value }
  }

  pub fn inc(&mut self, val: Register) {
    self.value = self.value.wrapping_add(val.value);
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
  pub implementation: fn(
    Vec<ImplementationArg>,
  ) -> Box<
    dyn Fn(&mut [Register; 32], &mut PC, &mut [u8; MEMORY_SIZE]),
  >,
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
    Box<dyn Fn(&mut [Register; 32], &mut PC, &mut [u8; MEMORY_SIZE])>,
}

impl fmt::Debug for Instruction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Instruction")
      .field("source", &self.source)
      .field("line_num", &self.line_num)
      .finish()
  }
}

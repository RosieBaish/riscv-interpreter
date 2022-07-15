use std::convert::{From, TryInto};
use std::fmt;
use std::ops;

use crate::build_common::*;
use crate::interface;
use crate::log;
use crate::registers;
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

impl<'a> InstructionSource {
  pub fn format_error(&self, tokens: Vec<String>) {
    interface::alert(format!("Invalid instruction format. Instruction \"{}\" should have format \"{}\" but instead had \"{}\"", self.mnemonic, self.syntax.join(" "), tokens.join(" ")).as_str());
  }

  pub fn parse(&self, code: &str) -> Option<Vec<ImplementationArg>> {
    let tokens: Vec<String> = tokenise(code);
    if tokens.len() != self.syntax.len() {
      log!("Wrong number of tokens");
      self.format_error(tokens);
      return None;
    }
    let mut arguments: Vec<ImplementationArg> = Vec::new();
    for (actual, expected) in core::iter::zip(tokens.iter(), self.syntax.iter())
    {
      if expected.eq(&"rd") || expected.eq(&"rs1") || expected.eq(&"rs2") {
        let reg_num = registers::NAMES.get(actual);
        if reg_num.is_none() {
          log!("Failed to get reg num");
          self.format_error(tokens);
          return None;
        }
        arguments.push(ImplementationArg::Register(
          (*reg_num.unwrap()).try_into().unwrap(),
        ));
      } else if expected.eq(&"imm") || expected.eq(&"offset") {
        let val = parse_imm::<12>(actual.to_string());
        if val.is_none() {
          log!("Failed to get val");
          self.format_error(tokens);
          return None;
        }
        arguments.push(ImplementationArg::Imm12(val.unwrap()));
      } else if expected.eq(&"imm20") {
        let val = parse_imm::<20>(actual.to_string());
        if val.is_none() {
          log!("Failed to get val");
          self.format_error(tokens);
          return None;
        }
        arguments.push(ImplementationArg::Imm20(val.unwrap()));
      } else if expected.eq(&"shamt") {
        arguments.push(ImplementationArg::Shamt(
          parse_int::parse::<u64>(actual).ok().unwrap(),
        ));
      } else if actual == expected {
        // If it matches, we're good
      } else {
        log!("Nothing matched");
        self.format_error(tokens);
        return None;
      }
    }
    return Some(arguments);
  }
}

#[allow(dead_code)] // TODO - connect source and line_num to front end.
pub struct Instruction {
  pub source: &'static InstructionSource,
  pub line_num: u32, // 1 indexed
  pub breakpoint: bool,
  pub implementation:
    Box<dyn Fn(&mut [Register; 32], &mut PC, &mut [u8; MEMORY_SIZE])>,
}

impl fmt::Debug for Instruction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Instruction")
      .field("source", &self.source)
      .field("line_num", &self.line_num)
      .field("breakpoint", &self.breakpoint)
      .finish()
  }
}

fn parse_imm<const ARRLEN: usize>(input: String) -> Option<[bool; ARRLEN]> {
  let mut bitvec: [bool; ARRLEN] = [false; ARRLEN];
  let val: i32 = parse_int::parse::<i32>(&input).ok()?;
  let upper_bound: i32 = 1 << ARRLEN;
  let lower_bound: i32 = -1 * (1 << ARRLEN - 1);
  if val < lower_bound || val >= upper_bound {
    log!(
      "parse_imm<{}>({}): Require {} < {} <= {}",
      ARRLEN,
      input,
      lower_bound,
      val,
      upper_bound
    );
    return None;
  }
  for i in 0..ARRLEN {
    bitvec[i] = (val >> i) & 1 == 1;
  }
  return Some(bitvec);
}

#[cfg(test)]
mod tests {
  use super::*;

  /*
   * Note: the imm arrays are LSB first, MSB last (i.e. the opposite of the
   * way that we write a binary number)
   * To make it easier to write the tests, this function reverses the array
   * you give it, so that you write the int array in the normal order
   */
  fn imm<const ARRLEN: usize>(input: [i32; ARRLEN]) -> [bool; ARRLEN] {
    let mut output = [false; ARRLEN];
    for i in 0..ARRLEN {
      assert!(input[i] == 0 || input[i] == 1);
      output[(ARRLEN - i) - 1] = input[i] == 1;
    }
    output
  }

  #[test]
  fn parse_immediate_decimal_positive_in_range() {
    assert_eq!(parse_imm(String::from("10")).unwrap(), imm([1, 0, 1, 0]));
  }

  #[test]
  fn parse_immediate_decimal_negative_in_range() {
    assert_eq!(parse_imm(String::from("-5")).unwrap(), imm([1, 0, 1, 1]));
  }

  #[test]
  fn parse_immediate_decimal_positive_out_of_range() {
    assert_eq!(parse_imm::<4>(String::from("16")), None);
  }

  #[test]
  fn parse_immediate_decimal_negative_out_of_range() {
    assert_eq!(parse_imm::<4>(String::from("-9")), None);
  }

  #[test]
  fn parse_immediate_hex_in_range() {
    assert_eq!(parse_imm(String::from("0xa")).unwrap(), imm([1, 0, 1, 0]));
  }

  #[test]
  fn parse_immediate_hex_out_of_range() {
    assert_eq!(parse_imm::<4>(String::from("0x10")), None);
  }

  #[test]
  fn parse_immediate_binary_in_range() {
    assert_eq!(
      parse_imm(String::from("0b1010")).unwrap(),
      imm([1, 0, 1, 0])
    );
  }

  #[test]
  fn parse_immediate_binary_out_of_range() {
    assert_eq!(parse_imm::<4>(String::from("0b10000")), None);
  }

  #[test]
  fn parse_immediate_invalid_input_1() {
    assert_eq!(parse_imm::<4>(String::from("Not a number")), None);
  }

  #[test]
  fn parse_immediate_invalid_input_2() {
    assert_eq!(parse_imm::<4>(String::from("10 Not a number")), None);
  }
}

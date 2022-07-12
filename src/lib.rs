use std::convert::TryInto;

mod build_common;
use build_common::*;
mod codegen;
mod instruction;
use instruction::ImplementationArg::*;
use instruction::*;
#[macro_use]
mod interface;
mod interpreter;
mod registers;
mod rv64_i;
mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

fn parse_imm<const ARRLEN: usize>(input: String) -> Option<[bool; ARRLEN]> {
  let mut bitvec: [bool; ARRLEN] = [false; ARRLEN];
  let val: Option<i32> = parse_int::parse::<i32>(&input).ok();
  if val.is_none() {
    return None;
  }
  for i in 0..ARRLEN {
    bitvec[i] = (val.unwrap() >> i) & 1 == 1;
  }
  return Some(bitvec);
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
        arguments.push(Register((*reg_num.unwrap()).try_into().unwrap()));
      } else if expected.eq(&"imm") || expected.eq(&"offset") {
        let val = parse_imm::<12>(actual.to_string());
        if val.is_none() {
          log!("Failed to get val");
          self.format_error(tokens);
          return None;
        }
        arguments.push(Imm12(val.unwrap()));
      } else if expected.eq(&"imm20") {
        let val = parse_imm::<20>(actual.to_string());
        if val.is_none() {
          log!("Failed to get val");
          self.format_error(tokens);
          return None;
        }
        arguments.push(Imm20(val.unwrap()));
      } else if expected.eq(&"shamt") {
        arguments.push(Shamt(parse_int::parse::<u64>(actual).ok().unwrap()));
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

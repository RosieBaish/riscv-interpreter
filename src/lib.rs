use std::cell::RefCell;
use std::convert::TryInto;
use std::fmt;

mod build_common;
use build_common::*;
mod registers;
mod utils;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[allow(dead_code)] // Dead code analysis doesn't check in generated code.
enum ImplementationArg {
  Register(usize),
  Imm12([bool; 12]),
  Imm20([bool; 20]),
  Shamt(u64),
}
use ImplementationArg::*;

#[allow(dead_code)] // Dead code analysis doesn't check in generated code.
struct InstructionSource {
  mnemonic: &'static str,
  expansion: &'static str,
  syntax: &'static [&'static str],
  description: &'static str,
  implementation_str: &'static str,
  implementation:
    fn(Vec<ImplementationArg>) -> Box<dyn Fn(&mut [u64; 32], &mut u64)>,
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
struct Instruction {
  source: &'static InstructionSource,
  line_num: u32,
  implementation: Box<dyn Fn(&mut [u64; 32], &mut u64)>,
}

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

fn sext<const ARRLEN: usize>(input: [bool; ARRLEN]) -> u64 {
  let mut total: u64 = 0;
  for i in 0..ARRLEN {
    total |= (input[i] as u64) << i;
  }
  for i in ARRLEN..64 {
    total |= (input[ARRLEN - 1] as u64) << i;
  }
  total
}

fn sext_n(input: u64, current_len: u32) -> u64 {
  let mut total: u64 = 0;
  for i in 0..current_len {
    total |= input & (1 << i);
  }
  let highbit = input & (1 << current_len) >> current_len;
  for i in current_len..64 {
    total |= highbit << i;
  }
  total
}

fn signed_lt(left: u64, right: u64) -> bool {
  let s_left = i64::from_ne_bytes(left.to_ne_bytes());
  let s_right = i64::from_ne_bytes(right.to_ne_bytes());
  return s_left < s_right;
}

fn arith_r_shift(val: u64, offset: u64) -> u64 {
  let s_val = i64::from_ne_bytes(val.to_ne_bytes());
  // The >> operator is an arithmetic shift if the LHS is signed
  // And a logical shift if not, which is what we want.
  let shifted_s_val = s_val >> offset;
  u64::from_ne_bytes(shifted_s_val.to_ne_bytes())
}

const MEMORY_SIZE: usize = 1024;
thread_local! {static MEMORY: RefCell<[u8; MEMORY_SIZE]> = RefCell::new([0; MEMORY_SIZE]);}

fn mem_read(address: u64, length: u32) -> u64 {
  assert!(length == 8 || length == 16 || length == 32 || length == 64);
  assert!(address as usize + ((length / 8) as usize) < MEMORY_SIZE);

  let mut val: u64 = 0;

  MEMORY.with(|m| {
    let mem: [u8; MEMORY_SIZE] = *m.borrow();
    for i in 0..(length / 8) {
      val += (mem[address as usize] as u64) << (i * 8);
    }
  });
  val
}

fn mem_read_sext(address: u64, length: u32) -> u64 {
  sext_n(mem_read(address, length), length)
}

fn mem_write(address: u64, length: u32, val: u64) {
  assert!(length == 8 || length == 16 || length == 32 || length == 64);
  assert!(address as usize + ((length / 8) as usize) < MEMORY_SIZE);

  MEMORY.with(|m| {
    let mut mem: [u8; MEMORY_SIZE] = *m.borrow_mut();
    for i in 0..(length / 8) {
      mem[address as usize] = ((val >> (i * 8)) & 0xFF) as u8;
    }
  });
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
  fn alert(s: &str);
}

fn parse_imm<const ARRLEN: usize>(input: String) -> Option<[bool; ARRLEN]> {
  let mut bitvec: [bool; ARRLEN] = [false; ARRLEN];
  let val: Option<i32> = input.parse::<i32>().ok();
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
    alert(format!("Invalid instruction format. Instruction \"{}\" should have format \"{}\" but instead had \"{}\"", self.mnemonic, self.syntax.join(" "), tokens.join(" ")).as_str());
  }

  pub fn parse(&self, code: &str) -> Option<Vec<ImplementationArg>> {
    let tokens: Vec<String> = tokenise(code);
    if tokens.len() != self.syntax.len() {
      self.format_error(tokens);
      return None;
    }
    let mut arguments: Vec<ImplementationArg> = Vec::new();
    for (actual, expected) in core::iter::zip(tokens.iter(), self.syntax.iter())
    {
      if expected.eq(&"rd") || expected.eq(&"rs1") || expected.eq(&"rs2") {
        let reg_num = registers::NAMES.get(actual);
        if reg_num.is_none() {
          self.format_error(tokens);
          return None;
        }
        arguments.push(Register((*reg_num.unwrap()).try_into().unwrap()));
      } else if expected.eq(&"imm") {
        let val = parse_imm::<12>(actual.to_string());
        if val.is_none() {
          self.format_error(tokens);
          return None;
        }
        arguments.push(Imm12(val.unwrap()));
      } else if expected.eq(&"imm20") {
        let val = parse_imm::<20>(actual.to_string());
        if val.is_none() {
          self.format_error(tokens);
          return None;
        }
        arguments.push(Imm20(val.unwrap()));
      } else if actual == expected {
        // If it matches, we're good
      } else {
        self.format_error(tokens);
        return None;
      }
    }
    return Some(arguments);
  }
}

#[wasm_bindgen]
pub struct Interpreter {
  code: String,
  instructions: Vec<Instruction>,
}

#[wasm_bindgen]
impl Interpreter {
  pub fn new() -> Interpreter {
    //    alert("Init");
    Interpreter {
      code: "".to_string(),
      instructions: Vec::new(),
    }
  }

  fn get_code(&self) -> String {
    let window = web_sys::window().expect("global window does not exists");
    let document = window.document().expect("expecting a document on window");
    let code_text = document
      .get_element_by_id("code")
      .unwrap() // De-optionify
      .dyn_into::<web_sys::HtmlTextAreaElement>() // Cast
      .unwrap(); // Unwrap the cast result
    return code_text.value();
  }

  fn update_if_necessary(&mut self) {
    let c: String = self.get_code();
    if c.ne(&self.code) {
      self.code = c;
      self.parse();
    }
  }

  #[wasm_bindgen]
  pub fn run_button(&mut self) {
    let mut registers: [u64; 32] = [0; 32];
    let mut pc: u64 = 0;
    self.update_if_necessary();
    for inst in &self.instructions {
      (inst.implementation)(&mut registers, &mut pc);
    }
    alert(format!("{:?}", registers).as_str());
  }

  #[wasm_bindgen]
  pub fn step_button(&mut self) {
    self.update_if_necessary();
    //    parse("hello".to_string());
    //    alert("Step");
  }

  #[wasm_bindgen]
  pub fn reset_button(&self) {
    //    parse("hello".to_string());
    //    alert("Reset");
  }

  #[wasm_bindgen]
  pub fn stop_button(&self) {
    //    alert("Stop");
  }

  fn parse(&mut self) {
    for (ln, line) in self.code.lines().enumerate() {
      let line_num: u32 = (ln + 1).try_into().unwrap(); // Source is 1 indexed
      let instruction: &str = line.split("//").nth(0).unwrap().trim();
      if instruction == "" {
        continue;
      }

      let opt_inst: Option<&InstructionSource> =
        INSTRUCTIONS.get(instruction.split_whitespace().nth(0).unwrap());
      if opt_inst.is_none() {
        alert(
          format!("Invalid instruction on line {}: {}", line_num, instruction)
            .as_str(),
        );
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

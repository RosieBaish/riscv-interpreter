use crate::codegen::INSTRUCTIONS;

mod rv64_i;
use rv64_i::RiscV64_i;

pub trait InterpreterTrait {
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
  fn run(&mut self);
  fn step(&mut self);
  fn stop(&mut self);
}

#[allow(non_camel_case_types)]
enum Architecture {
  RiscV64_i(RiscV64_i),
}

pub struct Interpreter {
  architecture: Architecture,
}

include!(concat!(env!("OUT_DIR"), "/interpreter_dispatch.rs"));

impl Interpreter {
  #[allow(non_snake_case)]
  pub fn create_RiscV64_i(initial_registers: Vec<String>) -> Self {
    Interpreter {
      architecture: Architecture::RiscV64_i(RiscV64_i::create(
        initial_registers,
      )),
    }
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

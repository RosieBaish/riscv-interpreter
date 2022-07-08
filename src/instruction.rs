use std::fmt;

#[allow(dead_code)] // Dead code analysis doesn't check in generated code.
pub enum ImplementationArg {
  Register(usize),
  Imm12([bool; 12]),
  Imm20([bool; 20]),
  Shamt(u64),
}

#[allow(dead_code)] // Dead code analysis doesn't check in generated code.
pub struct InstructionSource {
  pub mnemonic: &'static str,
  pub expansion: &'static str,
  pub syntax: &'static [&'static str],
  pub description: &'static str,
  pub implementation_str: &'static str,
  pub implementation:
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
pub struct Instruction {
  pub source: &'static InstructionSource,
  pub line_num: u32,
  pub implementation: Box<dyn Fn(&mut [u64; 32], &mut u64)>,
}

use serde::Serialize;

#[path = "../src/build_common.rs"]
mod build_common;
use build_common::*;

#[derive(Debug, Serialize)]
pub struct Instruction {
  pub mnemonic: String,
  pub expansion: String,
  pub syntax: Vec<String>,
  pub description: String,
  pub implementation: String,
}

impl Instruction {
  fn reg_or_imm(arg: &String) -> &'static str {
    if arg.eq(&"imm".to_string()) || arg.eq(&"offset".to_string()) {
      "Imm12"
    } else if arg.eq(&"imm20".to_string()) {
      "Imm20"
    } else if arg.eq(&"shamt".to_string()) {
      "Shamt"
    } else {
      "Register"
    }
  }

  fn get_args(&self) -> Vec<&String> {
    self
      .syntax
      .iter()
      .filter(|x| x.chars().all(char::is_alphanumeric))
      .skip(1) // First one is the mnemonic, not an arg
      .collect()
  }

  fn escaped_mnemonic(&self) -> String {
    self.mnemonic.replace(|c: char| !c.is_alphanumeric(), "_")
  }

  pub fn create_implementation_source(&self) -> String {
    let mut impl_src = format!(
      "#[allow(unused_variables)]\n\
       fn {} (args: Vec<ImplementationArg>) -> MachineInstruction {{\n\
       if let [",
      self.escaped_mnemonic(),
    );
    for arg in self.get_args() {
      impl_src.push_str(
        format!(
          "ImplementationArg::{}({}), ",
          Instruction::reg_or_imm(arg),
          arg
        )
        .as_str(),
      );
    }
    let mut log_string = String::from("\"");
    for arg in self.get_args() {
      log_string.push_str(format!("{}: {{:?}} ", arg).as_str());
    }
    log_string.push_str("\", ");
    for arg in self.get_args() {
      log_string.push_str(format!("{}, ", arg).as_str());
    }
    impl_src.push_str(
      format!(
        "] = args[..] {{\n\
         \tBox::new(move |x: &mut [Register; 32], pc: &mut PC, mem: &mut [u8; crate::rv64_i::MEMORY_SIZE]| {{\n\
         crate::log!({});
         \t\t{}\n\
         \t}})\n\
         }} else {{\n\
         \tunreachable!(\"Wrong arg type\") }}\n\
         }}\n\n",
        log_string,
        self.implementation
      )
      .as_str(),
    );
    impl_src
  }

  pub fn as_source(&self) -> String {
    let syntax_str = "&[\"".to_string() + &self.syntax.join("\", \"") + "\"]";
    format!(
      "InstructionSource {{\n\
             mnemonic: \"{}\",\n\
             expansion: \"{}\",\n\
             syntax: {},\n\
             description: r#\"{}\"#,\n\
             implementation_str: \"{}\",\n\
             implementation: {}\n\
             }}",
      self.mnemonic,
      self.expansion,
      syntax_str,
      self.description,
      self.implementation,
      self.escaped_mnemonic()
    )
  }

  pub fn parse(cells: [&str; 5]) -> Option<Instruction> {
    if let [m, e, s, d, i] = &cells[..] {
      let i_str = i.trim().replace("BITWISE_OR", "|");
      Some(Instruction {
        mnemonic: m.trim().to_string(),
        expansion: e.trim().to_string(),
        syntax: tokenise(s),
        description: d.trim().to_string(),
        implementation: i_str,
      })
    } else {
      println!("{:?}", cells);
      None
    }
  }
}

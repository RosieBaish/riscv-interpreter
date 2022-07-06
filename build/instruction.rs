#[path = "../src/build_common.rs"]
mod build_common;
use build_common::*;

#[derive(Debug)]
pub struct Instruction {
  pub mnemonic: String,
  pub expansion: String,
  pub syntax: Vec<String>,
  pub description: String,
  pub implementation: String,
}

const NUM_COLUMNS: usize = 5;

impl Instruction {
  fn reg_or_imm(arg: &String) -> &'static str {
    if arg.eq(&"imm".to_string()) || arg.eq(&"offset".to_string()) {
      return "Imm12";
    } else if arg.eq(&"imm20".to_string()) {
      return "Imm20";
    } else if arg.eq(&"shamt".to_string()) {
      return "Shamt";
    } else {
      return "Register";
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
    return self.mnemonic.replace(|c: char| !c.is_alphanumeric(), "_");
  }

  pub fn create_implementation_source(&self) -> String {
    let mut impl_src = String::from(format!(
      "#[allow(unused_variables)]\n\
       fn {} (args: Vec<ImplementationArg>) -> Box<dyn Fn(&mut [u64; 32], &mut u64)> {{\n\
       if let [",
      self.escaped_mnemonic(),
    ));
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
    impl_src.push_str(
      format!(
        "] = args[..] {{\n\
         \tBox::new(move |x: &mut [u64; 32], pc: &mut u64| {{\n\
         \t\t{}\n\
         \t}})\n\
         }} else {{\n\
         \tunreachable!(\"Wrong arg type\") }}\n\
         }}\n\n",
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

  pub fn parse(line: &str) -> Option<Instruction> {
    let mut cols: Vec<&str> = line.split("|").collect();
    cols = cols[1..].to_vec();
    cols.pop();
    assert_eq!(cols.len(), NUM_COLUMNS);
    if let [m, e, s, d, i] = &cols[..] {
      let i_str = i.trim().replace("BITWISE_OR", "|");
      Some(Instruction {
        mnemonic: m.trim().to_string(),
        expansion: e.trim().to_string(),
        syntax: tokenise(s),
        description: d.trim().to_string(),
        implementation: i_str,
      })
    } else {
      println!("{}", line);
      None
    }
  }
}

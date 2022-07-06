use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;

#[path = "../src/build_common.rs"]
mod build_common;
use build_common::*;
mod rustfmt;

#[derive(Debug)]
struct Instruction {
  mnemonic: String,
  expansion: String,
  syntax: Vec<String>,
  description: String,
  implementation: String,
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

  fn create_implementation_source(&self) -> String {
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

  fn as_source(&self) -> String {
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
}

fn parse_instruction(line: &str) -> Option<Instruction> {
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

fn parse_org_table(org_table: &str) -> Vec<Instruction> {
  let mut lines: Vec<&str> = org_table.lines().collect();
  let mut column_titles: Vec<&str> = lines[0].split("|").collect();
  column_titles = column_titles[1..].to_vec();
  column_titles.pop();
  for title in column_titles {
    print!("{}\n", title.trim());
  }
  lines = lines[3..].to_vec();
  lines.pop();
  let mut instructions: Vec<Instruction> = Vec::new();
  for line in lines {
    instructions.push(parse_instruction(line).unwrap());
  }
  println!("{:?}", instructions[0].syntax);
  instructions
}

fn main() -> std::io::Result<()> {
  let mut file = File::open("rv64_i.org")?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;
  let instructions = parse_org_table(&contents);

  let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
  let mut file = BufWriter::new(File::create(&path).unwrap());

  let mut instruction_map = phf_codegen::Map::new();
  for instruction in instructions {
    rustfmt::write(instruction.create_implementation_source(), &mut file)
      .unwrap();
    instruction_map
      .entry(instruction.mnemonic.clone(), &instruction.as_source());
  }
  rustfmt::write(
    format!("#[allow(unused_must_use)]\nstatic INSTRUCTIONS: phf::Map<&'static str, InstructionSource> = {};\n",
    instruction_map.build()),
    &mut file,
  )
  .unwrap();
  Ok(())
}

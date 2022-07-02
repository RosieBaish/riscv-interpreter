use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::{BufWriter, Write};
use std::path::Path;

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
  fn as_source(&self) -> String {
    let syntax_str = "&[\"".to_string() + &self.syntax.join("\", \"") + "\"]";
    format!("Instruction {{ mnemonic: \"{}\", expansion: \"{}\", syntax: {}, description: r#\"{}\"#, implementation: \"{}\", }}", self.mnemonic, self.expansion, syntax_str, self.description, self.implementation)
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
  lines = lines[2..].to_vec();
  lines.pop();
  let mut instructions: Vec<Instruction> = Vec::new();
  for line in lines {
    instructions.push(parse_instruction(line).unwrap());
  }
  println!("{:?}", instructions[0].syntax);
  instructions
}

fn main() -> std::io::Result<()> {
  let mut file = File::open("../asm_spec/rv64_i.org")?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;
  let instructions = parse_org_table(&contents);

  let path = Path::new(&env::var("OUT_DIR").unwrap()).join("codegen.rs");
  let mut file = BufWriter::new(File::create(&path).unwrap());

  let mut instruction_map = phf_codegen::Map::new();
  for instruction in instructions {
    instruction_map
      .entry(instruction.mnemonic.clone(), &instruction.as_source());
  }
  write!(
    &mut file,
    "static INSTRUCTIONS: phf::Map<&'static str, Instruction> = {}",
    instruction_map.build(),
  )
  .unwrap();
  write!(&mut file, ";\n").unwrap();
  Ok(())
}

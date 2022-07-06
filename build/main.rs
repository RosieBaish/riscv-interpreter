use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;
use tera::{Context, Tera};

mod instruction;
use instruction::Instruction;
mod rustfmt;

fn create_html(instructions: &Vec<Instruction>) {
  // Use globbing
  let tera = Tera::new("templates/*.html").expect("Parsing error(s):");

  let path = Path::new("./www/index.html");
  let mut file = BufWriter::new(File::create(&path).unwrap());

  let mut context = Context::new();
  context.insert("instruction", &instructions);

  tera
    .render_to("index.html", &context, &mut file)
    .expect("Rendering Error");
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
    instructions.push(Instruction::parse(line).unwrap());
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
  for instruction in &instructions {
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

  create_html(&instructions);

  Ok(())
}

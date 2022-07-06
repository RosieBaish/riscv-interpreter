use serde::Serialize;
use std::collections::HashMap;
use std::convert::TryInto;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::path::Path;
use tera::{Context, Tera};

mod instruction;
use instruction::Instruction;
mod rustfmt;

fn create_html(instructions: &Vec<Instruction>, registers: &Vec<Register>) {
  // Use globbing
  let tera = Tera::new("templates/*.html").expect("Parsing error(s):");

  let path = Path::new("./www/index.html");
  let mut file = BufWriter::new(File::create(&path).unwrap());

  let mut context = Context::new();
  context.insert("instructions", &instructions);
  context.insert("registers", &registers);

  tera
    .render_to("index.html", &context, &mut file)
    .expect("Rendering Error");
}

#[allow(dead_code)]
#[derive(Debug, Serialize)]
struct Register {
  primary_name: String,
  secondary_names: Vec<String>,
  description: String,
  saver: String,
}

impl Register {
  fn parse(cells: [&str; 4]) -> Option<Register> {
    if let [primary, secondary, desc, saver] = &cells[..] {
      Some(Register {
        primary_name: primary.trim().to_string(),
        secondary_names: secondary
          .trim()
          .split("/")
          .map(str::trim)
          .map(str::to_string)
          .collect(),
        description: desc.trim().to_string(),
        saver: saver.trim().to_string(),
      })
    } else {
      println!("{:?}", cells);
      None
    }
  }
}

fn parse_org_file(filename: &str) -> (Vec<Instruction>, Vec<Register>) {
  let mut file = File::open(filename).expect("File open error");
  let mut contents = String::new();
  file.read_to_string(&mut contents).expect("File read error");

  let mut sections: HashMap<&str, Vec<&str>> = HashMap::new();

  let mut current_title: &str = "";
  let mut current_section: Vec<&str> = Vec::new();
  for line in contents.lines() {
    if line.starts_with("* ") {
      if current_section.len() != 0 {
        sections.insert(current_title, current_section);
        current_section = Vec::new();
      }
      current_title = &line[2..];
    } else {
      current_section.push(line);
    }
  }
  sections.insert(current_title, current_section);

  let instruction_strings: Vec<[&str; 5]> =
    parse_org_table(sections.get("Instructions").unwrap());
  let register_strings: Vec<[&str; 4]> =
    parse_org_table(sections.get("Registers").unwrap());
  (
    instruction_strings
      .into_iter()
      .map(Instruction::parse)
      .map(Option::unwrap)
      .collect(),
    register_strings
      .into_iter()
      .map(Register::parse)
      .map(Option::unwrap)
      .collect(),
  )
}

fn parse_org_table<'a, const NUM_COLUMNS: usize>(
  table: &Vec<&'a str>,
) -> Vec<[&'a str; NUM_COLUMNS]> {
  let mut column_titles: Vec<&str> = table[0].split("|").collect();
  column_titles = column_titles[1..].to_vec();
  column_titles.pop();
  for title in column_titles {
    print!("{}\n", title.trim());
  }
  let mut cells: Vec<[&str; NUM_COLUMNS]> = Vec::new();
  let mut found_split = false;
  for line in table {
    if line.starts_with("|-") {
      found_split = true;
      continue;
    }
    if !found_split {
      continue;
    }
    if line.trim() == "" {
      continue;
    }
    let mut cols: Vec<&str> = line.split("|").collect();
    cols = cols[1..].to_vec();
    cols.pop();
    println!("{}: {:?}", line, cols);
    cells.push(cols.try_into().unwrap());
  }
  cells
}

fn main() -> std::io::Result<()> {
  let (instructions, registers) = parse_org_file("rv64_i.org");

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

  create_html(&instructions, &registers);

  Ok(())
}

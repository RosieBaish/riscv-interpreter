use crate::rustfmt;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::io::{Error, ErrorKind};
use std::path::Path;

/// code is a string that starts with optional whitespace followed by
/// one of { ( [. It then searches until it finds the corresponding
/// close bracket. It assumes that all brackets are matched, e.g. that
/// the source is well formed and doesn't contain any comments or strings
/// which contain non-matching brackets.
fn find_matching<'a>(code: &'a str) -> Option<&'a str> {
  let mut num_levels = 0;
  let trimmed_code = code.trim();
  let opening = trimmed_code.chars().nth(0)?;
  println!(
    "code: {}; trimmed: {}; opening: {}",
    code[..10].to_string(),
    trimmed_code[..10].to_string(),
    opening
  );
  let closing = match opening {
    '{' => '}',
    '[' => ']',
    '(' => ')',
    _ => {
      println!("Couldn't work out what to match");
      return None;
    }
  };

  let mut closing_index = 0;
  for (i, c) in trimmed_code.chars().enumerate() {
    if c == opening {
      num_levels += 1;
    } else if c == closing {
      num_levels -= 1;
    }
    println!("{}, {}: {}", i, c, num_levels);
    if num_levels <= 0 {
      closing_index = i;
      break;
    }
  }

  if closing_index > 1 {
    return Some(&trimmed_code[1..closing_index]);
  }

  None
}

fn find_structure<'a>(code: &'a str, name: &'a str) -> Option<&'a str> {
  let index = code.find(name)? + name.len();
  let remainder = &code[index..];
  println!(
    "find_structure({}, {}); index: {}; remainder: {};",
    code, name, index, remainder
  );
  assert_eq!(&remainder[0..2], " {");
  find_matching(remainder)
}

#[derive(Debug)]
struct FunctionDefn {
  name: String,
  self_arg: String,
  args: Vec<(String, String)>,
  return_type: Option<String>,
}

impl FunctionDefn {
  fn parse(code: &str) -> Option<FunctionDefn> {
    assert!(code.trim().starts_with("fn "));
    println!("{}", code.trim());

    let (name, _) = &code.trim().strip_prefix("fn ")?.split_once("(")?;
    println!("{}", name);
    let remainder = &code.trim().strip_prefix("fn ")?.strip_prefix(name)?;
    let args = find_matching(remainder)?;
    println!("{}", args);

    let (self_arg, other_args) = match args.split_once(",") {
      Some((l, r)) => (l, r),
      None => (args, ""),
    };

    let option_return_type = match remainder.split_once(" -> ") {
      None => None,
      Some((_, r)) => Some(r.trim().strip_suffix(";")?.to_string()),
    };

    let mut vec_args: Vec<(String, String)> = Vec::new();
    for arg in other_args.split(",").map(|s| s.trim()) {
      if arg == "" {
        continue;
      }
      println!("{}", arg);
      let (arg_name, arg_type) = arg.split_once(": ")?;
      println!("{}: {}", arg_name, arg_type);
      vec_args.push((arg_name.to_string(), arg_type.to_string()))
    }

    Some(FunctionDefn {
      name: name.to_string(),
      self_arg: self_arg.to_string(),
      args: vec_args,
      return_type: option_return_type,
    })
  }

  fn parse_lines(code: &str) -> Vec<FunctionDefn> {
    code
      .trim()
      .lines()
      .map(FunctionDefn::parse)
      .flatten()
      .collect()
  }
}

fn get_enum_members(code: &str) -> Vec<String> {
  let mut members: Vec<String> = Vec::new();
  for line in code.trim().lines() {
    println!("{}", line);
    if let [name, _, comma] = &line
      .split(&['(', ')'][..])
      .map(|s| s.to_string())
      .collect::<Vec<String>>()[..]
    {
      assert_eq!(comma, ",");
      members.push(name.to_string());
    } else {
      panic!("Enum definition was not of the form \"name(type),\"");
    }
  }

  members
}

pub fn create_dispatch_file() -> std::io::Result<()> {
  let mut output_file = BufWriter::new(
    File::create(
      Path::new(&env::var_os("OUT_DIR").unwrap())
        .join("interpreter_dispatch.rs"),
    )
    .expect("File open error"),
  );

  let mut interpreter_file =
    File::open("src/interpreter.rs").expect("File open error");

  let mut contents = String::new();
  interpreter_file
    .read_to_string(&mut contents)
    .expect("File read error");

  let trait_defn = find_structure(&contents, "pub trait InterpreterTrait")
    .ok_or(Error::from(ErrorKind::NotFound))?;
  let enum_defn = find_structure(&contents, "enum Architecture")
    .ok_or(Error::from(ErrorKind::NotFound))?;

  println!(
    "trait_defn: {}[...]{}",
    trait_defn[..10].to_string(),
    trait_defn[trait_defn.len() - 10..].to_string()
  );

  let function_defns = FunctionDefn::parse_lines(trait_defn);
  println!("{:?}", function_defns);

  let enum_members = get_enum_members(enum_defn);

  let implementations: Vec<String> = function_defns
    .iter()
    .map(|function_defn| {
      let preamble = format!(
        "#[allow(dead_code)]
pub fn {}({}, {}) -> {} {{
match {}.architecture {{
",
        function_defn.name,
        function_defn.self_arg,
        function_defn
          .args
          .iter()
          .map(|(n, t)| format!("{}: {}", n, t))
          .collect::<Vec<String>>()
          .join(","),
        function_defn
          .return_type
          .as_ref()
          .unwrap_or(&"()".to_string()),
        function_defn.self_arg,
      );
      let member_lines: Vec<String> = enum_members
        .iter()
        .map(|name| {
          format!(
            "{}(architecture) => architecture.{}({}),",
            name,
            function_defn.name,
            function_defn
              .args
              .iter()
              .map(|(n, _)| n.to_string())
              .collect::<Vec<String>>()
              .join(",")
          )
        })
        .collect();
      let postamble = "}\n}\n";
      format!("{}\n{}\n{}", preamble, member_lines.join("\n\t"), postamble)
        .to_string()
    })
    .collect();

  rustfmt::write(
    format!(
      "use crate::interpreter::Architecture::*;
impl Interpreter {{\n{}\n}}",
      implementations.join("\n"),
    )
    .to_string(),
    &mut output_file,
  )
  .unwrap();

  Ok(())
}

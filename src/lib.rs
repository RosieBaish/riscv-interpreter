mod utils;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Debug)]
struct Instruction {
  mnemonic: &'static str,
  expansion: &'static str,
  syntax: &'static [&'static str],
  description: &'static str,
  implementation: &'static str,
}

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
  fn alert(s: &str);
}

fn parse_reg(code: &str) -> Option<u32> {
  let leaders = code.split(|c: char| !c.is_ascii_alphanumeric());
  None
}

impl Instruction {
  pub fn parse(&self, orig_code: &str) -> Vec<u32> {
    let code: &mut str = orig_code.clone();
    let mut syntax_index = 0;
    while code != "" {
      code = code.trim();
    }
    return vec![];
  }
}

#[wasm_bindgen]
pub struct Interpreter {
  code: String,
  // Instructions??
}

#[wasm_bindgen]
impl Interpreter {
  pub fn new() -> Interpreter {
    //    alert("Init");
    Interpreter {
      code: "".to_string(),
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
    self.update_if_necessary();
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

  fn parse(&self) {
    for (ln, line) in self.code.lines().enumerate() {
      let line_num = ln + 1; // Source is 1 indexed
      let instruction = line.split("//").nth(0).unwrap().trim();
      if instruction == "" {
        continue;
      }
      let opt_inst: Option<&Instruction> = INSTRUCTIONS.get(instruction);
      if opt_inst.is_none() {
        alert(
          format!("Invalid instruction on line {}: {}", line_num, instruction)
            .as_str(),
        );
        continue;
      }
      let inst: &Instruction = opt_inst.unwrap();
      let args = inst.parse(instruction);
      alert(format!("{:?}", args).as_str());
    }
  }
}

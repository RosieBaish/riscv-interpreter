use crate::interpreter::*;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
  pub fn alert(s: &str);
}

#[wasm_bindgen]
impl Interpreter {
  pub fn new() -> Interpreter {
    //    alert("Init");
    Interpreter {
      code: "".to_string(),
      instructions: Vec::new(),
      registers: [0; 32],
      errors: Vec::new(),
      frequency: Some(0),
    }
  }

  fn update_if_necessary(&mut self) {
    let c: String = self.get_code();
    if c.ne(&self.code) {
      self.code = c;
      self.parse();
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

  pub fn run_button(&mut self) {
    let mut pc: u64 = 0;
    self.update_if_necessary();
    for inst in &self.instructions {
      (inst.implementation)(&mut self.registers, &mut pc);
    }
    alert(format!("{:?}", self.registers).as_str());
  }

  pub fn step_button(&mut self) {
    self.update_if_necessary();
    //    parse("hello".to_string());
    //    alert("Step");
  }

  pub fn reset_button(&self) {
    //    parse("hello".to_string());
    //    alert("Reset");
  }

  pub fn stop_button(&self) {
    //    alert("Stop");
  }

  pub fn get_errors(&self) -> *const String {
    self.errors.as_ptr()
  }

  pub fn set_freqency(&mut self, unlimited: bool, freq: u32) {
    if unlimited {
      self.frequency = None;
    } else {
      self.frequency = Some(freq);
    }
  }
}

use crate::interpreter::*;
use crate::rv64_i::MEMORY_SIZE;
use crate::utils;
use crate::PC;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
  pub fn alert(s: &str);
}

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[macro_export]
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
impl Interpreter {
  pub fn new() -> Interpreter {
    utils::set_panic_hook();
    let interpreter = Interpreter {
      code: "".to_string(),
      instructions: Vec::new(),
      registers: [0; 32],
      memory: [0; MEMORY_SIZE],
      pc: PC::new(),
      errors: Vec::new(),
      frequency: Some(0),
      running: false,
    };
    interpreter.update_memory();
    interpreter
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

  fn get_initial_registers(&mut self) {
    self.registers[0] = 0;
    let window = web_sys::window().expect("global window does not exists");
    let document = window.document().expect("expecting a document on window");
    let registers: web_sys::HtmlCollection = document
      .get_element_by_id("registers")
      .unwrap()
      .get_elements_by_class_name("init-value");
    for i in 1..32 {
      let init_string: String = registers
        .item((i as u32) - 1) // -1 because x0 has no init-value field
        .unwrap()
        .dyn_into::<web_sys::HtmlInputElement>()
        .unwrap()
        .value();
      log!("{}: {}\n", i, init_string);
      self.registers[i] = parse_int::parse::<u64>(&init_string).unwrap();
    }
  }

  fn update_registers(&self) {
    assert_eq!(self.registers[0], 0);
    let window = web_sys::window().expect("global window does not exists");
    let document = window.document().expect("expecting a document on window");
    for i in 1..32 {
      document
        .get_element_by_id(format!("register_{}_decimal", i).as_str())
        .unwrap()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap()
        .set_inner_text(format!("{}", self.registers[i]).as_str());
      document
        .get_element_by_id(format!("register_{}_hex", i).as_str())
        .unwrap()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap()
        .set_inner_text(format!("0x{:016X}", self.registers[i]).as_str());
      document
        .get_element_by_id(format!("register_{}_binary", i).as_str())
        .unwrap()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap()
        .set_inner_text(format!("0b{:064b}", self.registers[i]).as_str());
    }
  }

  fn try_update_memory(
    &self,
    document: &web_sys::Document,
    memory_element: &web_sys::HtmlElement,
  ) -> Option<()> {
    // This doesn't actually need to be bool
    let memory_address_str: String = document
      .get_element_by_id("memory-address")?
      .dyn_into::<web_sys::HtmlInputElement>()
      .ok()?
      .value();
    log!("memory_address: \"{}\"\n", memory_address_str);
    let memory_address: u32 =
      parse_int::parse::<u32>(&memory_address_str).ok()?;

    const BYTES_PER_ROW: u32 = 16; // Must be a power of 2
    const NUM_ROWS: u32 = 10;
    let mut start = memory_address & (!(BYTES_PER_ROW - 1));
    if start > ((NUM_ROWS / 2) * BYTES_PER_ROW) {
      start = start - ((NUM_ROWS / 2) * BYTES_PER_ROW);
    } else {
      start = 0;
    }
    let mut end = start + (NUM_ROWS * BYTES_PER_ROW);
    if end > 0x8000000 {
      end = 0x8000000;
      start = end - (NUM_ROWS * BYTES_PER_ROW)
    }

    let mut memory_table: String = String::from("<tr>");
    for row_start in (start..end).step_by(BYTES_PER_ROW as usize) {
      memory_table.push_str(&format!("<td>0x{:08x}</td>", row_start));
      for byte in row_start..(row_start + BYTES_PER_ROW) {
        memory_table
          .push_str(&format!("<td>{:02x}</td>", self.memory[byte as usize]));
      }
      let build_string_vec: Vec<String> = self.memory
        [row_start as usize..(row_start + BYTES_PER_ROW) as usize]
        .iter()
        .map(|num| {
          if *num >= 32 && *num <= 126 {
            (*num as char).to_string()
          } else {
            '.'.to_string()
          }
        })
        .collect();
      memory_table.push_str(&format!("<td>{}</td>", build_string_vec.join("")));
      memory_table.push_str("</tr>");
    }

    memory_element.set_inner_html(&memory_table);
    Some(())
  }

  pub fn update_memory(&self) {
    let window = web_sys::window().expect("global window does not exists");
    let document = window.document().expect("expecting a document on window");
    let memory_element = document
      .get_element_by_id("memory")
      .unwrap()
      .dyn_into::<web_sys::HtmlElement>()
      .unwrap();

    match self.try_update_memory(&document, &memory_element) {
      Some(()) => (),
      None => {
        memory_element.set_inner_html(format!("<tr class=\"danger\"><td>Invalid memory location or error printing memory. Memory addresses must be from 0x00000000 - 0x7fffffff</td></tr>").as_str());
      }
    }
  }

  fn start(&mut self) {
    self.update_if_necessary();
    self.get_initial_registers();
    self.running = true;
  }

  pub fn run_button(&mut self) {
    if !self.running {
      self.start();
    }
    // 4 bytes/instruction
    let max_pc: u64 = self.instructions.len() as u64 * 4;
    while self.pc.get() < max_pc {
      self.step();
    }
    self.running = false;
  }

  fn step(&mut self) {
    log!("{:?}; {}", self.registers, self.pc.get());
    self.pc.changed = false;
    let inst = &self.instructions[(self.pc.get() / 4) as usize];
    log!("{:?}", inst);
    (inst.implementation)(&mut self.registers, &mut self.pc, &mut self.memory);
    if !self.pc.changed {
      self.pc.inc(4);
    }
    self.registers[0] = 0;
  }

  pub fn step_button(&mut self) {
    if !self.running {
      self.start();
    }
    self.step();
    self.update_registers();
    self.update_memory();
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

use crate::instruction::*;
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

#[wasm_bindgen]
pub struct WebInterface {
  interpreter: Interpreter,
}

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! log_inner {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! log_inner {
    ( $( $t:tt )* ) => {
      println!($( $t )* );
    }
}

#[macro_export]
macro_rules! function {
  () => {{
    fn f() {}
    fn type_name_of<T>(_: T) -> &'static str {
      std::any::type_name::<T>()
    }
    let name = type_name_of(f);
    &name[..name.len() - 3]
  }};
}

#[macro_export]
macro_rules! log {
    ($($tts:tt)*) => {
      crate::log_inner!("{}: {}", crate::function!(), format!($($tts)*));
    }
}

#[wasm_bindgen]
impl WebInterface {
  pub fn new() -> WebInterface {
    utils::set_panic_hook();
    let interpreter = Interpreter {
      code: "".to_string(),
      instructions: Vec::new(),
      registers: [Register { value: 0 }; 32],
      memory: [0; MEMORY_SIZE],
      pc: PC::new(),
      errors: Vec::new(),
      warnings: Vec::new(),
      frequency: Some(0),
      running: false,
    };
    WebInterface {
      interpreter: interpreter,
    }
  }

  pub fn update_if_necessary(&mut self) {
    let c: String = self.get_code();
    if c.ne(&self.interpreter.code) {
      self.interpreter.code = c;
      self.interpreter.parse();
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
    self.interpreter.registers[0] = Register { value: 0 };
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
      self.interpreter.registers[i] = Register {
        value: parse_int::parse::<u64>(&init_string).unwrap(),
      };
    }
  }

  fn update_registers(&self) {
    assert_eq!(self.interpreter.registers[0].value, 0);
    for i in 1..32 {
      self.set_inner_html(
        format!("register_{}_decimal", i).as_str(),
        format!("{}", self.interpreter.registers[i]).as_str(),
      );
      self.set_inner_html(
        format!("register_{}_hex", i).as_str(),
        format!("0x{:016X}", self.interpreter.registers[i]).as_str(),
      );
      self.set_inner_html(
        format!("register_{}_binary", i).as_str(),
        format!("0b{:064b}", self.interpreter.registers[i]).as_str(),
      );
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
    if end > MEMORY_SIZE as u32 {
      end = MEMORY_SIZE as u32;
      start = end - (NUM_ROWS * BYTES_PER_ROW)
    }

    let mut memory_table: String = String::from("<tr>");
    for row_start in (start..end).step_by(BYTES_PER_ROW as usize) {
      memory_table.push_str(&format!("<td>0x{:08x}</td>", row_start));
      for byte in row_start..(row_start + BYTES_PER_ROW) {
        memory_table.push_str(&format!(
          "<td>{:02x}</td>",
          self.interpreter.memory[byte as usize]
        ));
      }
      let build_string_vec: Vec<String> = self.interpreter.memory
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

  pub fn update_ui(&self) {
    self.update_registers();
    self.update_memory();

    self.set_parent_visibility("reset", true);
    self.set_parent_visibility("step", !self.interpreter.running);
    self.set_parent_visibility("run", !self.interpreter.running);
    self.set_parent_visibility("stop", self.interpreter.running);

    self.set_inner_html("errors", &self.interpreter.errors.join("<br>"));
    self
      .set_id_visibility("errors-container", self.interpreter.errors.len() > 0);
    self.set_inner_html("warnings", &self.interpreter.warnings.join("<br>"));
    self.set_id_visibility(
      "warnings-container",
      self.interpreter.warnings.len() > 0,
    );

    self.set_breakpoints_and_current_line();
  }

  pub fn run_button(&mut self) {
    self.interpreter.run();
    self.update_ui();
  }

  pub fn step_button(&mut self) {
    self.interpreter.running = true;
    self.update_ui();
    self.interpreter.step();
    self.interpreter.running = false;
    self.update_ui();
  }

  pub fn reset_button(&mut self) {
    panic!("Not implemented yet")
    /*
     * self.interpreter.reset();
     * self.set_inner_html(
     *   "recent-instruction",
     *   "The most recent instructions will be shown here when stepping.",
     * );
     * self.update_ui();
     */
  }

  pub fn stop_button(&mut self) {
    self.interpreter.stop();
    self.update_ui();
  }

  pub fn get_errors(&self) -> *const String {
    self.interpreter.errors.as_ptr()
  }

  pub fn set_freqency_button(&mut self, unlimited: bool, freq: u32) {
    if unlimited {
      self.interpreter.frequency = None;
      self.set_inner_html(
        "freq",
        "CPU: Unrestricted <span class=\"caret\"></span>",
      );
    } else {
      self.interpreter.frequency = Some(freq);
      self.set_inner_html(
        "freq",
        &format!("CPU: {} Hz <span class=\"caret\"></span>", freq).as_str(),
      );
    }
  }

  fn set_parent_visibility(&self, id: &str, visible: bool) {
    //log!("set_parent_visibility({}, {})", id, visible);
    let window = web_sys::window().expect("global window does not exists");
    let document = window.document().expect("expecting a document on window");
    let display_str = if visible { "" } else { "none" };
    document
      .get_element_by_id(id)
      .unwrap() // De-optionify
      .parent_element()
      .unwrap() // De-optionify
      .dyn_into::<web_sys::HtmlElement>()
      .unwrap()
      .style()
      .set_property("display", display_str)
      .expect("able to change element");
  }

  fn set_id_visibility(&self, id: &str, visible: bool) {
    //log!("set_id_visibility({}, {})", id, visible);
    let window = web_sys::window().expect("global window does not exists");
    let document = window.document().expect("expecting a document on window");
    let display_str = if visible { "" } else { "none" };
    document
      .get_element_by_id(id)
      .unwrap() // De-optionify
      .dyn_into::<web_sys::HtmlElement>()
      .unwrap()
      .style()
      .set_property("display", display_str)
      .expect("able to change element");
  }

  fn set_inner_html(&self, id: &str, html: &str) {
    let window = web_sys::window().expect("global window does not exists");
    let document = window.document().expect("expecting a document on window");
    //log!("set_inner_html({}, {})", id, html);
    document
      .get_element_by_id(id)
      .unwrap() // De-optionify
      .dyn_into::<web_sys::HtmlElement>()
      .unwrap()
      .set_inner_html(html);
  }

  fn add_class_if_missing(&self, element: &web_sys::HtmlElement, class: &str) {
    let classes: web_sys::DomTokenList = element.class_list();
    if !classes.contains(class) {
      classes.add_1(class).ok();
    }
  }

  fn remove_class_if_present(
    &self,
    element: &web_sys::HtmlElement,
    class: &str,
  ) {
    let classes: web_sys::DomTokenList = element.class_list();
    if classes.contains(class) {
      classes.remove_1(class).ok();
    }
  }

  fn set_breakpoints_and_current_line(&self) {
    let window = web_sys::window().expect("global window does not exists");
    let document = window.document().expect("expecting a document on window");
    let lines_elements: web_sys::HtmlCollection =
      document.get_elements_by_class_name("codelines");
    // It's dynamically initialised so may be missing on startup
    if lines_elements.length() == 0 {
      return;
    }
    assert_eq!(lines_elements.length(), 1);

    let max_line_num = self
      .interpreter
      .instructions
      .iter()
      .map(|i| i.line_num)
      .max()
      .unwrap_or(0);

    let lines_element = lines_elements.item(0).unwrap();
    log!("{}, {}", lines_element.child_element_count(), max_line_num);
    if lines_element.child_element_count() < max_line_num {
      return;
    }

    let lines: web_sys::HtmlCollection = lines_element.children();

    assert!(lines.length() > max_line_num);
    // Have to create and then set, because of blank lines
    let mut is_break: Vec<bool> = Vec::with_capacity(max_line_num as usize);
    for _i in 0..lines.length() {
      is_break.push(false);
    }
    for instruction in &self.interpreter.instructions {
      is_break[(instruction.line_num - 1/* 1 indexed */) as usize] =
        instruction.breakpoint;
    }

    for line_num in 0..lines.length() {
      let line = lines
        .item(line_num)
        .unwrap()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();
      self.remove_class_if_present(&line, "lineselect");
      if is_break[line_num as usize] {
        line
          // Unicode big red dot
          .set_inner_html(format!("🔴 {}", line_num + 1).as_str());
      } else {
        line.set_inner_html(format!("{}", line_num + 1).as_str());
      }
    }
    let pc_line_num = (self.interpreter.pc.get().value / 4) as usize;
    if pc_line_num < self.interpreter.instructions.len() {
      let next_inst_line = lines
        .item(
          self.interpreter.instructions[pc_line_num].line_num - 1, /* 1 indexed */
        )
        .unwrap()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();
      self.add_class_if_missing(&next_inst_line, "lineselect");
    }
  }

  pub fn toggle_breakpoint(&mut self, line_num: &str) {
    log!("toggle_breakpoint({})", line_num);
    let ln =
      parse_int::parse::<u32>(line_num.trim_matches(|c| !char::is_numeric(c)))
        .unwrap();

    for mut instruction in self.interpreter.instructions.iter_mut() {
      if instruction.line_num == ln {
        instruction.breakpoint = !instruction.breakpoint;
        log!("{:?}", instruction);
      }
    }
    self.set_breakpoints_and_current_line();
  }

  pub fn start(&mut self) {
    self.update_if_necessary();
    self.get_initial_registers();
    self.update_ui();
  }
}
impl Interpreter {
  fn run(&mut self) {
    self.running = true;
    // 4 bytes/instruction
    let max_pc: u64 = self.instructions.len() as u64 * 4;
    while self.pc.get().value < max_pc {
      self.step();
    }
    self.running = false;
  }

  fn step(&mut self) {
    log!("{:?}; {}", self.registers, self.pc.get().value);
    self.pc.changed = false;
    let inst = &self.instructions[(self.pc.get().value / 4) as usize];
    log!("{:?}", inst);
    (inst.implementation)(&mut self.registers, &mut self.pc, &mut self.memory);
    if !self.pc.changed {
      self.pc.inc(Register { value: 4 });
    }
    self.registers[0] = Register { value: 0 };
  }

  fn stop(&mut self) {
    self.running = false;
  }
  /*
   * Commented out because this is now totally wrong
   * pub fn reset(&mut self) {
   *   self.stop();
   *   *self = Interpreter::new();
   *   self.start();
   * }
   */
}

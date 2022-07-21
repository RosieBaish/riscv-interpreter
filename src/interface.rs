use crate::interpreter::*;
use crate::utils;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
  pub fn alert(s: &str);
}

#[wasm_bindgen]
pub struct WebInterface {
  interpreter: Interpreter,
  code_changed: bool,
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
      crate::log_inner!("{}:{} - {}", crate::function!(), std::line!(), format!($($tts)*));
    }
}

// Not part of the impl because it's called in the constructor
fn get_initial_registers() -> Vec<String> {
  let window = web_sys::window().expect("global window does not exists");
  let document = window.document().expect("expecting a document on window");
  let registers: web_sys::HtmlCollection = document
    .get_element_by_id("registers")
    .unwrap()
    .get_elements_by_class_name("init-value");
  let mut values: Vec<String> = Vec::new();
  for i in 0..registers.length() {
    let init_string: String = registers
      .item(i)
      .unwrap()
      .dyn_into::<web_sys::HtmlInputElement>()
      .unwrap()
      .value();
    values.push(init_string)
  }
  values
}

#[wasm_bindgen]
impl WebInterface {
  pub fn new() -> WebInterface {
    utils::set_panic_hook();
    let interpreter = Interpreter::create(get_initial_registers());
    WebInterface {
      interpreter: interpreter,
      code_changed: false,
    }
  }

  fn update_code(&mut self) {
    let window = web_sys::window().expect("global window does not exists");
    let document = window.document().expect("expecting a document on window");
    let code_text = document
      .get_element_by_id("code")
      .unwrap() // De-optionify
      .dyn_into::<web_sys::HtmlTextAreaElement>() // Cast
      .unwrap(); // Unwrap the cast result
    let code: String = code_text.value();
    self.interpreter.set_code(code);
  }

  pub fn code_change(&mut self) {
    if self.interpreter.running() {
      self.interpreter.stop();
    }
    self.code_changed = true;
  }

  fn update_registers(&self) {
    let register_representations = self.interpreter.registers_repr();
    for (i, (dec, hex, bin)) in register_representations.iter().enumerate() {
      self.set_inner_html(format!("register_{}_decimal", i).as_str(), dec);
      self.set_inner_html(format!("register_{}_hex", i).as_str(), hex);
      self.set_inner_html(format!("register_{}_binary", i).as_str(), bin);
    }
  }

  fn try_update_memory(
    &self,
    document: &web_sys::Document,
    memory_element: &web_sys::HtmlElement,
  ) -> Option<()> {
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
    if end > self.interpreter.memory_size() {
      end = self.interpreter.memory_size();
      start = end - (NUM_ROWS * BYTES_PER_ROW)
    }

    let mut memory_table: String = String::from("<tr>");
    for row_start in (start..end).step_by(BYTES_PER_ROW as usize) {
      memory_table.push_str(&format!(
        "<td>0x{:08x}</td><td>{}</td>",
        row_start,
        self
          .interpreter
          .memory_byte_repr(row_start as usize, BYTES_PER_ROW as usize)
          .join("</td><td>")
      ));
      let build_string_vec: Vec<String> = self
        .interpreter
        .memory_ascii_repr(row_start as usize, BYTES_PER_ROW as usize);
      memory_table
        .push_str(&format!("<td>{}</td>", build_string_vec.join("</td><td>")));
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

    let running = self.interpreter.running();
    self.set_parent_visibility("reset", true);
    self.set_parent_visibility("step", !running);
    self.set_parent_visibility("run", !running);
    self.set_parent_visibility("stop", running);

    let errors = self.interpreter.errors();
    self.set_inner_html("errors", &errors.join("<br>"));
    self.set_id_visibility("errors-container", errors.len() > 0);
    let warnings = self.interpreter.warnings();
    self.set_inner_html("warnings", &warnings.join("<br>"));
    self.set_id_visibility("warnings-container", warnings.len() > 0);

    self.set_breakpoints_and_current_line();
  }

  pub fn run_button(&mut self) {
    if self.code_changed {
      self.update_code();
      self.code_changed = false;
    }
    self.interpreter.run();
    self.update_ui();
  }

  pub fn step_button(&mut self) {
    if self.code_changed {
      self.update_code();
      self.code_changed = false;
    }
    self.interpreter.set_running(true);
    self.update_ui();
    self.interpreter.step();
    self.interpreter.set_running(false);
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

  pub fn set_freqency_button(&mut self, unlimited: bool, freq: u32) {
    if unlimited {
      self.interpreter.set_frequency(None);
      self.set_inner_html(
        "freq",
        "CPU: Unrestricted <span class=\"caret\"></span>",
      );
    } else {
      self.interpreter.set_frequency(Some(freq));
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

    let max_line_num = document
      .get_element_by_id("code")
      .unwrap() // De-optionify
      .dyn_into::<web_sys::HtmlTextAreaElement>() // Cast
      .unwrap() // Unwrap the cast result
      .value()
      .trim()
      .split("\n")
      .count() as u32;

    let lines_element = lines_elements.item(0).unwrap();
    log!("{}, {}", lines_element.child_element_count(), max_line_num);
    for i in lines_element.child_element_count()..max_line_num + 1 {
      log!("{}, {}", lines_element.child_element_count(), max_line_num);
      let new_div =
        document.create_element("div").expect("Couldn't create div");
      new_div
        .class_list()
        .add_1("lineno")
        .expect("Couldn't add class");
      new_div.set_inner_html(&i.to_string());
      lines_element
        .append_with_node_1(&new_div)
        .expect("failed to append new element");
    }

    let lines: web_sys::HtmlCollection = lines_element.children();

    assert!(lines.length() > max_line_num);
    // Have to create and then set, because of blank lines
    let is_break: Vec<bool> = self.interpreter.breakpoints();

    assert_eq!(lines.length(), is_break.len() as u32);

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
    let next_inst_line = lines
      .item(self.interpreter.next_inst_line_num())
      .unwrap()
      .dyn_into::<web_sys::HtmlElement>()
      .unwrap();
    self.add_class_if_missing(&next_inst_line, "lineselect");
  }

  pub fn toggle_breakpoint(&mut self, line_num: &str) {
    log!("toggle_breakpoint({})", line_num);
    let ln =
      parse_int::parse::<u32>(line_num.trim_matches(|c| !char::is_numeric(c)))
        .unwrap();

    self.interpreter.toggle_breakpoint(ln);
    self.set_breakpoints_and_current_line();
  }

  pub fn start(&mut self) {
    self.update_code();
    self.update_ui();
  }
}

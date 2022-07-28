use crate::interpreter::*;
use crate::utils;
use std::fmt::Write;
use std::sync::{Arc, Mutex};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
  pub fn alert(s: &str);
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
      $crate::log_inner!("{}:{} - {}", $crate::function!(), std::line!(), format!($($tts)*));
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
pub struct WebInterface {
  rci: Arc<Mutex<Interpreter>>,
  code_changed: bool,
  step_func_token: Option<i32>,
}

impl Default for WebInterface {
  fn default() -> Self {
    Self::new()
  }
}

#[wasm_bindgen]
impl WebInterface {
  pub fn new() -> WebInterface {
    utils::set_panic_hook();
    let interpreter = Interpreter::create_RiscV64_i(get_initial_registers());

    WebInterface {
      rci: Arc::new(Mutex::new(interpreter)),
      code_changed: false,
      step_func_token: None,
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
    self.rci.lock().unwrap().set_code(code);
  }

  pub fn code_change(&mut self) {
    if self.rci.lock().unwrap().running() {
      self.rci.lock().unwrap().stop();
    }
    self.code_changed = true;
  }

  fn update_registers(&self) {
    let register_representations = self.rci.lock().unwrap().registers_repr();
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
    let interpreter = self.rci.lock().unwrap();
    log!("memory_address: \"{}\"\n", memory_address_str);
    let memory_address: u32 =
      parse_int::parse::<u32>(&memory_address_str).ok()?;

    const BYTES_PER_ROW: u32 = 16; // Must be a power of 2
    const NUM_ROWS: u32 = 10;
    let mut start = memory_address & (!(BYTES_PER_ROW - 1));
    if start > ((NUM_ROWS / 2) * BYTES_PER_ROW) {
      start -= (NUM_ROWS / 2) * BYTES_PER_ROW;
    } else {
      start = 0;
    }
    let mut end = start + (NUM_ROWS * BYTES_PER_ROW);
    if end > interpreter.memory_size() {
      end = interpreter.memory_size();
      start = end - (NUM_ROWS * BYTES_PER_ROW)
    }

    let mut memory_table: String = String::from("<tr>");
    for row_start in (start..end).step_by(BYTES_PER_ROW as usize) {
      write!(
        memory_table,
        "<td>0x{:08x}</td><td>{}</td>",
        row_start,
        interpreter
          .memory_byte_repr(row_start as usize, BYTES_PER_ROW as usize)
          .join("</td><td>")
      )
      .ok()?;
      let build_string_vec: Vec<String> = interpreter
        .memory_ascii_repr(row_start as usize, BYTES_PER_ROW as usize);
      write!(
        memory_table,
        "<td>{}</td></tr>",
        build_string_vec.join("</td><td>")
      )
      .ok()?;
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
        memory_element.set_inner_html(
          "<tr class=\"danger\">
<td>Invalid memory location or error printing memory.
Memory addresses must be from 0x00000000 - 0x7fffffff</td></tr>",
        );
      }
    }
  }

  pub fn update_ui(&self) {
    self.update_registers();
    self.update_memory();

    {
      let interpreter = self.rci.lock().unwrap();
      let running = interpreter.running();
      self.set_parent_visibility("reset", true);
      self.set_parent_visibility("step", !running);
      self.set_parent_visibility("run", !running);
      self.set_parent_visibility("stop", running);

      let errors = interpreter.errors();
      self.set_inner_html("errors", &errors.join("<br>"));
      self.set_id_visibility("errors-container", !errors.is_empty());
      let warnings = interpreter.warnings();
      self.set_inner_html("warnings", &warnings.join("<br>"));
      self.set_id_visibility("warnings-container", !warnings.is_empty());
    }
    self.set_breakpoints_and_current_line();
  }

  pub fn run_button(&mut self) {
    if self.code_changed {
      self.update_code();
      self.code_changed = false;
    }
    self.rci.lock().unwrap().set_running(true);
    let interpreter = self.rci.clone();
    let step_func: Closure<dyn FnMut()> = Closure::new(move || {
      interpreter.lock().unwrap().step();
    });
    let window = web_sys::window().expect("global window does not exists");
    let interval: i32 = match self.rci.lock().unwrap().get_frequency() {
      Some(freq) => (1000 / freq) as i32,
      None => 1000,
    };
    let token = window
      .set_interval_with_callback_and_timeout_and_arguments_0(
        step_func.as_ref().unchecked_ref(),
        interval,
      )
      .expect("Managed to set callback");
    std::mem::forget(step_func);
    self.step_func_token = Some(token);
    self.update_ui();
  }

  pub fn step_button(&mut self) {
    if self.code_changed {
      self.update_code();
      self.code_changed = false;
    }
    self.rci.lock().unwrap().set_running(true);
    self.update_ui();
    self.rci.lock().unwrap().step();
    self.rci.lock().unwrap().set_running(false);
    self.update_ui();
  }

  pub fn reset_button(&mut self) {
    panic!("Not implemented yet")
    /*
     * log!("Test"); self.rci.lock().unwrap().reset();
     * self.set_inner_html(
     *   "recent-instruction",
     *   "The most recent instructions will be shown here when stepping.",
     * );
     * self.update_ui();
     */
  }

  pub fn stop_button(&mut self) {
    match self.step_func_token {
      Some(token) => {
        log!("Token is: {}", token);
        let window = web_sys::window().expect("global window does not exists");
        window.clear_interval_with_handle(token);
        self.step_func_token = None;
      }
      None => {
        log!("No Token");
      }
    }
    self.rci.lock().unwrap().stop();
    self.update_ui();
  }

  pub fn set_frequency_button(&mut self, unlimited: bool, freq: u32) {
    if unlimited {
      self.rci.lock().unwrap().set_frequency(None);
      self.set_inner_html(
        "freq",
        "CPU: Unrestricted <span class=\"caret\"></span>",
      );
    } else {
      self.rci.lock().unwrap().set_frequency(Some(freq));
      self.set_inner_html(
        "freq",
        format!("CPU: {} Hz <span class=\"caret\"></span>", freq).as_str(),
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

    let lines: web_sys::HtmlCollection =
      lines_elements.item(0).unwrap().children();

    let is_break: Vec<bool>;
    let next_inst_line_num: u32;

    // Extra scope for interpreter lock
    {
      let interpreter = self.rci.lock().unwrap();
      is_break = interpreter.breakpoints();
      next_inst_line_num = interpreter.next_inst_line_num();
    }

    for line_num in 0..std::cmp::min(lines.length(), is_break.len() as u32) {
      let line = lines
        .item(line_num)
        .unwrap()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();

      // Set the highlight for the next line
      if line_num == next_inst_line_num {
        self.add_class_if_missing(&line, "lineselect");
      } else {
        self.remove_class_if_present(&line, "lineselect");
      }

      // Add breakpoint symbol if required
      if is_break[line_num as usize] {
        line
          // Unicode big red dot
          .set_inner_html(format!("🔴 {}", line_num + 1).as_str());
      } else {
        line.set_inner_html(format!("{}", line_num + 1).as_str());
      }
    }
  }

  pub fn toggle_breakpoint(&mut self, line_num: &str) {
    log!("toggle_breakpoint({})", line_num);
    let ln =
      parse_int::parse::<u32>(line_num.trim_matches(|c| !char::is_numeric(c)))
        .unwrap();

    self.rci.lock().unwrap().toggle_breakpoint(ln);
    self.set_breakpoints_and_current_line();
  }

  pub fn start(&mut self) {
    self.update_code();
    self.update_ui();
  }

  pub fn running(&self) -> bool {
    self.rci.lock().unwrap().running()
  }
}

use std::borrow::Cow;
use std::env;
use std::io;
use std::io::*;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Write these bindings as source text to the given `Write`able.
pub fn write<W: std::io::Write>(
  text: String,
  writer: &mut BufWriter<W>,
) -> io::Result<()> {
  match rustfmt_generated_string(&text) {
    Ok(formatted_text) => {
      writer.write_all(formatted_text.as_bytes())?;
    }
    Err(err) => {
      eprintln!("Failed to run rustfmt: {} (non-fatal, continuing)", err);
      writer.write_all(text.as_bytes())?;
    }
  }
  Ok(())
}

/// Gets the rustfmt path to rustfmt the generated bindings.
fn rustfmt_path<'a>() -> io::Result<Cow<'a, PathBuf>> {
  if let Ok(rustfmt) = env::var("RUSTFMT") {
    return Ok(Cow::Owned(rustfmt.into()));
  }
  match which::which("rustfmt") {
    Ok(p) => Ok(Cow::Owned(p)),
    Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("{}", e))),
  }
}

/// Checks if rustfmt_bindings is set and runs rustfmt on the string
fn rustfmt_generated_string(source: &str) -> io::Result<Cow<str>> {
  let rustfmt = rustfmt_path()?;
  let mut cmd = Command::new(&*rustfmt);

  cmd.stdin(Stdio::piped()).stdout(Stdio::piped());

  let mut child = cmd.spawn()?;
  let mut child_stdin = child.stdin.take().unwrap();
  let mut child_stdout = child.stdout.take().unwrap();

  let source = source.to_owned();

  // Write to stdin in a new thread, so that we can read from stdout on this
  // thread. This keeps the child from blocking on writing to its stdout which
  // might block us from writing to its stdin.
  let stdin_handle = ::std::thread::spawn(move || {
    let _ = child_stdin.write_all(source.as_bytes());
    source
  });

  let mut output = vec![];
  io::copy(&mut child_stdout, &mut output)?;

  let status = child.wait()?;
  let source = stdin_handle.join().expect(
    "The thread writing to rustfmt's stdin doesn't do \
     anything that could panic",
  );

  match String::from_utf8(output) {
    Ok(bindings) => match status.code() {
      Some(0) => Ok(Cow::Owned(bindings)),
      Some(2) => Err(io::Error::new(
        io::ErrorKind::Other,
        "Rustfmt parsing errors.".to_string(),
      )),
      Some(3) => {
        println!("Rustfmt could not format some lines.");
        Ok(Cow::Owned(bindings))
      }
      _ => Err(io::Error::new(
        io::ErrorKind::Other,
        "Internal rustfmt error".to_string(),
      )),
    },
    _ => Ok(Cow::Owned(source)),
  }
}

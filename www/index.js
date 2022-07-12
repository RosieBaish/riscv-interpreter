import { Interpreter } from "riscv-interpreter";

"use strict";

const interpreter = Interpreter.new()
interpreter.update_ui();

document.getElementById('memory-go').onclick = () => interpreter.update_ui();
document.getElementById("run").onclick = () => interpreter.run_button();
document.getElementById("step").onclick = () => interpreter.step_button();
document.getElementById("reset").onclick = () => interpreter.reset_button();
document.getElementById("stop").onclick = () => interpreter.stop_button();


if (typeof(Storage) !== "undefined") {
  let previousSessionCode = localStorage.getItem("code");
  if (previousSessionCode !== null) {
    document.getElementById('code').value = previousSessionCode;
  }
  window.onbeforeunload = function() {
    localStorage.setItem("code", document.getElementById('code').value);
  };
}
$('#code').linedtextarea();
$('.codelines').on('click', '.lineno', function() {
  $(this).toggleClass('lineselect');
});
$('#code').bind('input propertychange', function() { // if the code changes, invalidate the current program instance
  stop();
  document.getElementById('recent-instruction').innerHTML = "<br>Execution automatically stopped because of code change."
});

function setFrequency(freq) {
  if (typeof freq === 'string' && freq == "unlimited") {
    interpreter.set_frequency(true, 0);
  } else {
    interpreter.set_frequency(false, freq);
  }
}

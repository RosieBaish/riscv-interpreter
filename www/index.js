import { WebInterface } from "riscv-interpreter";

"use strict";

const interpreter = WebInterface.new()

var interval_id = null;

document.getElementById('memory-go').onclick = () => interpreter.update_ui();
document.getElementById("run").onclick = () => {
  interval_id = setInterval(() => interpreter.update_ui(), 100);
  interpreter.run_button();
}
document.getElementById("step").onclick = () => interpreter.step_button();
document.getElementById("reset").onclick = () => interpreter.reset_button();
document.getElementById("stop").onclick = () => {
  interpreter.stop_button();
  if (interval_id !== null) {
    clearInterval(interval_id);
    interval_id = null;
    interpreter.update_ui();
  }
}

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
  interpreter.toggle_breakpoint($(this).html());
});

$('#code').bind('input propertychange', function() { // if the code changes, invalidate the current program instance
  interpreter.code_change();
  document.getElementById('recent-instruction').innerHTML = "Execution automatically stopped because of code change."
  interpreter.update_if_necessary();
});

interpreter.start();


function setFrequency(freq) {
  if (typeof freq === 'string' && freq == "unlimited") {
    interpreter.set_frequency_button(true, 0);
  } else {
    interpreter.set_frequency_button(false, freq);
  }
}

for (var i=0; i<=8; i++){
  var id_name = "set_freq_" + 2**i;
  var elem = document.getElementById(id_name);
  console.log(elem);
  elem.onclick = (function(i) {return function() {
    setFrequency(2**i);
  };})(i);
}

setFrequency(1);


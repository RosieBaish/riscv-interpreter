import { Interpreter } from "riscv-interpreter";

"use strict";

const interpreter = Interpreter.new()

setup();

document.getElementById('memory-go').onclick = () => interpreter.update_memory();


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
  document.getElementById('recent-instruction').innerHTML += "<br>Execution automatically stopped because of code change."
});

function getRecentInsnsHTML() {
  var recentInstructionHTML = "";
  for (var i = 0; i < recentInsns.length; ++i) {
    // mark the last element as the current instruction executed
    if (i === recentInsns.length-1) {
      recentInstructionHTML += '<span class="current">' + "[line " + recentInsns[i][0][1] + "]: " + recentInsns[i][0][2] + '</span>' + ((recentInsns[i][1]) ? " (Delay Slot)":"");
    }
    else {
      recentInstructionHTML += "[line " + recentInsns[i][0][1] + "]: " + recentInsns[i][0][2] + ((recentInsns[i][1]) ? " (Delay Slot)":"") + "<br>";
    }
  }
  return recentInstructionHTML;
}



function initializeRegisters(program) {
  initialRegisters = new Int32Array(32);
  var initDOM = document.getElementById('registers').getElementsByClassName('init-value');
  initialRegisters[0] = 0; // zero register is hard-wired to 0
  for (var i = 1; i < 32; i++) {
    var value = parseInt(initDOM[i-1].value) >> 0;
    initialRegisters[i] = value;
  }
  program.registers = initialRegisters.slice();
}

function displayErrors() {
  var errors = interpreter.get_errors();
  if (errors.length > 0) {
    document.getElementById('errors').innerHTML = errors.join('<br>');
    document.getElementById('errors-container').style.display = "";
  }
}

function clearAlerts() {
  document.getElementById('warnings-container').innerHTML = '';
  document.getElementById('warnings-container').style.display = "none";
  document.getElementById('errors').innerHTML = '';
  document.getElementById('errors-container').style.display = "none";
}

function setFrequency(freq) {
  document.getElementById('freq').innerHTML = 'CPU: ' + freq + ' Hz <span class="caret"></span>'
  interpreter.set_frequency(freq);
}

function setup() {
  interpreter.stop_button();
  clearAlerts(); // Clear any previous warnings/errors
  document.getElementById('step').parentElement.style.display = "";
  document.getElementById('run').parentElement.style.display = "";
  document.getElementById('stop').parentElement.style.display = "none";
}



document.getElementById("run").onclick = () => {
  clearAlerts();
  document.getElementById('step').parentElement.style.display = "none";
  document.getElementById('run').parentElement.style.display = "none";
  document.getElementById('stop').parentElement.style.display = "";
  interpreter.run_button();
}

document.getElementById("step").onclick = () => interpreter.step_button();
document.getElementById("reset").onclick = () => {
  setup();
  document.getElementById('recent-instruction').innerHTML = "The most recent instructions will be shown here when stepping."
  interpreter.reset_button();
}

function stop() {
  interpreter.stop_button();
  document.getElementById('step').parentElement.style.display = "";
  document.getElementById('run').parentElement.style.display = "";
  document.getElementById('stop').parentElement.style.display = "none";
}

document.getElementById("stop").onclick = () => stop();

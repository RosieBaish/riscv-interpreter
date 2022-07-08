import { Interpreter } from "riscv-interpreter";

"use strict";

const interpreter = Interpreter.new()

setup();

document.getElementById('memory-go').addEventListener('click', displayMemory);


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

function displayMemory() {
  var memory = globalP.getMemory();
  var memoryHTML = "";
  var memAddress = parseInt(document.getElementById('memory-address').value) >>> 0;
  var addr = memAddress - (memAddress % 4); // keep memory locations as multiple of 4
  if (!memory.isValidAddress(addr)) {
    memoryHTML = '<tr class="danger"><td>Invalid memory location: ' + sprintf("0x%x", memAddress) + '. Memory addresses must be from 0x00000000 - 0x7fffffff</td><td></td><td></td><td></td></tr>';
  }
  else {
    var start = Math.max(addr - 40, 0x00000000);
    var end = Math.min(addr + 40, 0x80000000);
    for (var i = start; i < end; i += 4) {
      var lsb = memory.getMem(i);
      var byte2 = memory.getMem(i+1) << 8;
      var byte3 = memory.getMem(i+2) << 16;
      var msb = memory.getMem(i+3) << 24;
      var memValue = msb | byte3 | byte2 | lsb;
      memoryHTML += (memValue == 0) ? '<tr>' : '<tr class="info">';
      memoryHTML += sprintf("<td>0x%08x</td><td>%d</td><td>0x%08x</td><td>0b%032s</td>",
                            i, memValue, memValue >>> 0, (memValue >>> 0).toString(2));
      memoryHTML += '</tr>';
    }
  }
  if (memoryHTML != document.getElementById('memory').innerHTML) {
    document.getElementById('memory').innerHTML = memoryHTML;
  }
}
function displayRegisters() {
  var registers = interpreter.get_registers();
  var registersHTML = sprintf('<tr><td>%d</td><td>$%d (%s)</td><td>%d</td><td>0x%08x</td><td>0b%032s</tr>',
			      registers[0], 0, getRegisterSyntacticSugar(0), registers[0], registers[0] >>> 0, (registers[i] >>> 0).toString(2));
  for (var i = 1; i < 32; ++i) {
    registersHTML += sprintf('<tr><td><input class="init-value" type="text" value="%d"></td><td>$%d (%s)</td><td>%d</td><td>0x%08x</td><td>0b%032s</tr>',
			     initialRegisters[i], i, getRegisterSyntacticSugar(i), registers[i], registers[i] >>> 0, (registers[i] >>> 0).toString(2));
  }
  if (registersHTML != document.getElementById('registers').innerHTML) {
    document.getElementById('registers').innerHTML = registersHTML;
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

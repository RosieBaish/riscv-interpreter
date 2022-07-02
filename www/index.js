import { Interpreter } from "riscv-interpreter";

const interpreter = Interpreter.new()
console.log(interpreter);



document.getElementById("run").onclick = () => interpreter.run_button();
document.getElementById("step").onclick = () => interpreter.step_button();
document.getElementById("reset").onclick = () => interpreter.reset_button();
document.getElementById("stop").onclick = () => interpreter.stop_button();

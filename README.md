# RISC-V Interpreter

An interpreter for RISC-V, with intention to extend it to CHERI-RISC-V later.

## Contributers
Interpreter written by [Rosie Baish](https://github.com/RosieBaish)

UI Created by [Danny Qiu](https://github.com/dannyqiu) for his [MIPS Interpreter](https://dannyqiu.me/mips-interpreter/). He very kindly licenced the [source code](https://github.com/dannyqiu/mips-interpreter) for me to use on this project.

The interpreter is written in Rust, and then compiled into WebAssembly.
It uses the [Rust + WAsm template](https://github.com/rustwasm/wasm-pack-template) and [create-wasm-app template](https://github.com/rustwasm/create-wasm-app) to tie everything together, based on [this tutorial](https://rustwasm.github.io/docs/book/introduction.html).

## Licencing

- The UI is under the MIT licence.
- The licencing on the wasm-pack-template repo is not 100% clear. An open PR implies that it's MIT or Apache, but this is unconfirmed.
- create-wasm-app is dual licenced MIT or Apache.
- All of my code is under AGPL-3.0-or-later.

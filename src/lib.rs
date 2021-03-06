mod build_common;
mod codegen;
mod instruction;
use instruction::*;
#[macro_use]
mod interface;
mod interpreter;
mod rv64_i;
mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

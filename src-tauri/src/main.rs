// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// Kept conditional so debug builds retain a console - the deep link registration
// in lib.rs reports failures on stderr and is itself debug-only on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    sage_lib::run();
}

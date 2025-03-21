use std::{io::Write, process};

use import_length_overflow::import_length_overflow;
use infinite_loop_wasi::infinite_loop_wasi;
use price_feed_tally::price_feed_tally;
use seda_sdk_rs::{oracle_program, Process};

mod call_result_write_0;
mod cannot_spam_call_result_write;
mod import_length_overflow;
mod infinite_loop_wasi;
mod price_feed_tally;

#[oracle_program]
impl TestVmOracleProgram {
    fn tally() {
        let inputs = String::from_utf8(Process::get_inputs()).unwrap();

        match inputs.as_str() {
            "call_result_write_0" => call_result_write_0::call_result_write_0(),
            "cannot_spam_call_result_write" => cannot_spam_call_result_write::cannot_spam_call_result_write(),
            "hello_world" => {
                println!("Foo");
                eprintln!("Bar");
            }
            "import_length_overflow" => import_length_overflow().unwrap(),
            "infinite_loop_wasi" => infinite_loop_wasi(),
            "long_stdout_stderr" => {
                println!("{}", "Hello, World!\n".repeat(1_000));
                eprintln!("{}", "I AM ERROR\n".repeat(1_000));
            }
            "price_feed_tally" => price_feed_tally().unwrap(),
            "stderr_non_utf8" => {
                let non_utf8 = b"\xff";
                std::io::stderr().write_all(non_utf8).unwrap();
            }
            "stdout_non_utf8" => {
                let non_utf8 = b"\xff";
                std::io::stdout().write_all(non_utf8).unwrap();
            }
            _ => process::exit(1),
        }
    }

    fn execute() {}
}

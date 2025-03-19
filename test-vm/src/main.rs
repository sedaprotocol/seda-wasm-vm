use std::process;

use import_length_overflow::import_length_overflow;
use infinite_loop_wasi::infinite_loop_wasi;
use price_feed_tally::price_feed_tally;
use seda_sdk_rs::{oracle_program, Process};

mod import_length_overflow;
mod infinite_loop_wasi;
mod price_feed_tally;

#[oracle_program]
impl TestVmOracleProgram {
    fn tally() {
        let inputs = String::from_utf8(Process::get_inputs()).unwrap();

        match inputs.as_str() {
            "import_length_overflow" => import_length_overflow().unwrap(),
            "infinite_loop_wasi" => infinite_loop_wasi(),
            "price_feed_tally" => price_feed_tally().unwrap(),
            "hello_world" => {
                println!("Foo");
                eprintln!("Bar");
            }
            _ => process::exit(1),
        }
    }

    fn execute() {}
}

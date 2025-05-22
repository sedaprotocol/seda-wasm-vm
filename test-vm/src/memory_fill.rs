use seda_sdk_rs::{log, oracle_program, Process};

const MAX_MEMORY_ALLOCATION: usize = 10 * 1024 * 1024; // 10MB in bytes

pub fn memory_fill(is_prealloc: bool) {
    let bytes_to_allocate = 44832551;

    let mut memory = if is_prealloc {
        vec![0u8; bytes_to_allocate]
    } else {
        Vec::new()
    };

    // Simulate some processing
    for i in 0..bytes_to_allocate {
        if is_prealloc {
            memory[i] = (i % 256) as u8; // Fill with some data
        } else {
            log!("Extending memory iteration: {}", i);
            memory.extend(std::iter::repeat((i % 256) as u8).take(500 * 1024));
        }
    }

    if bytes_to_allocate > MAX_MEMORY_ALLOCATION {
        Process::exit_with_message(1, "This should never be reached");
    }

    Process::exit_with_message(0, "Successfully allocated memory");
}

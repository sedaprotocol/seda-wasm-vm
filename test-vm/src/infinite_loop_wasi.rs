pub fn infinite_loop_wasi() {
    let mut a = 10;
    loop {
        // println calls fd_write which is the cheapest WASI call
        // We need to make sure the operation does not take too long and open up to attacks
        println!("{}", a);
        a = a + 1;
    }
}

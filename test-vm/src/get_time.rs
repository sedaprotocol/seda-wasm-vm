pub fn get_time() {
    let now = std::time::Instant::now();
    let current_time = std::time::SystemTime::now();

    println!("Time: {:?}", now.elapsed());
    println!("Current time: {:?}", current_time);
}

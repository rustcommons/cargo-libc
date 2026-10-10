fn main() {
    println!(
        "{}; {}",
        hello_macro::message!(),
        env!("HELLO_BUILD_SCRIPT")
    );
    let result = std::thread::spawn(|| 6 * 7)
        .join()
        .expect("worker panicked");
    assert_eq!(result, 42);
    println!("target executable ran; thread result = {result}");
}

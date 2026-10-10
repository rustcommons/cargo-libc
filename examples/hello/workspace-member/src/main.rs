fn main() {
    let result = std::thread::spawn(|| 42).join().unwrap();
    assert_eq!(result, 42);
    println!("workspace member ran");
}

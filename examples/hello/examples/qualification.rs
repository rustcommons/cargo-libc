fn main() {
    assert!(cfg!(feature = "qualification"));
    assert!(!cfg!(debug_assertions));
    println!("qualification example ran");
}

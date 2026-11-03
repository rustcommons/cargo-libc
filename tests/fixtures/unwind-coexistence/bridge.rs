// Compile as cdylib. Callbacks that can unwind have C-unwind on BOTH ends.
use std::{panic::catch_unwind, sync::atomic::{AtomicUsize, Ordering}};
type Callback = unsafe extern "C-unwind" fn();
type Through = unsafe extern "C-unwind" fn(Callback);
static DROPS: AtomicUsize = AtomicUsize::new(0);
struct Guard;
impl Drop for Guard { fn drop(&mut self) { DROPS.fetch_add(1, Ordering::SeqCst); } }
#[no_mangle] pub extern "C" fn internal_catch() -> i32 {
    let start = DROPS.load(Ordering::SeqCst);
    let caught = catch_unwind(|| { let _g = Guard; panic!("dso internal"); }).is_err();
    i32::from(caught && DROPS.load(Ordering::SeqCst) == start + 1)
}
#[no_mangle] pub unsafe extern "C-unwind" fn bridge(callback: Callback) {
    let _g = Guard;
    unsafe { callback() }
}
unsafe extern "C-unwind" fn origin_panic() { let _g = Guard; panic!("dso-origin-payload"); }
#[no_mangle] pub unsafe extern "C" fn origin_roundtrip(through_executable: Through) -> i32 {
    // The DSO's own panic returns to a catch in the same DSO runtime.
    let start = DROPS.load(Ordering::SeqCst);
    match catch_unwind(|| unsafe { through_executable(origin_panic) }) {
        Err(error) => {
            let matched = error.downcast_ref::<&str>() == Some(&"dso-origin-payload");
            drop(error);
            i32::from(matched && DROPS.load(Ordering::SeqCst) == start + 1)
        }
        Ok(()) => 0,
    }
}
#[no_mangle] pub extern "C" fn drops() -> usize { DROPS.load(Ordering::SeqCst) }

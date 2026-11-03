use std::{ffi::{CString, CStr, c_void, c_char, c_int}, panic::catch_unwind,
          sync::atomic::{AtomicUsize, Ordering}};
type Callback = unsafe extern "C-unwind" fn();
type Bridge = unsafe extern "C-unwind" fn(Callback);
type Internal = unsafe extern "C" fn() -> i32;
type Roundtrip = unsafe extern "C" fn(Bridge) -> i32;
type Drops = unsafe extern "C" fn() -> usize;
#[link(name = "dl")]
extern "C" {
    fn dlopen(path: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
    fn dlerror() -> *const c_char;
}
static DROPS: AtomicUsize = AtomicUsize::new(0);
struct Guard;
impl Drop for Guard { fn drop(&mut self) { DROPS.fetch_add(1, Ordering::SeqCst); } }
unsafe extern "C-unwind" fn executable_panic() {
    let _g = Guard;
    panic!("executable-origin-payload");
}
unsafe extern "C-unwind" fn executable_bridge(callback: Callback) {
    let _g = Guard;
    unsafe { callback() }
}
unsafe fn symbol(handle: *mut c_void, name: &str) -> *mut c_void {
    let name = CString::new(name).unwrap();
    unsafe { dlerror(); }
    let p = unsafe { dlsym(handle, name.as_ptr()) };
    let err = unsafe { dlerror() };
    if !err.is_null() { panic!("dlsym: {}", unsafe { CStr::from_ptr(err) }.to_string_lossy()); }
    assert!(!p.is_null()); p
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert_eq!(args.len(), 4, "host DSO u07|u10|u11 local|global");
    let flags = match args[3].as_str() { "local" => 2, "global" => 2 | 0x100, _ => panic!("scope") };
    let path = CString::new(args[1].as_str()).unwrap();
    let handle = unsafe { dlopen(path.as_ptr(), flags) };
    if handle.is_null() {
        let err = unsafe { dlerror() };
        panic!("dlopen: {}", if err.is_null() { "unknown".into() } else { unsafe { CStr::from_ptr(err) }.to_string_lossy() });
    }
    let internal: Internal = unsafe { std::mem::transmute(symbol(handle, "internal_catch")) };
    let bridge: Bridge = unsafe { std::mem::transmute(symbol(handle, "bridge")) };
    let roundtrip: Roundtrip = unsafe { std::mem::transmute(symbol(handle, "origin_roundtrip")) };
    let dso_drops: Drops = unsafe { std::mem::transmute(symbol(handle, "drops")) };
    let before = unsafe { dso_drops() };
    match args[2].as_str() {
        "u07" => {
            assert_eq!(unsafe { internal() }, 1);
            assert_eq!(unsafe { dso_drops() }, before + 1);
            assert_eq!(DROPS.load(Ordering::SeqCst), 0);
        }
        "u10" => {
            let result = catch_unwind(|| unsafe { bridge(executable_panic) });
            let error = result.expect_err("executable-origin panic must reach executable catch");
            assert_eq!(*error.downcast_ref::<&str>().unwrap(), "executable-origin-payload");
            drop(error); // destroyed by originating runtime before returning/ending process
            assert_eq!(unsafe { dso_drops() }, before + 1);
            assert_eq!(DROPS.load(Ordering::SeqCst), 1);
        }
        "u11" => {
            assert_eq!(unsafe { roundtrip(executable_bridge) }, 1);
            assert_eq!(unsafe { dso_drops() }, before + 1);
            assert_eq!(DROPS.load(Ordering::SeqCst), 1);
        }
        _ => panic!("case"),
    }
    // Confirm runtime remains usable; ordinary C entry contains its own panic.
    let after = unsafe { dso_drops() };
    assert_eq!(unsafe { internal() }, 1);
    assert_eq!(unsafe { dso_drops() }, after + 1);
    println!("PASS {} {} exe_drops={} dso_drops={}", args[2], args[3],
             DROPS.load(Ordering::SeqCst), unsafe { dso_drops() });
    // Intentionally no dlclose: no plugin unloading claim in this focused kit.
}

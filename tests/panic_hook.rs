//! A custom panic hook on a thread outside any async runtime: what it tracked is still sent before the hook returns.
//! (A test binary of its own: the panic hook is set once per process.)

#[path = "../src/test_server.rs"]
mod test_server;

use aptabase_rs::{Builder, InitOptions};

// A thread outside any runtime has no reactor to send on: flushing there used to panic inside the panic hook, which
// aborts the process.
#[test]
fn a_panic_outside_the_runtime_sends_what_the_hook_tracked() {
    let server = test_server::server(&[200]);
    let _client = Builder::new("A-SH-0000000000", "1.0.0")
        .with_options(InitOptions {
            host: Some(server.base.clone()),
            ..Default::default()
        })
        .with_default_panic_hook()
        .build();

    let panicked = std::thread::spawn(|| panic!("boom")).join();
    assert!(panicked.is_err());

    let requests = server.requests.lock().unwrap();
    assert_eq!(requests.len(), 1, "delivered before the hook returned");
    assert_eq!(requests[0].1[0]["eventName"], "panic");
}

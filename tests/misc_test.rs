extern crate base64;
extern crate jets;

use jets::utils::GUIDGenerator;
use std::sync::Arc;
use std::thread;

#[test]
fn test_guid() {
    let gen = Arc::new(GUIDGenerator::default());
    let gen2 = gen.clone();
    thread::spawn(move || {
        println!("------> next: {}", gen2.next_b64());
    });
    for _ in 0..20 {
        thread::sleep_ms(100);
        println!("next: {}", gen.next_b64());
    }
}

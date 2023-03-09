use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

fn reader(lock: Arc<RwLock<()>>) {
    // loop forever, holding a read lock for most of the time
    loop {
        let delay = Duration::from_millis(333);
        let start = Instant::now();
        let _guard = lock.read().expect("Poisoned read lock!");
        let read_delay = start.elapsed();
        std::thread::sleep(delay);
        println!("Reader waited {} ms for a read lock, held it {} ms",
            read_delay.as_millis(), delay.as_millis());
    }

}

fn main() {
    println!("Hello, world!");

    let lock = Arc::new(RwLock::new(()));
    let rlock = lock.clone();
    let r = std::thread::spawn(move || reader(rlock));
    r.join().unwrap();
}

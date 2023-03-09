/// Test for write-starvation with RwLock

use rand::Rng;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

fn reader(lock: Arc<RwLock<()>>) {
    // loop forever, holding a read lock for most of the time
    let id = thread::current().id();
    let mut rng = rand::thread_rng();
    let dist = rand::distributions::Uniform::new(10,1000);
    loop {
        let delay = Duration::from_millis(rng.sample(dist));
        let start = Instant::now();
        let _guard = lock.read().expect("Poisoned read lock!");
        let read_delay = start.elapsed();
        thread::sleep(delay);
        println!("Reader waited {} μs for a read lock, held it {} ms {:?}",
            read_delay.as_micros(), delay.as_millis(), id);
    }
}

fn main() {
    println!("Hello, world!");

    // Create a lock
    let lock = Arc::new(RwLock::new(()));
    // Pass it to a number of read threads
    let count = match thread::available_parallelism() {
        Ok(value) => value.get(),
        Err(_) => 4,
    };
    let readers: Vec<thread::JoinHandle<_>> = (0..count).into_iter().map(|_| {
        let rlock = lock.clone();
        thread::spawn(move || reader(rlock))
    }).collect();

    // Wait for threads to return
    for r in readers.into_iter() {
        r.join().unwrap();
    }
}

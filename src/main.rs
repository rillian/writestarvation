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
        let _guard = lock.read().expect("Poisoned lock!");
        let read_delay = start.elapsed();
        thread::sleep(delay);
        println!("Reader waited {} μs for a lock, held it {} ms {:?}",
            read_delay.as_micros(), delay.as_millis(), id);
    }
}

fn writer(lock: Arc<RwLock<()>>) {
    // loop forever, asking for the write lock periodically
    let id = thread::current().id();
    let mut rng = rand::thread_rng();
    let dist = rand::distributions::Uniform::new(500, 5000);
    loop {
        let delay = Duration::from_millis(rng.sample(dist));
        let start = Instant::now();
        {
            let _guard = lock.write().expect("Poisoned lock!");
            let write_delay = start.elapsed();
            println!("Writer waited {} μs for a lock {:?}",
                write_delay.as_micros(), id);
            // lock is dropped before sleeping
        }
        thread::sleep(delay);
    }
}

fn main() {
    println!("Rust RwLock contetion test");

    // Create a lock
    let lock = Arc::new(RwLock::new(()));

    // How many threads do we want?
    let count = match thread::available_parallelism() {
        Ok(value) => value.get(),
        Err(_) => 4,
    };
    let mut threads = Vec::with_capacity(count + 1);
    println!("spawning {} threads...", threads.capacity());

    // Pass the lock to a number of read threads
    for _ in 0..count {
        let rlock = lock.clone();
        let r = thread::spawn(move || reader(rlock));
        threads.push(r);
    }

    // Pass the lock to a write thread
    let w = thread::spawn(move || writer(lock));
    threads.push(w);

    // Wait for all threads to return
    for t in threads.into_iter() {
        t.join().unwrap();
    }
}

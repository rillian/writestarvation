# Writer starvation and RwLock

The `RwLock` type from the Rust standard library is implemented
differently on different operating systems. In particular the
docs [warn](https://doc.rust-lang.org/std/sync/struct.RwLock.html)
(as of Rust 1.68.0) that a write lock request may or may not
block concurrent read lock requests. Since a write lock must
be exclusive, this means that under contention a write lock
request may *never* unblock. This is known as "writer starvation."

Prior to Rust 1.62, writer starvation was possible on Linux
hosts when the implementation just wrapped `pthread_rwlock`.
Since then, it has used a
[futex-based implementation](https://github.com/rust-lang/rust/pull/95801)
on Linux and BSDs which explicitly prioritizes writers.

This means writer contention can starve readers instead, but
usually `RwLock` is used when writer updates are rare. Otherwise
a `Mutex` or other symmetric type makes more sense.

## Measurement

This is a quick example to check if a particular toolchain and
target result in writer starvation under load. It spawns multiple
reader threads, each of which holds a read lock most of the time,
along with a single write thread that periodically tries to obtain
a lock.

Both threads report how long they had to wait for a lock. If the
writer is succeeding, it will report waiting about as long as
the readers hold their locks for. If the writer is starving,
it won't report acquiring the lock at all.

## Results

Reports so far where write locks succeed under contention:

- 🔏 Rust 1.68.0, x86_64-pc-linux-gnu, Fedora Linux 6.1.13-200.fc37
- 🔏 Rust 1.68.0, aarch64-apple-darwin, macOS Ventura 13.2

Reports with writer starvation:

- 🙅 Rust 1.60.0, x86_64-pc-linux-gnu, Fedora Linux 6.1.13-200.fc37

## Running

Build and run the program for a bit, and look for `Writer waited` lines.

    cargo run
    [wait ~10 seconds]
    ctrl-c

Example output on x86_64-unknown-linux-gnu
```
$ cargo +1.68.0 run
[...]
Reader waited 1 μs for a lock, held it 350 ms ThreadId(4)
Reader waited 1 μs for a lock, held it 463 ms ThreadId(5)
Reader waited 1 μs for a lock, held it 951 ms ThreadId(8)
Reader waited 1 μs for a lock, held it 996 ms ThreadId(2)
Writer waited 883336 μs for a lock ThreadId(10)
Reader waited 789910 μs for a lock, held it 110 ms ThreadId(7)
Reader waited 0 μs for a lock, held it 192 ms ThreadId(2)
Reader waited 570165 μs for a lock, held it 233 ms ThreadId(5)
Reader waited 571152 μs for a lock, held it 320 ms ThreadId(4)
Reader waited 650124 μs for a lock, held it 358 ms ThreadId(3)
Reader waited 711898 μs for a lock, held it 382 ms ThreadId(6)
Reader waited 1 μs for a lock, held it 459 ms ThreadId(2)
```

Since the write lock is exclusive, it's expected that there will
be a tail of delayed read locks acquired after the write lock
is released. Unless there is contention from multiple writer
requests, this should quickly return to a few μs delay.

If the writer is starved, it's expected read lock delays are
all short, since none of them need to wait for the quick
write lock.
```
$ cargo +1.60.0 run
[...]
Reader waited 0 μs for a lock, held it 174 ms ThreadId(6)
Reader waited 1 μs for a lock, held it 177 ms ThreadId(2)
Reader waited 1 μs for a lock, held it 332 ms ThreadId(5)
Reader waited 0 μs for a lock, held it 396 ms ThreadId(9)
Reader waited 1 μs for a lock, held it 322 ms ThreadId(2)
Reader waited 2 μs for a lock, held it 176 ms ThreadId(5)
Reader waited 2 μs for a lock, held it 336 ms ThreadId(6)
Reader waited 0 μs for a lock, held it 522 ms ThreadId(3)
Reader waited 3 μs for a lock, held it 75 ms ThreadId(2)
Reader waited 3 μs for a lock, held it 190 ms ThreadId(5)
Reader waited 2 μs for a lock, held it 144 ms ThreadId(2)
```

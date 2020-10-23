# Async Rust
This is a digression into async Rust during an exploration to convert the onvm mgr into a runtime (in Rust)

## How to use
Reading through the References section is of immense value. However, for some of the crates here, it might be of some value to look around them.
- `rayon-crossbeam-demo`: shows simple recursive tasks with rayon and crossbeam scoped threads.
- `thread-demo`: shows how to use standard library threads in Rust
- `greenthreads`: contains complete working code for creating green threads in Rust
- `tokio-basic` and `testing-mio`: contains a basic tokio and mio example
- `threadpool-demo`: demonstrates Rayon threadpool. The julia set in `output.png` is amazing!
- `mio-http`: updates the [low level TCP Server](https://sergey-melnychuk.github.io/2019/08/01/rust-mio-tcp-server/#:~:text=Low-level%20TCP%20server%20in%20Rust%20with%20MIO%20Aug,low-level%20cross-platform%20abstraction%20over%20epoll%2Fkqueue%20written%20in%20Rust.) to the latest version of mio

### Note
* All code examples are either taken from the references below or from `docs.rs`
* Some code may have been slightly modified
* Comments about concept and code have been placed close together

## Disclaimer
- Some of the crates have been left in an incomplete state (generally commented out in the workspace toml)
- `greenthreads` have been commented out in the workspace toml since it needs nightly Rust. To run this crate - uncomment the greenthreads line in the workspace toml and run the following commands:
```bash
cd greenthreads
rustup override set nightly
cargo build
```

## References
### Required
[Rust Cookbook Concurrency](https://rust-lang-nursery.github.io/rust-cookbook/concurrency/threads.html)<br>
[Rayon](https://docs.rs/rayon/1.4.1/rayon/index.html)<br>
[Tokio](https://docs.rs/tokio/0.3.0/tokio/index.html)<br>
[async-std](https://book.async.rs)<br>
[Rust async await](https://rust-lang.github.io/async-book/#:~:text=This%20book%20aims%20to%20be%20a%20comprehensive%2C%20up-to-date,general%2C%20and%20to%20Rust%27s%20particular%20take%20on%20it.)<br>

### Optional to read but highly recommended
[Rust Mio](https://docs.rs/mio/0.7.4/mio/)<br>
[Greenthreads in Rust](https://cfsamson.gitbook.io/green-threads-explained-in-200-lines-of-rust/)<br>
[Node.js eventloop Rust threadpool](https://cfsamson.github.io/book-exploring-async-basics/)<br>
[IO Event Loops with Rust](https://cfsamsonbooks.gitbook.io/epoll-kqueue-iocp-explained/)<br>
[Rust Futures](https://hoverbear.org/blog/the-future-with-futures/)<br>
[Writing your own Futures in Rust](https://cfsamson.github.io/books-futures-explained/0_background_information.html)<br>
[State Machine Patterns in Rust](https://hoverbear.org/blog/rust-state-machine-pattern/)<br>
[Writing a custom OS](https://os.phil-opp.com)<br>
[Low level TCP server with Rust Mio](https://sergey-melnychuk.github.io/2019/08/01/rust-mio-tcp-server/#:~:text=Low-level%20TCP%20server%20in%20Rust%20with%20MIO%20Aug,low-level%20cross-platform%20abstraction%20over%20epoll%2Fkqueue%20written%20in%20Rust.)
use tokio;

/// tokio can run many tasks concurrently by repeatedly swapping out currently running tasks on each thread
/// However, this can only happen at .await points. So code that takes long in reaching .await points will prevent other tasks from running.
/// In order to prevent this, tokio provides two kinds of threads:
/// 1. A Core thread on each core. This is where all asynchronous code runs. Tokio, by default, spawns one for each thread.
/// 2. The blocking threads are spawned on demand, and can be used to run blocking code that would otherwise block other tasks from running. The upper limit on the number of blocking threads is very large and can be configured in the Builder

/// tokio::main provides only basic configuration options
/// tokio::runtime with crate level {feature="rt"} or {feature="rt-multi-thread"} is a more flexible alternative
#[tokio::main]
async fn main() {
    // this is running on a core thread
    let blocking_task = tokio::task::spawn_blocking(|| {
        // it is okay to block here
    });
    // wait on the blocking task and propagate if the blocking task panics
    blocking_task.await.unwrap();
}

// For CPU bound code, it can be run on a threadpool like rayon and tokio::sync::oneshot (available only with crate level {feature="sync"}) can be used to send results back to tokio when rayon task finishes

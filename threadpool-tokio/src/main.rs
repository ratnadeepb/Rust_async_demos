use tokio::sync::oneshot;

async fn some_computation() -> String {
    "represents the result of the computation".to_string()
}

#[tokio::main]
async fn main() {
    let (tx, rx) = oneshot::channel();

    tokio::spawn(async move {
        let res = some_computation().await;
        tx.send(res).unwrap();
    });
    // Note, if the task produces a computation result as its final action before terminating, the JoinHandle can be used to receive that value instead of allocating resources for the oneshot channel. Awaiting on JoinHandle returns Result. If the task panics, the Joinhandle yields Err with the panic cause.
    let join_handle = tokio::spawn(async move { some_computation().await });
    // wait for the computation result
    let res = rx.await.unwrap();
    println!("oneshot channel: {}", res);

    // Wait for the computation result
    let res2 = join_handle.await.unwrap();
    println!("join handle: {}", res2);
}
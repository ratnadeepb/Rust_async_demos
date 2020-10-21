use futures::executor::block_on;

async fn hello_world() {
    println!("Hello world!");
}

async fn async_main() {
    // let future = hello_world();
    println!("Future created!");
    // block_on(future);
    // println!("Future completed!");
    let future2 = hello_world().await;
    futures::join!(hello_world());
    println!("Future 2 completed!");
}

fn main() {
    block_on(async_main());
}

use std::{sync::mpsc, thread, time::Duration};

fn main() {
    let (tx, rx) = mpsc::channel();
    let tx1 = mpsc::Sender::clone(&tx);
    let v = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let v1 = v.clone();
    // move v into the spawned thread
    let handle = thread::spawn(move || {
        for i in 1..10 {
            tx.send(v[i]).unwrap();
            println!("hi {} from spawned thread", v[i]);
            thread::sleep(Duration::from_millis(1));
        }
    });

    let handle1 = thread::spawn(move || {
        for i in 1..10 {
            tx1.send(v1[i] + 10).unwrap();
            println!("hi {} from spawned thread", v1[i] + 10);
            thread::sleep(Duration::from_millis(1));
        }
    });

    for i in 1..5 {
        println!("hi {} from main thread", i);
        thread::sleep(Duration::from_millis(1));
    }

    // let mut recv_iter = rx.iter();
    // while let Some(msg) = recv_iter.next() {
    //     println!("Got: {:?}", msg);
    // }
    for recvd in rx {
        println!("Got: {:?}", recvd);
    }

    handle.join().unwrap(); // block the main thread
}

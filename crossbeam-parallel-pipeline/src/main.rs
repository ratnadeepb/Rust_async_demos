use crossbeam;
use crossbeam_channel::bounded;
use std::{thread, time};

fn main() {
    // a channel with a buffer of size 1
    let (snd1, rcv1) = bounded(1);
    let (snd2, rcv2) = bounded(1);
    let n_msgs = 4;
    let n_workers = 2;

    crossbeam::scope(|s| {
        // producer thread
        s.spawn(|_| {
            for i in 0..n_msgs {
                // the send call blocks for half a second because the receiver threads sleep for half a second
                snd1.send(i).unwrap();
                println!("source sent {}", i);
            }
            drop(snd1); // close the channel
        });
        // parallel processing by two threads
        for _ in 0..n_workers {
            // send to sink, receive from source
            let (sndr, rcvr) = (snd2.clone(), rcv1.clone());
            // spawn workers in separate threads
            s.spawn(move |_| {
                // each thread sleeps for half a second before processing a message
                thread::sleep(time::Duration::from_millis(500));
                // receive until channel closes
                for msg in rcvr.iter() {
                    println!("Worker {:?} received {}", thread::current().id(), msg);
                    sndr.send(msg * 2).unwrap();
                }
            });
        }
        // close channel, otherwise sink will not exit the for-loop
        drop(snd2);
        // sink
        for msg in rcv2.iter() {
            println!("Sink received {}", msg);
        }
    })
    .unwrap();
}

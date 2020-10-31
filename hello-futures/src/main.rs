use futures;
use libc;
use std::sync::mpsc::channel;
use std::{
    mem,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

fn main() {
    let thread1 = thread::Builder::new()
        .name("thread1".to_string())
        .spawn(|| {
            println!("thread builder");

            let (tx_m, rx_m) = channel();
            let flag_m = Arc::new(AtomicBool::new(false));
            let flag_master = Arc::clone(&flag_m);

            let (tx_c, rx_c) = channel();
            let flag_c = Arc::new(AtomicBool::new(false));
            let flag_core = Arc::clone(&flag_c);

            let master_executor = thread::spawn(move || unsafe {
                let mut set: libc::cpu_set_t = mem::zeroed();
                libc::CPU_SET(0_usize, &mut set);
                let master_future = async move {
                    println!("Master Future Completed");
                };
                tx_m.send(true).unwrap();
                drop(tx_m);
                while !flag_m.load(Ordering::Acquire) {
                    println!("Parking Master Executor");
                    thread::park();
                    println!("Unparking Master Executor");
                }
                futures::executor::block_on(master_future);
            });

            let core_executor = thread::spawn(move || unsafe {
                let mut set: libc::cpu_set_t = mem::zeroed();
                libc::CPU_SET(1_usize, &mut set);
                let core_future = async move {
                    println!("Core Future Completed");
                };
                tx_c.send(true).unwrap();
                drop(tx_c);
                while !flag_c.load(Ordering::Acquire) {
                    println!("Parking Core Executor");
                    thread::park();
                    println!("Unparking Core Executor");
                }
                futures::executor::block_on(core_future);
            });

            let m = rx_m.recv().unwrap();
            let c = rx_c.recv().unwrap();
            if m && c {
                println!("Unparking the executors");
                flag_master.store(true, Ordering::Release);
                flag_core.store(true, Ordering::Release);
            }
            println!("Both threads are now ready");

            master_executor.thread().unpark();
            core_executor.thread().unpark();

            master_executor.join().unwrap();
            core_executor.join().unwrap();
        })
        .unwrap();

    thread1.join().unwrap();
}

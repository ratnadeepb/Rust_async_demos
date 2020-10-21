use minimio;
use std::{
    collections::{BTreeMap, HashMap},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

static mut RUNTIME: *mut Runtime = std::ptr::null_mut();

pub struct Runtime {
    // available threads
    available_threads: Vec<usize>,
    // callbacks scheduled to run
    callbacks_to_run: Vec<(usize, Js)>,
    // all registered callbacks
    callback_queue: HashMap<usize, Box<dyn FnOnce()>>,
    // number of pending poll events, only used to print
    epoll_pending_events: usize,
    // register for events of interet with the OS
    epoll_registrator: minimio::Registrator,
    // The handle to our epoll thread
    epoll_thread: thread::JoinHandle<()>,
    // None = infinite, Some(n) = timeout in n ms, Some(0) = immediate
    epoll_timeout: Arc<Mutex<Option<i32>>>,
    // Channel used by both our threadpool and our epoll thread to send events to the main loop
    event_reciever: Receiver<PollEvent>,
    // Creates an unique identity for our callbacks
    identity_token: usize,
    // The number of events pending. When this is zero, we're done
    pending_events: usize,
    // Handles to our threads in the threadpool
    thread_pool: Vec<NodeThread>,
    // Holds all our timers, and an Id for the callback to run once they expire
    timers: BTreeMap<Instant, usize>,
    // A struct to temporarely hold timers to remove. We let Runtinme have ownership so we can reuse the same memory
    timers_to_remove: Vec<Instant>,
}

impl Runtime {
    pub fn new() -> Self {
        // ===== THE REGULAR THREADPOOL =====
        let (event_sender, event_receiver) = channel::<PollEvent>();
        let mut threads = Vec::with_capacity(4);

        for i in 0..4 {
            let (evt_sender, evt_receiver) = channel::<Task>();
            let event_sernder = event_sender.clone();

            let handle = thread::Builder::new()
            .name(format!("Pool{}", i))
            .spawn(move || {

                while Ok(task) = evt_receiver.recv() {
                    print(format!("received a task of type: {}", task.kind));

                    if let ThreadPoolTaskKind::Close == task.kind {
                        break;
                    };

                    let res = (task.task)();
                    print(format!("finished running a task of type: {}.", task.kind));

                    let event = PollEvent::Threadpool((i, task.callback_id, res));
                    event_sender.send(event).expect("threadpool");
                }
            })
            .expect("Couldn't initialize thread pool.");
            
            let node_thread = NodeThread {
                handle,
                sender: evt_sender,
            }
            threads.push(node_thread);
        }

        // ===== EPOLL THREAD =====
        let mut poll = minimio::Poll::new().expect("Error creating epoll queue");
        let registrator = poll.registrator();
        let epoll_timeout = Arc::new(Mutex::new(None));
        let epoll_timeout_clone = epoll_timeout.clone();

        let epoll_thread = thread::Builder::new()
        .name("epoll".to_string())
        .spawn(move || {
            let mut events = minimio::Events::with_capacity(1024);

            loop {
                let epoll_timeout_handle = epoll_timeout_clone.lock().unwrap();
                let timeout = *epoll_timeout_handle;
                drop(epoll_timeout_handle);

                match poll.poll(&mut events, timeout) {
                    Ok(v) if v > 0 => {
                        for i in 0..v {
                            let event = events.get_mut(i).expect("No events in event list.");
                            println!(format!("epoll event {} is ready", event.id().value()));

                            let event = PollEvent::Epoll(event.id().value() as usize);
                            event_sender.send(event).expect("epoll event");
                        }
                    }
                    Ok(v) if v == 0 {
                        println!("epoll event timeout is ready");
                        
                    }
                }
            }
        })
    }

    // The run function on our Runtime will consume self so it's the last thing that we'll be able to call on this instance of our Runtime
    pub fn run(mut self, f: impl Fn()) {
        let rt_ptr: *mut Runtime = &mut self;
        unsafe { RUNTIME = rt_ptr };
        let mut ticks = 0;

        // run the main function
        f();

        // ====== Event Loop ======
        while self.pending_events > 0 {
            ticks += 1;
            // NOT PART OF LOOP, JUST FOR US TO SEE WHAT TICK IS EXCECUTING
            print(format!("===== TICK {} =====", ticks));

            // ====== 2. TIMERS ======
            // check if any of the timers have expired
            self.process_expired_timers();
            // ====== 3. CALLBACKS ======
            // if a timer has expired then run the associated callback
            self.run_callbacks();

            // ====== 4. IDLE/PREPARE ======
            // we won't use this

            // ====== 5. POLL ======
            // if we don't have any outstanding events then we are finished
            if self.pending_events == 0 {
                break;
            }

            // We want to get the time to the next timeout (if any) and we
            // set the timeout of our epoll wait to the same as the timeout
            // for the next timer. If there is none, we set it to infinite (None)
            let next_timeout = self.get_next_timer();
            let mut epoll_timeout_lock = self.epoll_timeout.lock().unwrap();
            *epoll_timeout_lock = next_timeout;
            drop(epoll_timeout_lock); // release the lock

            // we handle one event but multiple events could be returned
            // on the same poll. We won't cover that here though but there are
            // several ways of handling this.
            // both threadpool threads and the epoll thread hold a sending part of the channel
            if let Ok(event) = self.event_reciever.recv() {
                match event {
                    PollEvent::Timeout => (),
                    PollEvent::Threadpool(thread_id, callback_id, data) => {
                        self.process_threadpool_events(thread_id, callback_id, data);
                    }
                    PollEvent::Epoll(event_id) => {
                        self.process_epoll_events(event_id);
                    }
                }
            }
            self.run_callbacks();

            // ====== 6. CHECK ======
            // an set immidiate function could be added pretty easily but we
            // won't do that here

            // ====== 7. CLOSE CALLBACKS ======
            // Release resources, we won't do that here, but this is typically
            // where sockets etc are closed.
        }

        // clean up resources, make sure all destructors run
        for thread in self.thread_pool.into_iter() {
            thread
                .sender
                .send(Task::close())
                .expect("threadpool cleanup");
            thread.handle.join().unwrap();
        }

        self.epoll_registrator.close_loop().unwrap();
        self.epoll_thread.join().unwrap();
    }
}

struct Task {
    task: Box<dyn Fn() -> Js + Send + 'static>,
    callback_id: usize,
    kind: ThreadPoolTaskKind,
}

impl Task {
    fn close() -> Self {
        Self {
            task: Box::new(|| Js::Undefined),
            callback_id: 0,
            kind: ThreadPoolTaskKind::Close,
        }
    }
}

// a thread in our threadpool
#[derive(Debug)]
struct NodeThread {
    pub(crate) handle: thread::JoinHandle<()>, // returned from thread::spawn
    sender: Sender<Task>,                      // sending part of the channel
}

// three kinds of events in our example:
// 1. 'FileRead' - file that has been read
// 2. 'Encrypt' - represents an operation from the Crypto module
// 3. 'Close' - indicating that the thread should be closed
pub enum ThreadPoolTaskKind {
    FileRead,
    Encrypt,
    Close,
}

// The Js object is simply there to make our code look more JavaScripty
// and to abstract over the return type of closures
#[derive(Debug)]
pub enum Js {
    Undefined,
    String(String),
    Int(usize),
}

impl Js {
    // convenience methods since we know the types
    fn into_string(self) -> Option<String> {
        match self {
            Js::String(s) => Some(s),
            _ => None,
        }
    }

    fn into_int(self) -> Option<usize> {
        match self {
            Js::Int(n) => Some(n),
            _ => None,
        }
    }
}

// events that we can accept from the threadpool and the event queue
// describes the three main events our epoll-eventloop handles
enum PollEvent {
    // An event from the `threadpool` with a tuple containing the `thread id`,
    // the `callback_id` and the data which the we expect to process in our
    // callback
    Threadpool((usize, usize, Js)),
    // An event from the epoll-based eventloop holding the `event_id` for the
    // event
    Epoll(usize),
    Timeout,
}

fn main() {
    println!("Hello, world!");
}

#![feature(llvm_asm)]
#![feature(naked_functions)]
use std::{io::Write, ptr};

// a small stack size so this can be printed
// There seems to be an issue in OSX using such a small stack.
// The minimum for this code to run is a stack size of 624 bytes
const SSIZE: isize = 1024;
static mut S_PTR: *const u8 = 0 as *const u8;
const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 2;
const MAX_THREADS: usize = 4;
static mut RUNTIME: usize = 0;

// A small and simple runtime to schedule and switch between our threads
pub struct Runtime {
    threads: Vec<Thread>,
    current: usize, // which thread is currently running
}

impl Runtime {
    pub fn new() -> Self {
        // base thread initialised in Running state
        let base_thread = Thread {
            id: 0,
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Running,
        };
        let mut threads = vec![base_thread];
        let mut available_threads = (1..MAX_THREADS).map(|i| Thread::new(i)).collect();
        threads.append(&mut available_threads);

        Runtime {
            threads,
            current: 0,
        }
    }

    /// This is cheating a bit, but we need a pointer to our Runtime
    /// stored so we can call yield on it even if we don't have a
    /// reference to it.
    pub fn init(&self) {
        unsafe {
            let r_ptr: *const Runtime = self;
            RUNTIME = r_ptr as usize;
        }
    }

    pub fn run(&mut self) -> ! {
        while self.t_yield() {}
        std::process::exit(0);
    }

    fn t_return(&mut self) {
        if self.current != 0 {
            self.threads[self.current].state = State::Available;
            self.t_yield();
        }
    }

    fn t_yield(&mut self) -> bool {
        let mut pos = self.current;
        while self.threads[pos].state != State::Ready {
            // pos += 1;
            pos = (pos + 1) % self.threads.len();

            // if pos == self.threads.len() {
            //     pos = 0;
            // }
            if pos == self.current {
                return false;
            }
        }
        if self.threads[self.current].state != State::Available {
            self.threads[self.current].state = State::Ready;
        }

        self.threads[pos].state = State::Running;
        let old_pos = self.current;
        self.current = pos;
        unsafe {
            switch(&mut self.threads[old_pos].ctx, &self.threads[pos].ctx);
        }
        // Prevents compiler from optimizing our code away on Windows.
        self.threads.len() > 0
    }

    pub fn spawn(&mut self, f: fn()) {
        let available = self
            .threads
            .iter_mut()
            .find(|t| t.state == State::Available)
            .expect("no available threads");
        let size = available.stack.len();
        unsafe {
            let s_ptr = available.stack.as_mut_ptr().offset(size as isize);
            let s_ptr = (s_ptr as usize & !15) as *mut u8;
            // guard gets called when the thread finishes
            ptr::write(s_ptr.offset(-16) as *mut u64, guard as u64);
            // skip does nothing but is there to ensure that f and guard are on 16 byte boundaries
            ptr::write(s_ptr.offset(-24) as *mut u64, skip as u64);
            // this is the funnction that will run
            ptr::write(s_ptr.offset(-32) as *mut u64, f as u64);
            available.ctx.rsp = s_ptr.offset(-32) as u64;
        }
        available.state = State::Ready;
    }
}

#[naked] // don't add Rust prologue or epilogue to Rust function
fn skip() {}

// Out function is done and should now return
fn guard() {
    unsafe {
        let r_ptr = RUNTIME as *mut Runtime;
        (*r_ptr).t_return();
    }
}

pub fn yield_thread() {
    unsafe {
        let rt_ptr = RUNTIME as *mut Runtime;
        (*rt_ptr).t_yield();
    }
}

#[naked]
#[inline(never)]
unsafe fn switch(old: *mut ThreadContext, new: *const ThreadContext) {
    llvm_asm!(
        "
        mov     %rsp, 0x00($0)
        mov     %r15, 0x08($0)
        mov     %r14, 0x10($0)
        mov     %r13, 0x18($0)
        mov     %r12, 0x20($0)
        mov     %rbx, 0x28($0)
        mov     %rbp, 0x30($0)

        mov     0x00($1), %rsp
        mov     0x08($1), %r15
        mov     0x10($1), %r14
        mov     0x18($1), %r13
        mov     0x20($1), %r12
        mov     0x28($1), %rbx
        mov     0x30($1), %rbp
        ret
        "
    :
    :"r"(old), "r"(new)
    :
    : "volatile", "alignstack"
    );
}

#[derive(PartialEq, Eq, Debug)]
enum State {
    Available,
    Running,
    Ready,
}

struct Thread {
    id: usize,
    stack: Vec<u8>,
    ctx: ThreadContext,
    state: State,
}

impl Thread {
    fn new(id: usize) -> Self {
        Self {
            id,
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            ctx: ThreadContext::default(),
            state: State::Available,
        }
    }
}

// struct that represents the CPU state (context)
#[derive(Debug, Default)]
#[repr(C)] // C has a stable ABI, Rust doesn't
struct ThreadContext {
    rsp: u64, // stack pointer
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64, // base pointer
}

fn print_stack(filename: &str) {
    let mut f = std::fs::File::create(filename).unwrap();
    unsafe {
        for i in (0..SSIZE).rev() {
            writeln!(
                f,
                "mem: {}, val: {}",
                S_PTR.offset(i as isize) as usize,
                *S_PTR.offset(i as isize)
            )
            .expect("Error writing to file");
        }
    }
}

fn hello() -> ! {
    println!("I LOVE WAKING UP ON A NEW STACK!");
    print_stack("AFTER.txt");
    loop {}
}

// inline assembly to switch to our stack
unsafe fn gt_switch(new: *const ThreadContext) {
    // this macro checks if the content is not valid assembly
    llvm_asm!(
        // the first line moves the value stored at 0x00 offset (no offset) of the memory location at $0 to rsp
        // this essentially overwrites the value at the top of the stack with an address we provide
        // $0 is the placeholder for the first argument
        // 'ret' instrsucts the CPU to pop the address from the top of the stack and make an unconditional jump to that location
        // in effect, we get the CPU to return to our stack
        "
        mov 0x00($0), %rsp
        ret
    "
    : // output
    : "r"(new) // input; r is a constraint that tells the compiler to put it in a general purpose register
    :   // clobber list; this is where we tell the compiler which registers we manage ourselves
    : "alignstack" // options; unique to Rust inline; alignstack is for the code to run on Windows
    );
}

fn main() {
    let mut runtime = Runtime::new();
    runtime.init();
    runtime.spawn(|| {
        println!("THREAD 1 STARTING");
        let id = 1;
        for i in 0..10 {
            println!("thread: {} counter: {}", id, i);
            yield_thread();
        }
        println!("THREAD 1 FINISHED");
    });
    runtime.spawn(|| {
        println!("THREAD 2 STARTING");
        let id = 2;
        for i in 0..15 {
            println!("thread: {} counter: {}", id, i);
            yield_thread();
        }
        println!("THREAD 2 FINISHED");
    });
    runtime.run();
    // let mut ctx = ThreadContext::default();
    // in Rust, there is no guarantee that this memory is 16-byte aligned
    // let mut stack = vec![0_u8; SSIZE as usize];
    // let stack_ptr = stack.as_mut_ptr();
    // unsafe {
    // S_PTR = stack_ptr;
    // hello is a function pointer so it can be directly cast as u64
    // let stack_bottom = stack.as_mut_ptr().offset(SSIZE);
    // round the memory address down to the nearest 16-byte aligned address
    // if it is already aligned, then it does nothing
    // let sb_aligned = (stack_ptr as usize & !15) as *mut u8;
    // write the pointer to an offset of 16 bytes from the base of our stack
    // std::ptr::write(sb_aligned.offset(-16) as *mut u64, hello as u64);
    // ctx.rsp = sb_aligned.offset(-16) as u64;
    // gt_switch(&mut ctx);
    // }
}

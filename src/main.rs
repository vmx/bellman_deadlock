use crossbeam_channel::{bounded, Receiver};
use lazy_static::lazy_static;
use std::env;

lazy_static! {
    static ref NUM_CPUS: usize = if let Ok(num) = env::var("BELLMAN_NUM_CPUS") {
        if let Ok(num) = num.parse() {
            num
        } else {
            num_cpus::get()
        }
    } else {
        num_cpus::get()
    };
    pub static ref THREAD_POOL_VMX: rayon::ThreadPool = rayon::ThreadPoolBuilder::new()
        .num_threads(*NUM_CPUS)
        .build()
        .unwrap();
}

#[derive(Clone)]
pub struct Worker {}

impl Worker {
    pub fn new() -> Worker {
        Worker {}
    }

    pub fn compute<F, R>(&self, f: F) -> Waiter<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        println!("vmx: multicore worker: compute 1: {}", *NUM_CPUS);
        let (sender, receiver) = bounded(1);
        println!(
            "vmx: multicore worker: compute 2: {:?}",
            THREAD_POOL_VMX.current_thread_has_pending_tasks()
        );
        THREAD_POOL_VMX.spawn(move || {
            println!("vmx: multicore worker: compute thread spawned 1");
            let res = f();
            println!("vmx: multicore worker: compute thread spawned 2");
            sender.send(res).unwrap();
            println!("vmx: multicore worker: compute thread spawned 3");
        });

        println!(
            "vmx: multicore worker: compute 3: {:?}",
            THREAD_POOL_VMX.current_thread_has_pending_tasks()
        );
        Waiter { receiver }
    }
}

pub struct Waiter<T> {
    receiver: Receiver<T>,
}

impl<T> Waiter<T> {
    /// Wait for the result.
    pub fn wait(&self) -> T {
        println!("vmx: multicore waiter: wait 1");
        let result = self.receiver.recv().unwrap();
        println!("vmx: multicore waiter: wait 2");
        result
    }

    /// One off sending.
    pub fn done(val: T) -> Self {
        println!("vmx: multicore waiter: done 1");
        let (sender, receiver) = bounded(1);
        sender.send(val).unwrap();
        println!("vmx: multicore waiter: done 2");

        Waiter { receiver }
    }
}

fn do_something() -> String {
    "I did something".to_string()
}

fn call_compute() {
    let worker = Worker::new();
    let computed = worker.compute(move || do_something());
    let result = computed.wait();
    println!("result: {:?}", result);
}

fn main() {
    // Works
    //call_compute();

    // Fails
    THREAD_POOL_VMX.install(|| call_compute());
}

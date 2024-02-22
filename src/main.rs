use rand::{distributions::Uniform, Rng};
use rayon::prelude::*;
use std::time::{Duration, Instant};
use tracing::{info, info_span, span, trace, Instrument};

pub mod logs;

async fn blocking() {
    let blocking_span = span!(tracing::Level::INFO, "blocking");
    let _guard = blocking_span.enter();
    let start = Instant::now();

    info!("Start timer!");
    // No .await here!! This is a blocking call
    std::thread::sleep(Duration::from_secs(1));
    info!("1 seconds later...");
    info!("Elapsed time: {:?}", start.elapsed());
}

async fn sleepy_printer_blocking(timer: i32) {
    info!("Start timer {}", timer);
    std::thread::sleep(Duration::from_secs(1));
    info!("Timer {} done", timer);
}

async fn multi_blocking() {
    let multi_blocking_span = span!(tracing::Level::INFO, "multi_blocking");
    let _guard = multi_blocking_span.enter();
    let start = Instant::now();

    // All tasks are guaranteed to run on the same thread
    // We're using `tokio::join!` here, which runs all the tasks concurrently on
    // the same thread.
    tokio::join!(
        sleepy_printer_blocking(1),
        sleepy_printer_blocking(2),
        sleepy_printer_blocking(3)
    );
    info!("Elapsed time: {:?}", start.elapsed());
}

async fn sleepy_printer_async(timer: i32) {
    info!("Start timer {}", timer);
    tokio::time::sleep(Duration::from_secs(1)).await;
    info!("Timer {} done", timer);
}

async fn single_thread_async() {
    let multi_async_span = span!(tracing::Level::INFO, "single_thread_async");
    let _guard = multi_async_span.enter();
    let start = Instant::now();

    // All tasks are guaranteed to run on the same thread
    // We're using `tokio::join!` here, which runs all the tasks concurrently on
    // the same thread.
    tokio::join!(
        sleepy_printer_async(1),
        sleepy_printer_async(2),
        sleepy_printer_async(3)
    );

    info!("Elapsed time: {:?}", start.elapsed());
}

async fn multi_thread_async() {
    let multi_thread_async_span = span!(tracing::Level::INFO, "multi_thread_async");
    let _guard = multi_thread_async_span.enter();
    let start = Instant::now();

    // All tasks are guaranteed to run on different threads
    // Note that we're using `tokio::spawn` instead of `tokio::join!', which
    // spawns a new async task on a new thread.
    let ops = vec![
        tokio::spawn(sleepy_printer_async(1).instrument(multi_thread_async_span.clone())),
        tokio::spawn(sleepy_printer_async(2).instrument(multi_thread_async_span.clone())),
        tokio::spawn(sleepy_printer_async(3).instrument(multi_thread_async_span.clone())),
    ];

    for op in ops {
        op.await.unwrap();
    }

    info!("Elapsed time: {:?}", start.elapsed());
}

async fn spawn_blocking() {
    let start = Instant::now();

    // This spawns a new thread outside of the Tokio thread pool
    let blocking_task = tokio::task::spawn_blocking(move || {
        // This is running on a thread where blocking is fine
        info_span!("spawn_blocking").in_scope(|| {
            info!("Start timer!");
            std::thread::sleep(Duration::from_secs(1));
            info!("1 seconds later...");
        });
    });

    // We can wait for the blocking task like this:
    // If the blocking task panics, the unwrap will propagate the panic
    blocking_task.await.unwrap();

    info!("Elapsed time: {:?}", start.elapsed());
}

async fn single_thread_parallel_sum(nums: Vec<i32>) -> i32 {
    let (send, recv) = tokio::sync::oneshot::channel();

    // Spawn a task on rayon
    rayon::spawn(move || {
        // Sum the numbers
        let sum: i32 = nums.iter().sum();

        // Send the result back to the tokio task
        let _ = send.send(sum);
    });

    // Wait for the rayon task to finish
    recv.await.expect("Rayon::spawn task panicked!")
}

async fn multi_thread_parallel_sum(nums: Vec<i32>) -> i32 {
    let (send, recv) = tokio::sync::oneshot::channel();

    // Spawn a task on rayon
    rayon::spawn(move || {
        // Compute sum on multiple threads
        let sum: i32 = nums.par_iter().sum();

        // Send the result back to the tokio task
        let _ = send.send(sum);
    });

    // Wait for the rayon task to finish
    recv.await.expect("Rayon::spawn task panicked!")
}

async fn rayon_single_thread(nums: Vec<i32>) {
    let rayon_spawning_span = span!(tracing::Level::INFO, "rayon_single_spawning");
    let _guard = rayon_spawning_span.enter();
    let start = Instant::now();

    let sum = single_thread_parallel_sum(nums.clone()).await;
    info!("Sum: {}", sum);
    info!("Elapsed time: {:?}", start.elapsed());
}

async fn rayon_multi_thread(nums: Vec<i32>) {
    let rayon_spawning_span = span!(tracing::Level::INFO, "rayon_multi_spawning");
    let _guard = rayon_spawning_span.enter();
    let start = Instant::now();

    let sum = multi_thread_parallel_sum(nums.clone()).await;
    info!("Sum: {}", sum);
    info!("Elapsed time: {:?}", start.elapsed());
}

async fn rayon_spawning() {
    let mut rng = rand::thread_rng();
    let range = Uniform::new(0, 100);
    let nums: Vec<i32> = (0..100).map(|_| rng.sample(range)).collect();

    // Note: Rayon is not designed for async tasks. It's designed for CPU-bound
    // tasks. If you need to run async tasks in parallel, use tokio::spawn or
    // tokio::task::spawn_blocking

    rayon_single_thread(nums.clone()).await;
    rayon_multi_thread(nums.clone()).await;
}

async fn thread_spawning() {
    let start = Instant::now();

    // Spawn a dedicated thread
    let handle = std::thread::spawn(|| {
        info_span!("thread_spawning").in_scope(|| {
            info!("Start timer!");
            std::thread::sleep(Duration::from_secs(1));
            info!("1 seconds later...");
        });
    });

    // Wait for the thread to finish
    handle.join().unwrap();

    info!("Elapsed time: {:?}", start.elapsed());
}

#[tokio::main]
async fn main() {
    logs::init_tracing();
    let start = Instant::now();

    // Single Thread Blocking Example
    trace!("Blocking example");
    blocking().await;

    // Multi-Threaded Blocking Example
    println!();
    trace!("Concurrent blocking example");
    multi_blocking().await;

    // Async (non-blocking) Example
    println!();
    trace!("Single-threaded async example");
    single_thread_async().await;

    // Multi-Threaded Async Example
    println!();
    trace!("Multi-threaded async example");
    multi_thread_async().await;

    // Intentional Blocking
    // Sometimes we want to block the thread. There are two common reasons for this:
    //   1. We're running an expensive CPU-bound operation
    //   2. We're running a synchronous IO operation
    //
    // In both cases, we're dealing with an operation that prevents the task from
    // reaching the next `await` statement. To solve this, we need to move the
    // blocking operation to a thread outside of Tokio's thread pool. There are
    // three variations of this:
    //  1. `tokio::task::spawn_blocking`
    //  2. Use the rayon crate
    //  3. Spawn a dedicated thread with std::thread::spawn

    // Spawn Blocking Example
    println!();
    trace!("Spawn blocking example");
    spawn_blocking().await;

    // Rayon Spawn Example
    println!();
    trace!("Rayon spawn example");
    rayon_spawning().await;

    // Thread Spawning Example
    println!();
    trace!("Thread spawning example");
    thread_spawning().await;

    println!();
    info!("Total elapsed time: {:?}", start.elapsed());
}

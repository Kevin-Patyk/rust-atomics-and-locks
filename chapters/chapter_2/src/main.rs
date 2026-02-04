// The word atomic means indivisible, something that cannot be cut into smaller pieces
// In computer science, it is used to describe an operation that is indivisible: it is either fully completed or hasn't happened yet

// As mentioned in chapter 1, multiple threads concurrently reading and modifying the same variable normally results in undefined behavior
// However, atomic operations do allow for different threads to safely read and modify the same variables
// Since such an operation is indivisible, it either happens completely before or completely after another operation, avoiding undefined behavior
// In chapter 7, we will see how this works at the hardware level

// Atomic operations are the main building black for anything involving multiple threads
// All the other concurrency primitives, such as mutexes and condition variables, are implemented using atomic operations

// In Rust, atomic operations are available as methods on the standard atomic types that live in std::sync::atomic
// They all have names starting Atomic, such as AtomicI32 or AtomicUsize
// Which ones are available depends on the hardware architecture and sometimes operating system, but almost all platforms provide at least all atomic types up to the size of a pointer

// Unlike most types, they allow modification through a shared reference `&AtomicU8`
// This is possible thanks to interior mutability

// Each of the available atomic types has the same interface with methods for storing and laoding, methods 
// for atomic fetch and modify operations, and some more advanced "compare-and-exchange" methods
// We will discuss them in detail in the rest of this chapter

// Before diving into different atomic operations, we briefly need to touch upon a concept called memory ordering
// Every atomic operation takes an argument of type std::sync::atomic::Ordering which determines what guarantees we get about the relative ordering of operations
// The simplest variant with the fewest guarantees is `Relaxed`
// `Relaxed` still guarantees consistency on a single atomic variable, but does not promise anything about the relative order of operations between different variables

// What this means is that two threads might see operations on different variables happen in a different order
// For example, if one thread writes to one variable first and then to a second variable quickly afterwards, another thread might see that happen in the opposite order

// In this chapter, we will only look at use cases where this is not a problem and simply used `Relaxed` everywhere without going into more detail
// We will discuss memory ordering in more detail in chapter 3

use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::AtomicU64;

fn main() {
    // Atomic Load and Store Operations -----

    // The first 2 atomic operations we will look at are the most basic ones: `load` and `store`
    // The function signatures are as follows, using `AtomicI32` as an example:
        // impl AtomicI32 {
        //     pub fn load(&self, ordering: Ordering) -> i32;
        //     pub fn store(&self, value: i32, ordering: Ordering);
        // }
    // The `load`` method atomically loads the value stored in the atomic variable
    // The `store` method atomically stores a new value in it
    // Now how the `store` method takes a shared reference `&T` rather than an exclusive reference `&mut T` even though it modifies the value

    // Let's take a look at some realistic use cases for these 2 methods

    // Example: Stop Flag -----

    // The first example uses an `AtomicBool` for a stop flag
    // Such a flag is used to inform other threads to stop running

    // Creates a static variable called STOP - this variable lives for the entire program
    // This creates a variable with a 'static lifetime, meaning it exists for the entire duration of the program
    // It is initialized to false and can be safely accessed from multiple threads
    static STOP: AtomicBool = AtomicBool::new(false);

    // This spawns a brackground thread to do the work
    // It will continuously runs work in a loop, checking a global STOP flag to know when to exit
    // It will continue running while it is not stopped (STOP = false), since (!STOP = false) flips it to true, so the condition is met
    // It will stop running when it is stopped (STOP = TRUE), since (!STOP = true) flips it to false, so the condition is not met
    let background_thread = thread::spawn(|| {
        while !STOP.load(Relaxed) {
            some_work();
        }
    });

    // Use the main thread to listen for user input
    // This reads lines from stdin, processing three possible commands
    for line in std::io::stdin().lines() {
        match line.unwrap().as_str() {
            "help" => println!("commands: help, stop"),
            "stop" => break,
            cmd => println!("unknown command: {cmd:?}"),
        }
    }

    // Inform the background thread it needs to stop
    // Here, the shutdown signal is sent
    // After the input loop exists (when the user types "stop"), the main threads sets STOP flag to true signaling the background thread to stop
    STOP.store(true, Relaxed);

    // Wait until the background thread finishes
    // The main thread calls join() on the background thread handle to block until the background thread completes its work and exists cleanly 
    background_thread.join().unwrap();

    // The above is a simple concurrent pattern where one thread does background work while another handles user interaction
    // With atomic-flag-based coordination for graceful shutdown

    // The background thread is repeatedly running `some_work()` while the main thread allows the user to enter some commands to interact with the program
    // In this simple example, the only useful command is `stop` to make the program stop

    // To make the background stop, the atomic `STOP` boolean is used to communicate this condition to the background thread
    // When the foreground thread reads the `stop` command, it sets the flag to true which is checked by the background thread before each new iteration
    // The main thread waits until the background thread is finished with its current iteration using the `join` method

    // This simple solution works great as long as the flag is regularly checked by the background thread
    // If it gets stuck in `some_work()` for a long time, that can result in an unacceptable delay between the `stop` command the program quitting

    // Example: Progress Reporting -----

    // In our next example, we process 100 items one by one on a background thread, while the main thread gives the user regular updates on the process
    
    // Create an atomic counter to track completed items, shared between threads
    let num_done = AtomicUsize::new(0);

    // Spawn a thread scope - this will automatically handle joining when all the threads finish
    thread::scope(|s| {
        // Spawn a background thread to process all 100 items
        s.spawn(|| {
            for i in 0..100 {
                process_item(i); // Assuming this takes some time
                // Update the counter after each item is processed 
                num_done.store(i + 1, Relaxed);
            }
        });

        // The main thread shows status updates, every second
        loop {
            // Read the current count of completed items
            let n = num_done.load(Relaxed);
            // Exit the loop when all 100 items are done
            if n == 100 { break; }
            // Display the progress to the user
            println!("Working.. {n}/100 done");
            // Wait 1 second before checking again
            thread::sleep(Duration::from_secs(1));
        }
    });
    // The `thread::scope` guarantees that when the scope ends, all spawned threads within it have completed
    // So even when the main thread's loop exits when it sees `n == 100`, the scope won't actually close until the background thread finishes its work
    // In this specific case, the background thread should be done when the main thread sees 100, but the scope provides the guarantee
    // The main thread's job is just to poll the progress and display status updates every second while the background thread does the actual work

    // All work is complete - the scope ensures the background thread has finished
    println!("Done!");

    // This time we used a scoped thread, which will automatically handle the joining of the thread for us, and also allow us to borrow local variables

    // Every time the background thread finishes processing an item, it stores the number of processed items in an `AtomicUsize`
    // Meanwhile, the main thread shows the number to the user to inform them of the progress, about once per second
    // Once the main thread sees that all 100 items have been processed, it exists the scope, which implicitly joins the background thread, and informs the user that everything is done

    // Synchronization -----

    // Once the last item is processed, it might take up to one whole second for the main thread to know, introducing an unnecessary delay at the end
    // To solve this, we can use thread parking to wake up the main thread from its sleep whenever there is new information it might be interested in
    let num_done = AtomicUsize::new(0);

    let main_thread = thread::current();

    thread::scope(|s| {
        // A background thread to process all 100 items.
        s.spawn(|| {
            for i in 0..100 {
                process_item(i); // Assuming this takes some time.
                num_done.store(i + 1, Relaxed);
                main_thread.unpark(); // Wake up the main thread.
            }
        });

        // The main thread shows status updates.
        loop {
            let n = num_done.load(Relaxed);
            if n == 100 { break; }
            println!("Working.. {n}/100 done");
            thread::park_timeout(Duration::from_secs(1));
        }
    });

    println!("Done!");
    // In the previous version:
        // THe main thread unconditionally sleeps for 1 full second between checks
        // Checks the progress exactly once per second, no matter what
        // Could wait up to 1 second after the work completes before printing "Done"
    // In the new version:
        // Main thread parks (sleeps) for up to 1 second
        // Background thread calls `main_thread.unpark()` after each item, which wakes up the main thread immediately
        // Main thread can display progress updates more frequently (after each item completes) instead of just once per second
        // When the 100th item finishes, the main thread wakes up immediately instead of potentially waiting another second
    
    // The second version is more responsive - the main thread gets notified right away when progress happens, rather than blindly checking on a fixed schedule
    // The status updates can now appear as soon as each item completes and the program exists faster when work is done

    // Not much has changed
    // We have obtained a handle the the main thread through `thread::current()`, which is now used by the background thread to unpark the main thread after every status update
    // The main thread now uses `park_timeout` rather than `sleep`, such that it can be interrupted
    // Now, any status updates are immediately reported to the user, while still repeating the last update every second to show that the program is still running

    // Example: Lazy Initialization -----

    // The last example before we move on to more advanced atomic operations is about lazy initialization

    // Imagine there is a value x, which we are reading from a file, obtaining from the OS, or calculating in some other way, that we expect to be constant during the run of a program
    // Maybe x is the version of the operating system, or the total amount of memory, or the 400th digit of tau - it doesn't really matter

    // Since we don't expect it to change, we can request or calculate it only the first time we need it and remember the result
    // The first thread that needs it will have to calculate the value but it can store it in an atomic `static` to make it available for all threads, including itself if it needs it again later

    // Let's take a look at an example of this.
    // To keep things simple, we will assume x is never 0 so that we can use 0 as a placeholder before it is calculated
    fn get_x() -> u64 {
        // Creating a new variable X with a 'static lifetime (lives the entire program)
        static X: AtomicU64 = AtomicU64::new(0);
        // Let the value from X and make it mutable 
        let mut x = X.load(Relaxed);
        // If x == 0, calculate a new value for x and then store the updated value
        if x == 0 {
            // As a note, there is no shadowing happening here 
            // When you do x = calculate_x(), you're just mutating the existing `x` variable not creating a new one
            // Shadowing would be if you redeclared a variable with the same name using `let`
            x = calculate_x();
            X.store(x, Relaxed);
        }
        x
    }
    // The first thread call to `get_x()` will check the static X and see it is still zero, calculate its value, and store the result back
    // In the static to make it available for future use
    // Later, any call to `get_x()` will see that the value in the static is nonzero and return it immediately without calculating it again

    // However, if a second thread calls `get_x()` while the first one is still calculating x, the second thread will also see a 0 and also calculate x in parallel
    // One of the threads will end up overwriting the result of the other, depending on which one finishes first
    // This is called a race
        // Not a data race, which is undefined behavior and impossible in Rust without using `unsafe`, but still a race with an unpredictable winner
    // Since we expect x to be constant, it doesn't matter who wins the race, as the result will be the same regardless
    // Depending on how much time we expect `calculate_x()` to take, this might be very good or very bad

    // If `calculate_x()` is expected to take a very long time, it's better if threads wait wil the first thread is still initializing X to avoid unnecessarily wasting processor time
    // You could implement this using a condition variable or thread parking but that quickly gets too complicated for a small example
    // The Rust standard library provides exactly this functionality through `std::sync::Once` and `std::sync::OnceLock`, so there's usually no need to implement these yourself

    // Fetch-and-Modify Operations -----
}

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
use std::sync::atomic::AtomicI32;
use std::sync::atomic::AtomicU32;

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
        // The main thread unconditionally sleeps for 1 full second between checks
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

    // Here are the key points about atomic load and store operations:
        // - Thread-safe access - Load and store operations on atomic types can be safely called from multiple threads simultaneously without data races
        // - Indivisible operations - Each load or store completes as a single uninterruptible operation - you can't observe half-written values
        // - Memory ordering - You specify an ordering that controls how the operation synchronizes with other threads
        // - Load reads, store writes - `load()` reads the current value from the atomic, `store()` writes a new value to it
        // - No locks required - these operations use CPU-level atomic instructions rather than locks, making them very fast for simple operations
        // - Return values - `load()` returns the value, `store()` returns nothing (just a write operation)

    // Fetch-and-Modify Operations -----

    // Now that we have seen a few use cases for `load()` and `store()`, let's move on to more interesting operations: the fetch-and-modify operations
    // These operations modify the atomic variable, but also load (fetch) the original value, as a single atomic operation

    // The most commonly used ones are `fetch_add` and `fetch_sub`, which perform addition and subtraction, respectively
    // Some of the other available ones are `fetch_or` and `fetch_and` for bitwise operation
    // `fetch_max` and `fetch_min` can be used to keep a running maximum or minimum

    // Their function signatures are:
        // impl AtomicI32 {
        //     pub fn fetch_add(&self, v: i32, ordering: Ordering) -> i32;
        //     pub fn fetch_sub(&self, v: i32, ordering: Ordering) -> i32;
        //     pub fn fetch_or(&self, v: i32, ordering: Ordering) -> i32;
        //     pub fn fetch_and(&self, v: i32, ordering: Ordering) -> i32;
        //     pub fn fetch_nand(&self, v: i32, ordering: Ordering) -> i32;
        //     pub fn fetch_xor(&self, v: i32, ordering: Ordering) -> i32;
        //     pub fn fetch_max(&self, v: i32, ordering: Ordering) -> i32;
        //     pub fn fetch_min(&self, v: i32, ordering: Ordering) -> i32;
        //     pub fn swap(&self, v: i32, ordering: Ordering) -> i32; // "fetch_store"
        // }
    
    // The one outlier is the operation that simply stores a new value, regardless of the old value
    // Instead of `fetch_store`, it has been called `swap`

    // Here is a quick demonstration showing how `fetch_add` returns the value before the operation
    let a = AtomicI32::new(100);
    // Adds 23 to the previous value and stores 100 in a new variable (b)
    let b = a.fetch_add(23, Relaxed);
    let c = a.load(Relaxed);

    assert_eq!(b, 100);
    assert_eq!(c, 123);

    // The `fetch_add` operation incremented a from 100 to 123, but returns to use the old value of 100
    // Any next operation will see the value of 123

    // The return value from these operations is not always relevant
    // If you only need the operation to be applied to the atomic value, but are not interested in the value itself, it's perfectly fine to simply ignore the return value
    
    // An important thing to keep in mind is that `fetch_add` and `fetch_sub` implement wrapping behavior for overflows
    // Incrementing a value past the maximum representable value will wrap around and result in the minimum representable value
    // This is different than the behavior of the plus and minus operators on regular integers, which will panic in debug mode on overflow

    // Example: Progress Reporting from Multiple Threads -----

    // In "Example: Progress Reporting", we used an `AtomicUsize` to report the progress of a background thread
    // If we had split the work over, for example, four threads with each processing 25 items, we'd know to know the progress from all 4 threads
    
    // We could use a separate `AtomicUsize` for each thread and load them all in the main thread and sum them up, but an easier
    // solution is to use a single `AtomicUsize` to track the total number of processed items over all threads

    // To make that work, we can no longer use the `store` method, as that would overwrite progress from other threads
    // Instead, we can use an atomic add operation to increment the counter after every processed item

    // Let's update the example from "Example: Progress Reporting" to split the work over four threads:

    let num_done = &AtomicUsize::new(0);

    thread::scope(|s| {
        // Four background threads to process all 100 items, 25 each.
        for t in 0..4 {
            // We use the `move` keyword to capture `t` (move it into the closure)
            s.spawn(move || {
                // Each thread processes its own 25 item chunk
                for i in 0..25 {
                    process_item(t * 25 + i); // Assuming this takes some time.

                    // Atomically increment the shared counter by 1
                    // This is thread safe - all 4 threads can call this simultaneously
                    // and the total will always be correct (no lost updates)
                    // We are not doing anything with the value that is "fetched"
                    num_done.fetch_add(1, Relaxed);
                }
            });
        }

        // The main thread shows status updates, every second.
        loop {
            // Read the current count of completed items
            // This is safe to do while other threads are still incrementing
            let n = num_done.load(Relaxed);

            // Once all 100 items are done, exit the loop
            if n == 100 { break; }
            println!("Working.. {n}/100 done");

            // Wait 1 second before checking again
            // This prevents spamming the console with updates
            thread::sleep(Duration::from_secs(1));
        }
    });
    // The `thread::scope` waits for all spawned threads to finish before continuing
    // The closing brace blocks until all 4 threads finish
    // This is why scoped threads can borrow safely from the outer scope

    println!("Done!");

    // The problem with `store` in a multi-threaded context, is that with multiple threads, they would be overwriting each other
    // The counter would jump around erratically and you'd lost track of the actual total progress across all threads
    // `store` overwrites the entire value, so it only works with one writer thread, whereas `fetch_add` atomically increments, so multiple threads can contribute to the same counter
    // without losing each other's progress

    // A few things have changed
    // Most importantly, we now spawn 4 background threads rather one one and use `fetch_add` instead of `store` to modify the `num_done` atomic variable

    // More subtly, we now use a `move` closure for the background threads and `num_done` is now a reference
    // This is not related to our use of `fetch_add` but rather to how we spawn four threads in a loop
    // This closure captrues `t` to know which of the four threads it is and thus whether we start at item 0, 25, 50, or 75
    // Without the `move` keyword, the closure would try to capture `t` by reference, which isn't allowed since it only exists briefly during the loop

    // As a `move`, closure it captures rather than borrowing them, giving it a copy of `t`.
    // Because it also captures `num_done`, we have changed that variable to be a reference since we still want to borrow that same `AtomicUsize`
    // Atomic types do not implement the `Copy` trait so we would have gotten an error if we had tried to move one into more than one thread

    // Scoped threads with borrows are like simpler, zero-cost verions of `Arc` that work when you have clear lifetime guarantees

    // Closure capture subtleties aside, the change to use `fetch_add` here is very simple
    // We don't know in which order threads will increment `num_done`, but as the addition is atomic, we don't have to worry about anything and can be sure it 
    // will be exactly 100 when all threads done

    // Example: Statistics -----

    // Continuing with this concept of reporting what other threads are doing through atomics, let's extend our example to also collect and report some statistics on the time it takes 
    // to process an item

    // Next to `num_done`, we are adding 2 atomic variables: `total_time` and `max_time` to keep track of the amount of time spent processing items
    // We will use these to report the average and peak processing times

    // Create 3 atomic counters, all as references from the start
    // This makes them easier to share with the spawned threads
    let num_done = &AtomicUsize::new(0);
    let total_time = &AtomicU64::new(0);
    let max_time = &AtomicU64::new(0);

    // Create a thread scope
    thread::scope(|s| {
        // Four background threads to process all 100 items, 25 each
        for t in 0..4 {
            s.spawn(move || {
                // Eacg thread processes its chunk of 25 items
                for i in 0..25 {
                    // Start timing this item
                    let start = Instant::now();

                    // Do the actual work
                    process_item(t * 25 + i); // Assuming this takes some time

                    // Calculate how long it took (in microseconds)
                    let time_taken = start.elapsed().as_micros() as u64;

                    // Atomatically increment the "done" counter
                    num_done.fetch_add(1, Relaxed);

                    // Atomically add this item's time to the running total
                    // This lets us calculate average later
                    total_time.fetch_add(time_taken, Relaxed);

                    // Atomically update the max if this item took longer than previous
                    // fetch_max only updates if time_taken > max_time
                    max_time.fetch_max(time_taken, Relaxed);
                }
            });
        }

        // The main thread shows status updates, every second=
        loop {
            let total_time = Duration::from_micros(total_time.load(Relaxed));
            let max_time = Duration::from_micros(max_time.load(Relaxed));
            let n = num_done.load(Relaxed);
            if n == 100 { break; }
            if n == 0 {
                println!("Working.. nothing done yet.");
            } else {
                println!(
                    "Working.. {n}/100 done, {:?} average, {:?} peak",
                    total_time / n as u32,
                    max_time,
                );
            }
            thread::sleep(Duration::from_secs(1));
        }
    });

    println!("Done!");

    // In the above example, we are creating references to the atomics to make it cleaner to use in closures
    // Since we are using a thread scope, we can borrow local variables since it ensures that all threads will finish when the scope ends

    // The background threads now use `Instant::now()` and `Instant::elapsed()` to measure the time they spend in `process_item()`.
    // An atomic add operation is used to keep track of the microseconds to `total_time` and an atomic max operation is used to keep track of the
    // highest measurement in `max_time`.
    
    // The main thread divides the total time by the number of processed items to obtain the average processing time, which it then reports together
    // with the peak time from `max_time`.

    // Since the 3 atomic variables are updated separately, it is possible for the main thread to load the values after a thread has incremented `num_done`
    // but before it has updated `total_time`, resulting in an underestimate of the average. 
    // More subtly, because the `Relaxed` memory ordering gives no guarantees about the relative order of operations as seen from another thread, it might even
    // briefly see a new updated value of `total_time` while still seeing an old value of `num_done`, resulting in an overestimate

    // Neither of this is a big issue in our example. The worst that can happen is that an inaccurate average is briefly reported to the user.

    // If we want to avoid this, we can put the three statistics inside a `Mutex`.
    // Then, we would briefly lock the mutex while updating the 3 numbers, which no longer have to be atomic by themselves.
    // This effectively turns the 3 updates into a single atomic operation, at the cost of locking and unlocking a mutex and potentially temporarily blocking threads

    // You could, for example, put the 3 statistics you want to track as fields in a struct, then wrap that struct in a mutex
    // so that each thread updates all 3 when it acquires the lock, rather than each separate atomic value being updated individually

    // Example: ID Allocation -----

    // Let's move on to a use case where we actually need to return the value from `fetch_add`.

    // Suppose we need some function, `allocate_new_id()` that gives a new unique number every time it is called
    // We might use these numbers to identify tasks or othings in our program; things that need to be uniquely identified by something small that
    // can be easily stored and passed around between threads, such as an integer. 

    fn allocate_new_id() -> u32 {
        // static = lives for the entire program, shared by all calls
        // its an atomic (thread safe) counter
        static NEXT_ID: AtomicU32 = AtomicU32::new(0);
        
        // Atomically: read current value, add 1, return the old value
        NEXT_ID.fetch_add(1, Relaxed)
    }
    // Even if the above function is called multiple times, the line with `static` only runs once whe nthe program starts
    // or the first time the function is called
    // In the first call, NEXT_ID would be created and initialized at 0, then `fetch_add()` runs, returns 0 and NEXT_ID becomes 1
    // On the second call, NEXT_ID already exists, so initialization is skipped, then we go to `fetch_add()`
    // If we were to use `let next_id = ...`, it would be created fresh every call

    // We simply keep track of the next number to give out and increment it every time we load it
    // The first caller will get 0, the second 1, and so on.

    // The only problem here is the wrapping behavior on overflow.
    // The 4,294,967,296th call will overflow the 32-bit integer, such that the next call will return 0 again

    // Whether this is a problem depends on the use case: how likely is it to be called this often, and what's the worst that can happen if the numbers are not unique?
    // While this might seem like a huge number, modern computers can easily execute our function that many times within seconds
    // If memory safety is dependent on these numbers being unique, our implementation above is not acceptable

    // To solve this, we can attempt to make the function panic if it is called too many times, like this:
    // This version is problematic.
    fn allocate_new_id() -> u32 {
        static NEXT_ID: AtomicU32 = AtomicU32::new(0);
        let id = NEXT_ID.fetch_add(1, Relaxed);
        assert!(id < 1000, "too many IDs!");
        id
    }
    // Now, the `assert` statement will panic after a thousand calls
    // However, this happens after the atomic add operation already happened, meaning that NEXT_ID has already been incremented to 100 when we panic
    // If another thread then calls the function, it'll increment it to 1002 before panicking and so on
    // Although it might take significantly longer, we will run into the same problem after 4,294,966,296 panics when NEXT_ID will overflow to 0 again

    // There are 3 common solutions to this problem:
        // 1. Not panic, but instead completely abort the process on overflow
            // - The `std::process::abort` function will abort the entire process, ruling out the possibility of anything continuing to call our function
            // - While the aborting process might take a brief moment in which the function can still be called by other threads, the chance of that happening
            // - before the program is truly aborted is negligible.
            // This is how the overflow check in `Arc::clone()` is implemented, in case you somehow manage to clone it `isize::MAX` times
            // That would take hundreds of years on a 64-bit computer, but is achievable in seconds if `isze` is only 32 bits
    
    // As a reminder, the above is about integer overflow - when atomics "wrap around" after reaching their maximum value
    // They will overflow back to 0
    // `Arc::clone()` has overflow checks that aborts the process if you somehow many to clone it too many times
    // If it overflows:
        // It wraps around to 0
        // Arc thinks there are 0 references
        // It might drop the data prematurely while clones still exist
        // Use-after-free bugs, crashes, data corruption

    // A second way to deal with the overflow is to use `fetch_sub` to decrement the counter again before panicking:
    fn allocate_new_id() -> u32 {
        static NEXT_ID: AtomicU32 = AtomicU32::new(0);
        let id = NEXT_ID.fetch_add(1, Relaxed);
        if id >= 1000 {
            NEXT_ID.fetch_sub(1, Relaxed);
            panic!("too many IDs!");
        }
        id
    }

    // It's still possible for the counter to very briefly be incremented beyond 100 when multiple threads execute this function as the same time
    // But it is limited by the number of active threads
    // It's reasonable to assume there will never be billions of active threads at once, especially not all simultaneously exeucing the same function 
    // in the brief moment between `fetch_add` and `fetch_sub`

    // This is how overflows are handled for the number of running threads in the standard library's `thread::scope` implementation

    // The third way of handling overflows is arguably the truly correct one, as it prevents the addition from happening at all if it would overflow
    // However, we cannot implement that with the atomic operations we've seen so far
    // For this, we would need compare-and-exchange operations.

    // Compare-and-Exchange Operations -----

    
}

// Every program starts with exactly one thread: the main thread.
// This thread will execute your main function and can be used to spawn more threads if necessary.
// In Rust, threads are spawned using std::thread::spawn.
// It takes a single argument: the function the new thread will execute
// The thread stops once this function returns

use std::thread;
use std::rc::Rc;
use std::sync::Arc;
use std::marker::PhantomData;
use std::sync::Mutex;
use std::cell::Cell;
use std::time::Duration;
use std::collections::VecDeque;
use std::sync::Condvar;

fn f() {
    println!("Hello from another thread!");

    // The Rust standard library assigns every thread a unique identifier
    // This identifier is accessible through Thread::id() and it of the type ThreadId
    // There's not much you can do with a ThreadId other than copying it around and checking for equality
    // There is no guarantee that these IDs will be assigned consecutively, only that they will be different for each thread
    let id = thread::current().id();
    println!("This is my thread id: {id:?}");
}

fn main() {
    // Below, we spawn 2 threads that execute f as their main function
    // Both of these threads will print a message and show their thread id, while the main thread will also print its own message
    thread::spawn(f);
    thread::spawn(f);

    println!("Hello from the main thread.");

    // If you run the above example several times, you might notice that the output varies between runs and that some of the output is missing
    // What happens is that the main thread finished executing the main function before the newly spawned threads finished executing theirs
    // Returning from main will exit the entire program, even if other threads are still running
    // If we want to make sure threads are finished before we return from main, we can wait for them by joining them 
    // To do so, we have to use the JoinHandle returns by the spawn function
    let t1 = thread::spawn(f);
    let t2 = thread::spawn(f);

    println!("Hello from the main thread.");

    t1.join().unwrap();
    t2.join().unwrap();
    // The .join() method waits until the thread has finished executing and returns a std::thread::Result
    // If the thread did not successfully finish its function because it panicked, it will contain the panic message
    // We could attempt to handle that situation or just call .unwrap() to panic when joining a panicked thread

    // Running the above version of the program will no longer result in truncated output

    // Note: The println!() macro uses std::io::Std::lock() to make sure its output does not get interrupted (each call gets exclusive access to stdout)
    // A println!() expression will wait until any concurrently running one is finished before writing any output 
    // If this was not the case, we would get interleaved text

    // Rather than passing the name of a function to std::thread::spawn, it's far more common to pass it a closure
    // This allows us to capture values to move into the new thread
    let numbers = vec![1, 2, 3];

    thread::spawn(move || {
        for n in &numbers {
            println!("{n}");
        }
    }).join().unwrap();

    // Here, ownership of numbers is transferred to the newly spawned thread, since we used a move closure
    // If we had not used the move keyword, the closure would have captured numbers by reference
    // This would have resulted in a compiler error, since the new thread might outlive that variable

    // Since a thread might run until the very end of the program's execution, the spawn function as 'static lifetime bound 
    // on its argument type
    // In other words, it only accepts functions that may be kept around forever
    // A closure capturing a local variable by reference may not be kept around forever, since that reference would become invalid the moment
    // the local variable ceases to exist

    // When you spawn a thread, it runs independently and might keep running after the function that created it returns
    // If the thread could borrow local variables, those variables would be dropped while the thread is trying to use them - a dangling reference

    // Gettubg a value back out of the thread is done by returning it from the closure
    // This return value can be obtained from the Result returned by the join method
    let numbers: Vec<u32> = Vec::from_iter(0..=1000);
    
    let t = thread::spawn(move || {
        let len: u32 = numbers.len() as u32;
        let sum: u32 = numbers.iter().sum();
        sum / len
    });

    let average: u32 = t.join().unwrap();

    println!("average: {average}");
    // Here, the value returned by the thread's closure is sent back to the main thread through the join method
    // If numbers had been empty, the thread would've panicked while trying to divide by 0 and join() would've returned that panic message
    // instead, causing the main thread to panic too because of unwrap()

    // The std::thread::spawn function is actually just a convenient shorthand for std::thread::Builder::new().spawn().unwrap()
    // A std::thread::Builder allows you to set some settings for the new thread before spawning it
    // You can configure the stack size and name it
    // The name of the thread is available through std::thread::current().name(), will be used for panic messages and is visible for debugging
    // Additionally, Builder's spawn function returns an std::io::Result, allowing you to handle situations where spawning a new thread fails

    // Scoped threads -----

    // If we know for sure that a spawned thread will definitely not outlive a certain scope, that thread could safely borrow things that do not live forever, such as local variables
    // Rust provides std::thread::scope to spawn scoped threads
    // This allows us to spawn threads that cannot outlive of the scope of the closure we pass to the function, making it possible to safely borrow local variables
    // In other words, scoped threads can borrow local data and solve the 'static limitation of regular threads - they are guaranteed to finish by the scope ends
    let numbers_two = vec![1, 2, 3];

    thread::scope(|s| { // s is a scope handle that lets you spawn threads within that specific scope
        s.spawn(|| {
            println!("length: {}", numbers_two.len()); // Borrowing
        });
        s.spawn(|| {
            for n in &numbers_two {
                println!("{n}"); // Borrowing
            }
        });
        // If you try returning a ScopedJoinHandle from a scoped thread you might get a lifetime issue
        // This is because the lifetime is tied to the scope itself and returning it might violate this constraint
        // The ScopedJoinHandle cannot outlive the scope it references - it is impossible
    }); // Implicit join happens here

    // In the above example, we:
        // 1. Call std::thread::scope with a closure. Our closure gets directly executed and gets an argument, s, representing the scope.
        // 2. We use s to spawn threads. The closures can borrow local variables like numbers.
        // 3. When the scope ends, all threads that haven't been joined are automatically joined
    
    // This pattern guarantees that none of the thread spawned in the scope can outlive the scope
    // Because of that, this scoped spawn method does not have a 'static bound on its argument type, allowing us to reference anything as long as it outlives the scope, such as numbers_two

    // In the example above, both of the new threads are concurrently accessing numbers
    // This is fine because neither of them (nor the main thread) modifies it
    // If we were to change the first thread to modify numbers, the compiler wouldn't allow us to spawn another thread that also uses numbers
    let mut _numbers_three = vec![1, 2, 3];

        // thread::scope(|s| {
        //     s.spawn(|| {
        //         numbers_three.push(1);
        //     });
        //     s.spawn(|| {
        //         numbers_three.push(2); // Error!
        //     });
        // });
    
    // In short, scoped threads allow us to borrow data because all threads spawned inside of the scope are guaranteed to end when the scope ends
        // And, as an important note, the borrowed data must outlive the scope
    // As opposed to regular threads, which have no guarantee that they will finish before the data is dropped (they run indefinitely), so they can't borrow

    // Shared ownership and reference counting -----

    // So far we have looked at transferring ownership of a value to a thread using a move closure and borrowing data from longer-living parent threads (scoped threads)
    // When sharing data between 2 threads where neither thread is guaranteed to outlive the other, neither of them can be the owner of that data
    // Any data shared between them will need to live as the longest living thread

    // Statics -----
    
    // There are several ways to create something that's not owned by a single thread
    // The simplest one is a static value, which is "owned" by the entire program, instead of an individual thread
    // In the following example, both threads can access X but none of them owns it:
    static X: [i32; 3] = [1, 2, 3];
    // static declares a variable with static storage duration - lives in the program's binary and lives for the entire program
    // 'static is a lifetime annotation meaning: "lives for the entire program duration"

    thread::spawn(|| dbg!(&X)).join().unwrap();
    thread::spawn(|| dbg!(&X)).join().unwrap();

    // A static item has a constant initializer, is never dropped, and already exists before the main function of the program even starts
    // Every thread can borrow it, since it's guaranteed to always exist

    // Leaking -----

    // Another way to share ownership is by leaking an allocation
    // Using Box::leak one can release ownership of a Box, promising to never drop it
    // From that point on, the Box will live forever, without an owner, allowing it to be borrowed by any thread as long as the program runs
    let x: &'static [i32; 3] = Box::leak(Box::new([4, 5, 6]));
    // &'static is a lifetime annotation meaning it is a reference that lives for the entire program (reference that has the lifetime 'static)
    // When we leak memory, we get &'static
    // Static variables are owned by the program itself, so you typically borrow them rather than move them
    // The memory is never freed

    thread::spawn(move || dbg!(x)).join().unwrap();
    thread::spawn(move || dbg!(x)).join().unwrap();
    // The move closure might make it look like we were moving ownership into threads, but a closer look at the 
    // type of x reveales that we are only giving threads a reference to the data

    // References are Copy, meaning when you move them, the original still exists, just like with an integer or boolean

    // Note how the 'static lifetime doesn't mean that the value lived since the start of the program, but only that it lives to the end of the program
    // The past is simply not relevant

    // The downside of leaking a Box is that we are leaking memory. We allocate something, but never drop and deallocate it
    // This is can be fine if it it happens only a limited number of times
    // But, if we keep doing this, the program will slowly run out of memory

    // Reference counting ----- 
     
    // To make sure that shared data gets dropped and deallocated, we can't completely give up its ownership
    // Instead, we can share ownership
    // By keeping track of the number of owners, we can make sure the value is dropped only when there are not owners left

    // The Rust standard library provides this functionality through the std::rc::Rc type, short for reference counted
    // It is similar to a Box, except cloning will not allocate anything new, but instead increment a counter stored next to the contained value
    // Both the original and cloned Rc refer to the same allocation (they share ownership)

    let a = Rc::new([1, 2, 3]);
    let b = a.clone();

    // Checking that two variables point to the same memory address
    // .as_ptr() returns the raw pointer (memory address) of the data
    assert_eq!(a.as_ptr(), b.as_ptr()); // Same allocation!

    // Dropping an Rc will decrement the counter
    // Only the last Rc, which will see the counter drop to zero, will be the one dropping and deallocating the contained data
    
    // If we were trying to send Rc to another thread, however, we would run into an error
    // As it turns out, Rc is not thread safe
    // If multiple threads had an Rc with the same allocation, they might try to modify the reference counter at the same time, which can give unpredictable results
    
    // We use std::sync::Arc, which stands for atomically reference counted, for sharing data across threads
    // It is identical to Rc, except it guarantees that modifications to the reference counter are indivisible atomic operations, making it safe to use it with multiple threads
    let a2 = Arc::new([1, 2, 3]);
    let b2 = a2.clone();

    thread::spawn(move || dbg!(a2)).join().unwrap();
    thread::spawn(move || dbg!(b2)).join().unwrap();
    // In the above example, we put an array in a new allocation together with a reference counter, which starts at 1
    // Cloning the arc increments the reference count to two and provides us with a second Arc to the same location
    // Both threads get their own Arc through which they can access the shared array
        // Both decrement the reference counter when they drop their Arc
        // The last thread to drop its Arc will see the counter drop to zero and will be the one to drop and deallocate the array
    
    // Naming clones:
        // Having to give every clone of an Arc a different name can quickly make the code quite cluttered
        // While every clone of Arc is a separate object, each clone rperesents the same shared value, which is not well reflected by naming each on differently
        // Rust allows (and encourages) you to shadow variables by defining a new variable with the same name
        // If you do that in the same scope, the original variable cannot be named anymore
        // By opening a new scope, a statement like let a = a.clone() can be used to reuse the same name within that scope, while leaving the original variable available outside the scope
        // By wrapping a closure in a new scope with {}, we can clone variables before moving them into the closure, without having to rename them
    
    // In the below example:
        // let a = Arc::new([1, 2, 3]); 

        // let b = a.clone();

        // thread::spawn(move || {
        //     dbg!(b);
        // });

        // dbg!(a);
    // Both a and b are created in the same scope (main function)
    // b is then moved into the thread's scope
    // a stays in the original scope (main thread)
    // Once you move into threads, they live in different scopes
    // At the end, main thread cannot access b since it moved into another thread
    // Each thread gets its own clone with a different name

    // Now, in this example:
        // let a = Arc::new([1, 2, 3]);

        // thread::spawn({
        //     let a = a.clone();
        //     move || {
        //         dbg!(a);
        //     }
        // });

        // dbg!(a);
    // We are using variable shadowing and block expression to avoid creating a separate variable name
    // We create the original a
    // We spawn a thread and create a new scope using {}
    // We read the outer a and clone it, which creates a new a that shadows the outer one
    // The a that moves into the thread is the shadowed one
    // The original a is still accessible

    // The {} block creates a new scope
    // Inside, let a = a.clone() shadows the outer a with a clone (same name)
    // The move closure captures the shadowed a (clone)
    // The original a remains accessible in the main thread
    // This is a common pattern to avoid inventing new variable names like b, c, etc.

    // Shadowing in Rust is when you declare a new variable with the same name as an existing variable, effectively hiding the old one
    // Shadowing creates a new variable, whereas mutation modifies an existing one
    // Very common for transforming values step-by-step, helps avoid creating lots of intermediate variable names
    // If you shadow in the same scope, the new variable overwrites the previous one
    // If you shadow in an inner scope or different scope, the new variable does not overwrite the previous one

    // In the example above, we are creating a new scope inside of a thread and shadowing, thus not overwriting the original variable
    // The outer scope owns the original a and the inner scope (block) owns the shadowed a
    // The shadowed a moves into the closure and the original a stays in the outer scope
    
    // We create a new scope with {}, shadow the variable in the new scope, then move the shadowed variable into the thread's closure

    // Because ownership is shared, reference counting pointers like Rc<T> and Arc<T> have the same restrictions as shared references
    // They do not give you mutable access to their contained value, since the value might be borrowed by other code at the same time
    // For example, if we were to try and sort the slice of integers in an Arc<[i32]>, the compiler would stop us, telling us that we are not allowed to mutate the data

    // Borrowing and data races -----

    // In Rust, values can be borrowed in two ways:
        // Immutable borrowing:
            // Borrowing something with & gives an immutable reference. Such a reference can be copied. Access to the data it references is shared between
            // all copies of such a reference
            // The compiler normally doesn't allow you to mutate something through such a reference, since that might affect other code that's currently borrowing the same data
        // Mutable borrowing:
            // Borrowing something with &mut gives you a mutable reference. A mutable borrow guarantees its the only active borrow of that data
            // This ensures that mutating the data will not change anything that other code is currently looking at
    
    // These 2 concepts together fully prevent data races: situations where on thread is mutating data while another is concurrently accessing it
    // Data races are generally undefined behavior, which means the compiler does not need to take these situations into account - it will simply assume they do not happen

    // To clarify what that means, let's take a look at an example where the compiler can make a useful assumption using the borrowing rules:
        // fn f(a: &i32, b: &mut i32) {
        //     let before = *a;
        //     *b += 1;
        //     let after = *a;
        //     if before != after {
        //         x(); // never happens
        //     }
        // }
    // The above demonstrates that, since a is an immutable borrow, it will never be changed
    // On the other hand, b can be changed since it is a mutable reference

    // Undefined behavior -----

    // Languages like Rust have a set of rules that need to followed to avoid something called undefined behavior
    // For example, Rust's rules is that there may never be more than one mutable reference to any object
    
    // In Rust, it is only possible to break any of these rules when using unsafe code
    // Unsafe doesn't mean that the code is incorrect or never safe to use, but rather than the compiler is not validating for you that the code is safe
    // If the code does violate these rules, it is called unsound

    // The compiler is allowed to assume, without checking, that these rules are never broken
    // When broken, this results in something called undefined behavior, which we need to avoid at all costs
    // If we allow the compiler to make an assumption that is not actually true, it can easily result in more wrong conclusions about different parts of your code

    // As a concrete example, let's take a look at a small snippet that uses the get_unchecked method on a slice:
        // let a = [123, 456, 789];
        // let b = unsafe { a.get_unchecked(index) };
    
    // The get_unchecked method gives us an element of the slice given its index, just like a[index] but allows the compiler to assume the index is always within bounds, without any checks
    // This means that in the code snippet, bacause a is of length 3, the compiler may assume that index is less than 3
    // It's up to use to make sure its assumption holds

    // If we break this assumption, anything might happen, such as crashing
    // Undefined behavior can even "travel back in time", causing problems in code that precedes it
    // When calling any unsafe function, read its documentation carefully and make sure you fully understand its safety requirements:
        // the assumptions you need to uphold, as the caller, to avoid undefined behavior
    
    // Interior mutability -----

    // The borrowing rules as introduced in the previous section are simple, but can be quite limiting - especially when multiple threads are involved
    // Following these rules can make communication between threads extremely limited and almost impossible, since no data that's accessible by multiple threads can be mutated
    // Luckily there is an escape hatch: interior mutability
    // Interior mutability slightly bends the borrowing rules - under certain conditions, those types can allow mutation through an "immutable reference"
   
   // Normally, the Rust borrowing rules are:
    // - If you have an immutable reference &T, the data cannot be modified
    // - If you have a mutable reference &mut T, you can modify the data but can't have any other references
    // Interior mutability lets you bend these rules safely by moving the enforcement from compile-time to runtime
    
    // As soon as interior mutable types are involved, calling a reference immutable or mutable becomes confusing and inaccurate
    // The more accurate terms are "shared" and "exclusive"
        // - A shared reference &T can be copied and shared with others
        // - An excelusive reference &mut T guarantees its the only exclusive borrowing of that T 
    // For most types, shared references do not allow mutation, but there are exceptions
    // In this book, we will be working with these exceptions

    // Let's take a look at a few types with interior mutability and how they can allow mutation through shared references without causing undefined behavior

    // Cell ----- 

    // A std::cell::Cell<T> simply wraps a T, but allows mutations through a shared reference
    // To avoid undefined behavior, it only allows you to copy the value out or replace it with another value as a whole
    // It can only be used in a single thread

    // Cell provides interior mutability for Copy types (ints, booleans, chars, etc.) 
    // It is one of the simplest forms of interior mutability
    // Characteristics:
        // You can change the value inside a Cell even when the Cell itself is not mutable
        // Cell doesn't hand out references to its inner value - it copies values in and out
        // It does not give references to T
        // Use when you need to mutate simple values through a shared reference
        // You interact with get() and set()
        // "A tiny box where I can swap values in and out, but never peek inside directly"
        // "A value I can replace but never borrow"
    // For example:
    
        // fn main() {
        //     let x = Cell::new(5);

        //     x.set(10); // x is not declared as mutable
        //     println!("{}", x.get()); // prints 10
        // }
    // No mut x, yet the value changes - that's interior mutability

    // The restrictions on a Cell are not always easy to work with
    // Since you can't directly borrow the value it holds, we need to move a value out (leaving something in its place), modify it, then put it back to mutate its contents
    // Cell is simpler and more efficient when you only need to work with copyable values

    // Cell is used commonly in situations when &mut self is too restrictive and you only want &self
        // When you have &self you cannot change anything inside it, but sometimes you want to, so Cell<T> lets you do exactly that safely
    // That is, you have an immutable reference but you need to update some simple metadata or state
    // Cell lets you keep the API simple while still tracking internal state

    // Cell is for when you want to:
        // Keep your API simple with &self methods
        // Modify some internal state from several places
        // Avoid the hassle of &mut self everywhere
    

    // RefCell -----

    // Unlike a regular Cell, a std::cell::RefCell does allow you to borrow its contents, at a small runetime cost
    // A RefCell<T> does not only hold a T, but also holds a counter that keeps track of any outstanding borrows
    // If you try to borrow it while it is already mutable borrowed, it will panic, which avoids undefined behavior
    // Just like a Cell, a RefCell can only be used by a single thread
    // Borrowing the contents of RefCell is done by calling .borrow() or .borrow_mut()
    // While Cell and RefCell can be very useful, they become rather useless when we need to do something with multiple threads
    
    // Normally in Rust, the borrow checker enforces rules at compile time:
        // You can have multiple immutable references or one mutable reference, but not both at the same time
    // However, sometimes you need to mutate data even when you only have an immutable reference to it 

    // RefCell moves borrow checking from compile time to runtime 
    // Instead of the compiler checking the rules, RefCell checks them when your program runs

    // Key characteristics:
        // Single threaded only
        // Runtime panics if you violate borrowing rules
        // Zero cost abstraction
    
    // Common use-cases:
        // Mutating data inside immutable structs
        // Implementing data structures with shared ownership (often combined with Rc<RefCell<T>>)
        // Mock objects in tests
    
    // Mutex and RwLock

    // A RwLock or reader-write lock is the concurrent version of RefCell
    // An RwLock<T> holds a T and tracks any outstanding borrows
    // However, it does not panic on conflicting borrows
    // Instead, it blocks the current thread, putting it to sleep, while waiting for conflicting borrows to disappear
    // Borrowing the contents of an RwLock is called locking
    // By locking it, we temporarily block concurrent conflicting borrows, allowing us to borrow it without causing data races

    // In other words, RwLock<T> provides interior mutability in multi-threaded contexts with multiple concurrent readers or one exclusive writer
    // When you have data that's read frequently but written rarely, a regular Mutex can be inefficient because it only allows one thread at a time, even for reads
    // RwLock allows:
        // Multiple threads to read simultaneously
        // Only one thread to write (with exclusive access)
    
    // A Mutex is very similar, but conceptually simpler
    // Instead of keeping track of the number of shared and exclusive borrows, it only allows exclusive borrows
    // Only one thread can read or write at a time

    // Atomics -----

    // Atomics are lock-free, thread-safe primitives that allow safe concurrent access without mutexes
    // You must specify memory ordering

    // UnsafeCell -----

    // An UnsafeCell is the primitive building block for interior mutability 
    // It wraps a T but does not come with any conditions or restrictions to avoid undefined behavior
    // Instead, the get() method gives a raw pointer to the value it wraps, which can be meaningfully used in unsafe blocks

    // Thread Safety: Send and Sync

    // In this chapter, we've seen several types that are not thread safe, types that can only be used on a single thread, such as Rc, Cell, and others
    // Since that restriction is needed to avoid undefined behavior, it's something the compiler needs to understand and check for you, so you can use these types without having to use unsafe blocks

    // The language uses 2 special traits to keep track of which types can be safely used across threads:
        // - Send: A type is Send if it can be sent to another thread. In other words, if ownership of a value of that type can be transferred to another thread.
        // - Sync: A type is Sync if it can be shared with another thread. In other words, a type T is Sync if and only if a shared reference to that type is Send. 
    // All primitive types, such as i32, bool, and str are both Send and Sync

    // Send and Sync are marker traits that define Rust's thread safety guarantees
    // They are automatically implemented by the compiler for most types
    // A type is Send if it's safe to transfer ownership to another thread
    // A type is Sync if it's safe to share references across threads
    // Send = "Can I move this to another thread?"
    // Sync = "Can multiple threads safely hold &T at the same time?"

    // Both of these traits are auto traits, which means that they are automatically implemented for your types based on their fields
    // A struct with fields that are all Send and Sync are itself also Send and Sync

    // The way to opt out of either of these is to add a field to your type that does not implement the trait
    // For that purpose, the special std::marker::PhantomData<T> type comes in handy
    // That type is treated by the compiler as T, except it doesn't actually exist at runtime
    // It's a zero-sized type, taking no space

    // Let's take a look at the following struct:

    struct X {
        handle: i32,
        _not_sync: PhantomData<Cell<()>>,
    }
    // In this example, X would be both Send and Sync if handle was the only field
    // However, we added a zero-sized PhantomData<Cell<()>> field, which is treated as it it were Cell<()>
    // Since a Cell<()> is not a Sync, neither is X 
    // The struct is send, however, since all of its fields implement Send

    // Raw pointers are neither Send nor Sync, since the compiler doesn't know much about what they represent

    // The way to opt in to either of the traits is the same as with any other trait: Use an impl block to implement the trait for your type
    struct Y {
        p: *mut i32,
    }

    unsafe impl Send for Y {}
    unsafe impl Sync for Y {}

    // Note how implementing these traits requires the unsafe keyword, since the compiler cannot check for you if it's correct
    // It's a promise you make to the compiler, which it will have to trust
    
    // If you try to move something into another thread which is not Send, the compiler will politely stop you from doing that
    // Such as with the example below:
        // fn main() {
        //     let a = Rc::new(123);
        //     thread::spawn(move || { // Error!
        //         dbg!(a);
        //     });
        // }
    
    // The thread::spawn function requires its argument to be Send and a closure is only Send if all of its captures are
    // If we try to capture something that's not Send, our mistake is caught, protecting us from undefined behavior

    // Locking: Mutexes and RwLocks -----

    // The most commonly used tool for sharing (mutable) data between threads is mutex, which is short for mutual exclusion
    // The job of a mutex is to ensure threads have exclusive access to some data by temporarily blocking other threads that try to access it at the same time
    
    // Conceptually, a mutex has only 2 states: locked and unlocked
    // When a thread locks an unlocked mutex, the mutex is marked as locked and the thread can immediately continue
    // When a thread then attempts to lock an already locked mutex, that operation will block
    // The thread is put to sleep while it waits for the mutex to be unlocked
    // Unlocking is only possible on a locked mutex, and should be done by the same thread that locked it
    // If other threads are waiting to lock the mutex, unlocking it will cause one of those threads to be woken up, so it can try to lock the mutex again and continue its course

    // Protecting data with a mutex is simply the agreement between all threads that they will only access the data they have mutex locked
    // That way, no 2 threads can ever access that data concurrently and cause a data race

    // Rust's Mutex -----

    // The Rust standard library provides this functionality through std::sync::Mutex<T>
    // It is a generic over type T, which is the type of the data the mutex is protecting
    // By making T part of the mutex, the data can only be accessed through the mutex, allowing for a safe interface that can guarantee threeads will uphold this agreement

    // To ensure a locked mutex can only be unlocked by a thread that locked it, it does not have an `unlock()` method
    // Instead, `.lock()` returns a special type called a MutexGuard
    // This guard represents the guarantee that we have locked the mutex
    // It behaves like an exclusive reference through the DerefMut trait, giving us exclusive access to the data the mutex protects
    // Unlocking the mutex is done by dropping the guard
    // When we drop the guard, we give up our ability to addess the data, and the Drop implementation of the guard will unlock the mutex

    // The code below spawns a thread scope
    // Within the thread scope, we spawn 10 threads
    // Each thread will acquire the lock and increment `n` 100 times
    // Once the for loop ends, the thread finishes, so the guard is dropped and another thread can acquire the lock
    let n = Mutex::new(0);

    thread::scope(|s| {
        for _ in 0..10 {
            s.spawn(|| {
                let mut guard = n.lock().unwrap();
                for _ in 0..100 {
                    *guard += 1;
                }
            });
        }
    });

    assert_eq!(n.into_inner().unwrap(), 1000);
    // Above, we have a Mutex<i32>, a mutex protecting an integer
    // We spawn 10 threads to each increment the integer 100 times
    // Each thread will first lock the mutex to obtain a MutexGuard and then use that guard to access the integer and modify it
    // The guard is implicitly dropped right after, when that variable goes out of scope

    // After the threads are done, we can safely remove the protection from the integer through `into_inner()`
    // `into_inner()` takes ownership of the mutex, which guarantees that nothing else can have a reference to the mutex anymore, making lock unnecessary

    // Even though the increments happen in steps of one, a thread observing the integer would only ever see multiples of 100, since it can only look at the integer when the mutex is unlocked
    // Effectively, thanks to the mutex, the one hundred increments together are now a single indivisible (atomic) operation

    // To clearly see the effect of the mutex, we can make each thread wait a second before unlocking the mutex
    let n2 = Mutex::new(0);
        thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    let mut guard = n2.lock().unwrap();
                    for _ in 0..100 {
                        *guard += 1;
                    }
                    thread::sleep(Duration::from_secs(1)); // New
                });
            }
        });
        assert_eq!(n2.into_inner().unwrap(), 1000);
        // In the above example, the mutex is dropped when the thread finishes, after waiting for 1 second in each thread
        // This will now take about 10 seconds to complete
        // Each thread waits for one second, but the mutex ensures that only one thread at a time can do so
        // Essentially, the lock remains held during the entire 1-second sleep, which forces threads to run sequentially

        // If we drop the guard and unlock the mutex, before sleeping for one second, we will see it happen in parallel instead:
        let n3 = Mutex::new(0);
        thread::scope(|s| {
            for _ in 0..10 {
                s.spawn(|| {
                    let mut guard = n3.lock().unwrap();
                    for _ in 0..100 {
                        *guard += 1;
                    }
                    drop(guard); // New: drop the guard before sleeping
                    thread::sleep(Duration::from_secs(1));
                });
            }
        });
        assert_eq!(n3.into_inner().unwrap(), 1000);
        // With this change, the program only takes about one second, since now the 10 threads can execute their one-second sleep at the same time
        // This shows the importance of keeping the amount of time a mutex is locked as short as possible
        // Keeping a mutex locked longer than necessary can completely nullify any benefits of parallelism, effectively forcing everythint to happen serially instead

        // Lock Poisoning -----

        // The .unwrap() calls in the examples above rlate to lock poisoning
        
        // A mutex in Rust gets marked as poisoned when a thread panics while holding the lock
        // When that happens, the Mutex will no longer be locked, but calling its lock method will result in an Err to indicate it has been poisoned

        // This is a mechanism to protect against leaving the data that's protected by a mutex in an inconsistent state
        // In the examples above, if a thread would panic after incrementing the integer fewer than 100 times, the mutex would unlock and the integer
        // would be left in an unexpected state where it is no longer a multiple of 100, possibly breaking assumptions made by other threads
        // Automatically marking the mutex as poisoned in that case forces the user to handle this possibility

        // Calling `lock()` on a poisoned mutex still locks the mutex.
        // The Err return by `lock()` contains the MutexGuard, allowing us to correct an inconsistent state if necessary

        // While lock poisoning might seem like a powerful mechanism, recovering from a potentially inconsistent state is not often done in practice
        // Most code either disregards poison or uses `unwrap()` to panic if the lock was poisoned, effectively propagating panics to all users of the mutex

        // Life of the MutexGuard -----

        // While it's convenient that implicitly dropping a guard unlocks the mutex, it can sometimes lead to subtle surprises
        // If we assign the guard a name with a `let` statement, it's relatively straightforward to see when it will be dropped 
        // since local variables are dropped at the end of the scope they are defined in.
        // Still, not explicitly dropping a guard might lead to keeping the mutex locked longer than necessary, as demonstrated in the examples above
        
        // Using a guard without assigning it a name is also possible and can be very convenient at times
        // Since MutexGuard behaves like an exclusive reference to protected data, we can directly use it without assigning a name to the guard first
        // For example, if you have a Mutex<Vec<i32>>, you can lock the mutex, push an item into the Vec, and unlock the mutex again in a single statement:
            // list.lock().unwrap().push(1);
        // The MutexGuard drops at the end here since, when you chain methods without assigning it to a variable, it has a temporary lifetime
        // In Rust, temporaries are automatically dropped at the end of the statement (at the semicolon)
        // As opposed to storing a guard in a variable, where the lock will be held until the variable goes out of scope
        
        // Any temporaries produced within a larger expression, such as the guard returned by `lock()`, will be dropped at the end of the statement.
        // While this might seem obvious and resonable, it leads to a common pitfall that usually involves a `match`, `if let`, or `while let` statement
        // Below is an example:
            // if let Some(item) = list.lock().unwrap().pop() {
            //     process_item(item);
            // }
        // If our intention was to lock the list, pop an item, unlock the list, and then process the item after the list is unlocked, we made an important mistake
        // The temporary guard is not dropped until the end of the entire `if let` statement, meaning we needlessly hold on to the lock will processing the item
        // This happens because Rust extends temporary lifetimes in certain contexts to keep them alive for the entire block
        // This is usually helpful, but with locks it can accidentally create longer critical sections than intended

        // Perhaps surprisingly, this does not happen for a similar `if` statement, such as in this example:
            // if list.lock().unwrap().pop() == Some(1) {
            //     do_something();
            // }
        // Here, the temporary guard does get dropped before the body of the `if` statement is executed
        // The reason is that the condition of a regular `if` statement is always a plain boolean, which cannot borrow anything
        // There is no reason to extend the lifetime of temporaries from the condition to the end of the statement
        // For an `if let` statement, however, that might not be the case
        // If we had used front() rather than pop(), `item` would be borrowed from the list, making it necessary to keep the guard around

        // With `if let`, the pattern binding `Some(item)` can extend the temporary's lifetime
        // But with a simple boolean condition `==Some(1)`, the temporary is dropped as soon as the value is extracted, before the comparison

        // We can avoid this by moving the pop operation to a separate `let` statement
        // Then the gard is dropped at the end of that statement, before the `if let`
            // let item = list.lock().unwrap().pop();
            // if let Some(item) = item {
            //     process_item(item);
            // }
        // This works since `.pop()` returns an owned value `Option<T>` not a reference
        // `.list.lock.unwrap()` creates a temporary MutexGuard
        // `.pop()` extracts and returns an owned `Option<T>`
        // `;` is when the statement ends, MutexGuard is dropped and the lock is released
        // `item` stores the owned `Option<T>` which no longer needs a lock
        // This pattern works when you move/copy data out of the mutex - it wouldn't work if you tried to keep a reference
        // The above works since you're extracting the owned data before releasing the lock

        // Reader-Writer Lock -----

        // A mutex is only concerned with exclusive access.
        // The mutex guard will provide us an exclusive reference `&mut T` to the protected data, even if we only want to look at the data and a shared reference `&T` would have sufficed

        // A reader-writer lock is a slightly more complicated version of a mutex that understands the difference between exlcusive and shared acess, and can provide either
        // It has 3 states:
            // Unlocked
            // Locked by a single writer (for exclusive access)
            // Locked by a number of readers (for shared access)
        // It is commonly used for data that is often read by multiple threads but only updated once in awhile

        // The Rust library provides this lock through the `std::sync::RwLock<T>` type
        // It works similarly to the standard `Mutex`, except its interface is mostly split into 2 parts
        // Instead of a single `.lock()` method, it has a `read()` and `write()` method for locking as either a reader or a writer
        // It comes with 2 guard types, one for readers and one for writers
        
        // It is effectively the multi-threaded version of `RefCell`, dynamically tracking the number of references to ensure the borrow rules are upheld

        // Both `Mutex<T>` and `RwLock<T>` require T to be `Send` since they can be used to send a `T` to another thread
        // `RwLock<T>` additionally requires `T` to also implement `Sync` because it allows multiple threads to hold a shared reference `&T` to the protected data

        // The Rust standard library provides only one general purpose `RwLock` type, but its implementation depends on the OS
        // There are many subtle variations between reader-writer lock implementations
        // Most implementations will block new readers when there is a writer waiting, even when the lock is already read locked
        // This is done to prevent writer starvation, a situation where many readers collectively keep the lock from ever unlocking, never allowing the writer to update the data

        // Waiting: Parking and Condition Variables -----

        // When data is mutated by multiple threads, there are many situations where they would need to wait for some event, for some condition about the data to become true
        // For example, if we have a mutex protecting a `Vec`, we might want to wait until it contains anything

        // While a mutex does allow threads to wait until it becomes unlocked, it does not provide functionality for waiting for any other conditions
        // If a mutex was all we had, we would have to keep locking the mutex to repeatedly check if there's anything in the `Vec` yet

        // Thread Parking -----

        // One way to wait for a notification from another thread is called thread parking
        // A thread can park itself which puts it to sleep, stopping it from consuming any CPU cycles
        // Another thread can then unpark the parked thread, waking it up from its nap

        // Thread parking is available through the `std::thread::park()` function.
        // For unparking, you call the `unpark()` method on a `Thread` object representing the thread you want to unpark
        // Such an object can be obtained from the join handle returned by `spawn` or by the thread itself through `std::thread::current()`

        //  Let's dive into an example that uses a mutex to share a queue between 2 threads
        // In the following example, a newly spawned thread will consume items from the queue, while the main thread will insert a new item in the queue every second
        // Thread parking is used to make the consuming thread wait until the queue is empty

        // Creating a Mutex wrapped, double ended queue
        let queue = Mutex::new(VecDeque::new());

        // Spawning a scope thread that will have a single thread inside of it
        // The thread inside of the scope will be the consumer thread
        // As a note, thread::scope just creates a scope, it doesn't spawn any threads itself
        thread::scope(|s| {
            // Consuming thread
            // We are using `loop` since we do not know how many items we will receive
            let t = s.spawn(|| loop {
                // Locking the mutex and popping from the front
                // The MutexGuard will drop at the end of this statement
                let item = queue.lock().unwrap().pop_front();
                // If the value (item) matches the pattern (Some(item)), we bind item to item
                // If it is Some(item), we use dbg!()
                // If it is None, we park the thread
                if let Some(item) = item {
                    dbg!(item);
                } else {
                    thread::park();
                }
            });

            // Producing thread
            for i in 0.. {
                // Locking the mutex and pushing to the back
                // The MutexGuard will drop at the end of this statement
                queue.lock().unwrap().push_back(i);
                // Unparking the thread
                t.thread().unpark();
                //Sleeping for 1 second
                thread::sleep(Duration::from_secs(1));
            }

        });
        // We are using a `thread::scope` to get access to the thread handle `t` so we can call `t.thread().unpark()` from the producer thread
        // `thread::scope` blocks and waits for all spawned threads to complete before returning
        // If the producing thread were outside of the `thread::scope`, it would never run because the scope would never exit (due to the infinite loop)

        // The consuming thread runs an infinite loop in which it pops items out of the queue to display them using the `dbg` macro
        // When the queue is empty, it stops and goes to sleep using the `park()` function
        // If it gets unparked, the `park()` call returns, and the `loop` continues, popping items for the queue again until it is empty, and so on

        // The producing thread produces a new number every second by pushing it into the queue
        // Every time it adds an item, it uses the `unpark()` method on the `Thread` object that referes to the consuming thread to unpark it
        // That way, the consuming thread gets woken up to process the new element

        // An import observation to make here is that this program would still be theoretically correct, although inefficient, if we remove parking.
        // This is import because `park()` does not guarantee that it will only return because of matching `unpark()`
        // While somewhat rare, it might have spurious wake-ups
        // Our example deals with that just fine, because the consuming thread will lock the queue, see that it is empty, and directly unlock it and park itself again

        // Key points from above:
        // 1. Parking doesn't guarantee one-to-one wake ups
            // `thread::park()` can wake up "spuriously" - meaning it might return even though no one called `unpark()` 
        // 2. The code above handles this correctly
            // If the thread wakes up spuriously, it loops back to the top, tries to pop, queue is empty, parks itself again
        // 3. Without `park` it would be inefficient since it's a busy-wait loop that burns CPU constantly checking an empty queue

        // The pattern: Always check your actual condition rather than trusting `park()`/`unpark()` to be perfectly synchronized

        // Unpark requests are saved (but don't stack)
            // - If you call `unpark()` before `park()`, the `unpark()` is remembered
            // - The next `park()` will immediately return without sleeping
            // - Multiple `unpark()` calls = only one saved request
                // - unpark() → unpark() → park() → park() = thread sleeps on the second park
        
        // Bottom line: Park/unpark is simple but fragile. Always check your actual condition in a loop and consider alternatives for anymore more complex

        // Condition Variables -----

        // Condition varaibles are a more commonly used option for waiting for something to happen to data protected by a mutex
        // They have 2 basic operations: wait and notify
        // Threads can wait on a condition variable, after which they can be woken up when another thread notifies that same condition variable
        // Multiple threads can wait on the same condition variable, and notifications can either be sent to one waiting thread or to all of them

        // This means that we can create a condition variable for specific events or conditions we are interested in, such as the queue being non-empty and wait on that condition
        // Any thread that causes that event or condition to happen then notifies the condition variable, without having to know which or how many threads are interested in that notification

        // To avoid the issue of missing notifications in the brief moment between unlocking a mutex and waiting for a condition variable, condition variables provide a way to atomtically unlock the mutex and start waiting
        // This means there is simply no possible moment for notifications to get lost

        // The Rust standard library provides a condition variable as `std::sync::Condvar`
        // Its `wait` method takes a `MutexGuard` that proves we've locked the mutex
        // It first unlocks the mutex and goes to sleep
        // Later, when woken up, it relocks the mutex and returns a new MutexGuard

        // It has 2 notify functions: `notify_one` to wake up just one waiting thread and `notify_all` to wake them all up

        // Create an empty queue wrapped in mutex
        let queue2 = Mutex::new(VecDeque::new());
        // Spawn a new conditional variable
        let not_empty = Condvar::new();

        // Spawn a new thread scope
        thread::scope(|s| {

            // Spawn a thread within the scope
            s.spawn(|| {
                loop {
                    // Lock the mutex
                    let mut q = queue2.lock().unwrap();
                    let item = loop { // Inner loop: wait until queue has an item
                        if let Some(item) = q.pop_front() {
                            break item; // Got an item - break out with it
                        } else { // Queue is empty, wait for notification
                            q = not_empty.wait(q).unwrap(); // Releases lock, sleeps, reacquires lock when woken
                        }
                    };
                    // Drop the mutex guard 
                    drop(q);
                    dbg!(item); // Process the item
                }
            });

            for i in 0.. {
                queue2.lock().unwrap().push_back(i);
                not_empty.notify_one();
                thread::sleep(Duration::from_secs(1));
            }
        });

        // We had to change a few things:
            // We now not only have a Mutex containing the queue, but also a Condvar to communicate the not empty condition
            // We no longer need to know which thread to wake up, so we don't store the return value from spawn anymore
                // Instead, we notify the consumer through the condition variable with the `notify_one` method
            // Unlocking, waiting, and relocking is all done by the `wait` method
                // We had to structure the control flow a bit to be able to pass the guard to the `wait` method, while still dropping before processing
        
        // Step-by-step:
            // 1. Check: Is the queue empty?
            // 2. If yes: Call `wait()` - this release the lock and sleeps
            // 3. Producer adds an item and calls `notify_one()`
            // 4. Consumer wakes up with the lock reacquired
            // 5. Loop back to step 1 - check if the queue is not empty now
            // 6. If there's an item: Pop it and break out of the loop
        // We use the loop because `wait()` might wake up even if the queue is still empty, so you must re-check the condition each time you wake up
        
        // Now we can spawn as many consuming threads as we like and even spawn more later without having to change anything
        // The condition variable takes care of delivering notifications to whichever thread is interested

        // If we had a more complicated system with threads that are interested in different conditions, we could define a `Condvar` for each condition
        // For example, we could define one to indicate the queue is non-empty and another one to indicate it is empty
        // Then, each thread would wait for whichever condition relevant to what they are doing

        // Normally, a `Condvar` is only ever used together with a single Mutex
        // If 2 threads try to concurrently `wait` on a condition variable using 2 different mutex, it might cause a panic

        // A downside of a `Condvar` is that it only works when used together with a `Mutex`, but for most use-cases this is perfectly fine, as that's exactly what's already
        // used to protect the data anyway

        // Summary -----

        // - Multiple threads can run concurrently within the same program and can be spawned at any time
        // - When the main thread ends, the entire program ends
        // - Data races are undefined behavior, which is fully prevented (in safe code) by Rust's type system
        // - Data that is `Send` can be sent to other threads and data that is `Sync` can be shared between threads
        // - Regular threads might run as long as the program does, and thus can only borrow `'static` data such as statics and leaked allocations
        // - Reference counting (`Arc`) can be used to share ownership to make sure data lives as long as at least one thread is using it
        // - Scoped threads are useful to limit the lifetime of a thread to allow it to borrow non-`'static` data, such as local variables
        // - `&T` is a shred reference, `&mut T` is an exclusive reference. Regular types don't allow mutation through a shared reference
        // - Some types have interior mutability, which allows for mutation through shared references
        // - `Cell` and `RefCell` are the standard type for single-threaded interior mutability. Atomics, `Mutex`, and `RwLock` are their multi thread equivalents
        // - `Cell` and atomics only allowing replacing the values as a whole. `RefCell`, `Mutex`, and `RwLock` allow you to mutate the value directly by dynamically enforcing access rules
        // - Thread parking can be a convenient way to wait for some condition
        // -  When a condition is about data protected by a `Mutex`, using `Condvar` is more convenient and can be more efficient than thread parking
}
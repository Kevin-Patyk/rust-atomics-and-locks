// Every program starts with exactly one thread: the main thread.
// This thread will execute your main function and can be used to spawn more threads if necessary.
// In Rust, threads are spawned using std::thread::spawn.
// It takes a single argument: the function the new thread will execute
// The thread stops once this function returns

use std::thread;
use std::rc::Rc;
use std::sync::Arc;

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

    // Refcell -----

}

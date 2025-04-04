// main.rs

// This program demonstrates the use of Rust pointer types (`Box`, `Rc`, `Arc`, and raw pointers)
// and reference types (`&` and `&mut`). It showcases ownership, borrowing, and memory safety.

use std::rc::Rc; // For reference counting (single-threaded)
use std::sync::Arc; // For atomic reference counting (multi-threaded)
use std::thread;

fn main() {
    // SECTION 1: Basic References (`&` and `&mut`)
    let x = 42; // Immutable variable
    let y = &x; // Immutable reference to `x`
    println!("Immutable reference: y = {}", y);

    let mut z = 10; // Mutable variable
    {
        let z_ref = &mut z; // Mutable reference to `z`
        *z_ref += 5; // Modify the value through the mutable reference
    }
    println!("Mutable reference: z = {}", z);

    // SECTION 2: Box Pointer (Heap Allocation)
    let boxed_value = Box::new(100); // Allocates an integer on the heap
    println!("Boxed value: {}", boxed_value);

    // Ownership transfer example with `Box`
    let moved_boxed_value = boxed_value; // Ownership of the Box is moved
    // println!("{}", boxed_value); // Uncommenting this will cause a compile error
    println!("Moved Boxed value: {}", moved_boxed_value);

    // SECTION 3: Rc (Reference Counting, Single-Threaded)
    let shared_value = Rc::new(200); // Create an Rc instance
    let shared_value_clone = Rc::clone(&shared_value); // Create a new reference
    println!("Rc value: {}", shared_value);
    println!(
        "Reference count after cloning: {}",
        Rc::strong_count(&shared_value)
    );

    // SECTION 4: Arc (Atomic Reference Counting, Multi-Threaded)
    let shared_arc = Arc::new(300); // Create an Arc instance
    let shared_arc_clone = Arc::clone(&shared_arc); // Create a new reference

    // Spawn a thread to demonstrate Arc usage
    let handle = thread::spawn({
        let shared_arc_clone = Arc::clone(&shared_arc); // Clone for the thread
        move || {
            println!("Value in thread (Arc): {}", shared_arc_clone);
        }
    });

    // Wait for the thread to finish
    handle.join().unwrap();
    println!(
        "Reference count after thread use: {}",
        Arc::strong_count(&shared_arc)
    );

    // SECTION 5: Raw Pointers (*const and *mut)
    // Unsafe block required for raw pointer dereferencing
    unsafe {
        let raw_const_ptr: *const i32 = &x; // Raw constant pointer
        let raw_mut_ptr: *mut i32 = &mut z; // Raw mutable pointer

        println!("Raw const pointer value: {}", *raw_const_ptr);
        println!("Raw mut pointer value (before): {}", *raw_mut_ptr);

        *raw_mut_ptr += 1; // Modify value through raw mutable pointer
        println!("Raw mut pointer value (after): {}", *raw_mut_ptr);
    }

    // SECTION 6: Dangling Pointers (Compile-Time Safety in Rust)
    // Rust ensures that dangling pointers are not possible at compile time.
    // Uncommenting the following code will result in a compile error:
    // let dangling_ref: &i32;
    // {
    //     let temp = 5;
    //     dangling_ref = &temp; // `temp` goes out of scope here
    // }
    // println!("{}", dangling_ref); // Dangling reference would occur here

    println!("Rust prevents dangling pointers at compile time!");

    // SECTION 7: Summary
    println!("\nSummary:");
    println!("1. `&` and `&mut` are references with strict borrowing rules.");
    println!("2. `Box` is for heap allocation with ownership.");
    println!("3. `Rc` is for shared ownership in single-threaded environments.");
    println!("4. `Arc` is for shared ownership in multi-threaded environments.");
    println!("5. Raw pointers (`*const` and `*mut`) are unsafe and require explicit handling.");
}
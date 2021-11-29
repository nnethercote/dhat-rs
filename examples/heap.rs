// This is a very simple example of how to do heap profiling of a program.

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    let _profiler = dhat::Profiler::new_heap();

    println!("Hello, world!");
}

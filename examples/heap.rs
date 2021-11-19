#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    let _dhat = dhat::start_heap_profiling();

    println!("Hello, world!");
}

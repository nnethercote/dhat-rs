// This is a very simple example of how to do heap profiling of a program. You
// may want to create a feature called `dhat-heap` in your `Cargo.toml` and
// uncomment the `#[cfg(feature = "dhat-heap")]` attributes.

//#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    //#[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    println!("Hello, world!");
}

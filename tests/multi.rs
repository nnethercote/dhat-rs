use dhat::{Dhat, DhatAlloc};

#[global_allocator]
static ALLOC: DhatAlloc = DhatAlloc;

// Test it panics if there are multiple `Dhat` instances.
#[test]
#[should_panic]
fn main() {
    let _dhat = Dhat::start_heap_profiling();

    let _v = vec![1u32, 2, 3, 4];

    let _dhat = Dhat::start_heap_profiling();
}

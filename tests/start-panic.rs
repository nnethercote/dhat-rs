use dhat::{Dhat, DhatAlloc};

#[global_allocator]
static ALLOC: DhatAlloc = DhatAlloc;

#[test]
#[should_panic]
fn start_panic() {
    let _dhat = Dhat::start_heap_profiling();

    let _v = vec![1u32, 2, 3, 4];

    let _dhat = Dhat::start_heap_profiling(); // panic
}

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
#[should_panic]
fn start_panic() {
    let _dhat = dhat::start_heap_profiling();

    let _v = vec![1u32, 2, 3, 4];

    let _dhat = dhat::start_heap_profiling(); // panic
}

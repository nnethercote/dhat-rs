use dhat::{Dhat, DhatAlloc};

#[global_allocator]
static ALLOC: DhatAlloc = DhatAlloc;

#[test]
#[should_panic]
fn stats_panic_2() {
    {
        let _dhat = Dhat::start_heap_profiling();

        let _v = vec![1u32, 2, 3, 4];
    }

    let _stats = dhat::get_stats(); // panic
}

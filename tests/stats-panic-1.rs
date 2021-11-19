use dhat::DhatAlloc;

#[global_allocator]
static ALLOC: DhatAlloc = DhatAlloc;

#[test]
#[should_panic]
fn stats_panic_1() {
    let _stats = dhat::get_stats(); // panic
}

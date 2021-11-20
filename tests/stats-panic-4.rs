#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
#[should_panic(expected = "dhat: called get_heap_stats() while doing ad hoc profiling")]
fn stats_panic_1() {
    let _dhat = dhat::start_ad_hoc_profiling();

    let _stats = dhat::get_heap_stats(); // panic
}

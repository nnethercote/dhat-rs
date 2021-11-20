#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
#[should_panic(expected = "dhat: called get_ad_hoc_stats() while doing heap profiling")]
fn stats_panic_1() {
    let _dhat = dhat::start_heap_profiling();

    let _stats = dhat::get_ad_hoc_stats(); // panic
}

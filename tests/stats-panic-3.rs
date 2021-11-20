#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
#[should_panic(expected = "dhat: called get_ad_hoc_stats() while doing heap profiling")]
fn main() {
    let _profiler = dhat::Profiler::heap_start();

    let _stats = dhat::get_ad_hoc_stats(); // panic
}

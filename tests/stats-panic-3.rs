#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
#[should_panic(expected = "dhat: called AdHocStats::get() while doing heap profiling")]
fn main() {
    let _profiler = dhat::Profiler::heap_start();

    let _stats = dhat::AdHocStats::get(); // panic
}

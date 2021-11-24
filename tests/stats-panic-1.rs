#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
#[should_panic(expected = "dhat: getting heap stats before the profiler has started")]
fn main() {
    let _stats = dhat::HeapStats::get(); // panic
}

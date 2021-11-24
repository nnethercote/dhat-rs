#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
#[should_panic(expected = "dhat: getting heap stats after the profiler has stopped")]
fn main() {
    {
        let _profiler = dhat::Profiler::heap_start();
        let _v = vec![1u32, 2, 3, 4];
    }

    let _stats = dhat::HeapStats::get(); // panic
}

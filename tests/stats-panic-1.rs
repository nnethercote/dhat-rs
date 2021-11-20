#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
#[should_panic(expected = "dhat: getting stats before profiling has begun")]
fn main() {
    let _stats = dhat::HeapStats::get(); // panic
}

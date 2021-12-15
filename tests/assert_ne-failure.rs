#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
#[should_panic(expected = "dhat: assertion failed: `(left != right)`\n  left: `32`,\n right: `32`")]
fn main() {
    let _profiler = dhat::Profiler::builder().testing().eprint_json().build();

    let _v1 = vec![1, 2, 3, 4];
    let _v2 = vec![5, 6, 7, 8];

    // Test with and without extra arguments.
    let stats = dhat::HeapStats::get();
    dhat::assert_ne!(stats.curr_blocks, 1);
    dhat::assert_ne!(stats.curr_bytes, 32); // failure
    dhat::assert_ne!(stats.curr_bytes, 33, "more");
}

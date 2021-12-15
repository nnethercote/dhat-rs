// This is a very simple example of how to do heap usage testing of a program.
// To use this code for a test, you need to make some changes:
// - Move it into an integration test file within `tests/`.
// - Rename `main()` to whatever you want, and add `#[test]` to it.
//
// Also, only use one `#[test]` per integration test, to avoid tests
// interfering with each other.

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    let _profiler = dhat::Profiler::builder().testing().build();

    let _v1 = vec![1, 2, 3, 4];
    let _v2 = vec![5, 6, 7, 8];

    let stats = dhat::HeapStats::get();
    dhat::assert_eq!(stats.curr_blocks, 2);
    dhat::assert_eq!(stats.curr_bytes, 32);
}

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

// Test most of the panics that can occur during heap profiling. Because we
// can't have multiple `#[test]` instances in a single test, we use
// `assert_is_panic` to test multiple panics within a single `#[test]`.
#[test]
fn main() {
    dhat::assert_is_panic(
        || dhat::HeapStats::get(),
        "dhat: getting heap stats when no profiler is running",
    );

    dhat::assert_is_panic(
        || dhat::assert!(true),
        "dhat: asserting when no profiler is running",
    );

    {
        let _profiler = dhat::Profiler::new_heap();

        dhat::assert_is_panic(
            || dhat::Profiler::new_heap(),
            "dhat: creating a profiler while a profiler is already running",
        );

        dhat::assert_is_panic(
            || dhat::AdHocStats::get(),
            "dhat: getting ad hoc stats while doing heap profiling",
        );

        dhat::assert_is_panic(
            || dhat::assert!(true),
            "dhat: asserting while not in testing mode",
        );
    }

    dhat::assert_is_panic(
        || dhat::HeapStats::get(),
        "dhat: getting heap stats when no profiler is running",
    );

    dhat::assert_is_panic(
        || dhat::assert!(true),
        "dhat: asserting when no profiler is running",
    );
}

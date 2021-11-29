// Test most of the panics that can occur during ad hoc profiling. Because we
// can't have multiple `#[test]` instances in a single test, we use
// `assert_is_panic` to test multiple panics within a single `#[test]`.
#[test]
fn main() {
    dhat::assert_is_panic(
        || dhat::AdHocStats::get(),
        "dhat: getting ad hoc stats before the profiler has started",
    );

    dhat::assert_is_panic(
        || dhat::assert!(true),
        "dhat: asserting before the profiler has started",
    );

    {
        let _profiler = dhat::Profiler::ad_hoc_start();

        dhat::assert_is_panic(
            || dhat::Profiler::ad_hoc_start(),
            "dhat: profiling started a second time",
        );

        dhat::assert_is_panic(
            || dhat::HeapStats::get(),
            "dhat: getting heap stats while doing ad hoc profiling",
        );

        dhat::assert_is_panic(
            || dhat::assert!(true),
            "dhat: asserting while not in testing mode",
        );
    }

    dhat::assert_is_panic(
        || dhat::AdHocStats::get(),
        "dhat: getting ad hoc stats after the profiler has stopped",
    );

    dhat::assert_is_panic(
        || dhat::assert!(true),
        "dhat: asserting after the profiler has stopped",
    );
}

use dhat::{Dhat, DhatAlloc};

#[global_allocator]
static ALLOC: DhatAlloc = DhatAlloc;

// Test things are ok if there are multiple `Dhat` instances.
#[test]
fn main() {
    let empty_stats = dhat::Stats {
        total_blocks: 0,
        total_bytes: 0,
        heap: Some(dhat::HeapStats {
            curr_blocks: 0,
            curr_bytes: 0,
            max_blocks: 0,
            max_bytes: 0,
        }),
    };
    let final_stats = dhat::Stats {
        total_blocks: 4,
        total_bytes: 64,
        heap: Some(dhat::HeapStats {
            curr_blocks: 4,
            curr_bytes: 64,
            max_blocks: 4,
            max_bytes: 64,
        }),
    };

    let dhat1 = Dhat::start_heap_profiling();

    assert_eq!(dhat::get_stats(), Some(empty_stats));

    let _v = vec![1u32, 2, 3, 4];

    let dhat2 = Dhat::start_heap_profiling();

    let _v = vec![1u32, 2, 3, 4];

    let dhat3 = Dhat::start_ad_hoc_profiling();

    let _v = vec![1u32, 2, 3, 4];

    let dhat4 = Dhat::start_heap_profiling();

    let _v = vec![1u32, 2, 3, 4];

    assert_eq!(dhat::get_stats(), Some(final_stats.clone()));

    drop(dhat3);
    drop(dhat1);

    let dhat5 = Dhat::start_heap_profiling();
    drop(dhat5);

    let dhat6 = Dhat::start_ad_hoc_profiling();

    drop(dhat4);

    assert_eq!(dhat::get_stats(), Some(final_stats));

    drop(dhat6);
    drop(dhat2);
}

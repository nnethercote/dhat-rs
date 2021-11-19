use dhat::{Dhat, DhatAlloc};

#[global_allocator]
static ALLOC: DhatAlloc = DhatAlloc;

// It's intended that the `Dhat` instance spans the entire program's runtime.
// This tests makes sure things are ok when that doesn't happen, and blocks
// allocated before the `Dhat`'s lifetime are reallocated or freed during or
// after its lifetime.
#[test]
fn main() {
    let v1 = vec![1u32, 2, 3, 4];
    let v2 = vec![1u32, 2, 3, 4];
    let mut v3 = vec![1u32, 2, 3, 4];
    let mut v4 = vec![1u32, 2, 3, 4];

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
        total_blocks: 1,
        total_bytes: 32,
        heap: Some(dhat::HeapStats {
            curr_blocks: 1,
            curr_bytes: 32,
            max_blocks: 1,
            max_bytes: 32,
        }),
    };

    {
        let _dhat = Dhat::start_heap_profiling();

        // Things allocated beforehand aren't counted.
        assert_eq!(dhat::get_stats(), empty_stats);

        // Allocated before, freed during.
        drop(v1);

        // Allocated before, reallocated during.
        v3.push(5);

        // Things allocated during are counted (and the realloc is treated like
        // an alloc, i.e. we count the entire thing, not just the difference
        // between the old and new sizes).
        assert_eq!(dhat::get_stats(), final_stats);
    }

    // Allocated before, freed after.
    drop(v2);

    // Allocated before, reallocated after.
    v4.push(5);
}

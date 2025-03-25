use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::atomic::{AtomicUsize, Ordering},
};

// We only count allocations of our "weird" size to avoid seeing allocations for DHAT
// metadata.
//
// 167 was chosen because it's a not-very-small prime number, so it's unlikely to occur naturally.
const ALLOC_SIZE: usize = 167;

static ALLOC_CALLED: AtomicUsize = AtomicUsize::new(0);
static DEALLOC_CALLED: AtomicUsize = AtomicUsize::new(0);

struct InnerAllocator;

unsafe impl GlobalAlloc for InnerAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.size() == ALLOC_SIZE {
            ALLOC_CALLED.fetch_add(1, Ordering::Relaxed);
        }
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.size() == ALLOC_SIZE {
            DEALLOC_CALLED.fetch_add(1, Ordering::Relaxed);
        }
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static _ALLOCATOR: dhat::Alloc<InnerAllocator> = dhat::Alloc::from_alloc(InnerAllocator);

#[test]
fn stacked_allocators_work() {
    let _profiler = dhat::Profiler::builder().build();

    // Nothing got allocated yet.
    let stats = dhat::HeapStats::get();
    assert_eq!(stats.total_blocks, 0);
    assert_eq!(stats.total_bytes, 0);
    assert_eq!(stats.curr_blocks, 0);
    assert_eq!(stats.curr_bytes, 0);
    assert_eq!(stats.max_blocks, 0);
    assert_eq!(stats.max_bytes, 0);

    let before = ALLOC_CALLED.load(Ordering::Relaxed);
    let x = Box::new([0_u8; ALLOC_SIZE]);
    // The second allocation is the backtrace vector.
    assert_eq!(ALLOC_CALLED.load(Ordering::Relaxed), before + 1);

    let stats = dhat::HeapStats::get();
    assert_eq!(stats.total_blocks, 1);
    assert_eq!(stats.total_bytes, ALLOC_SIZE as u64);
    assert_eq!(stats.curr_blocks, 1);
    assert_eq!(stats.curr_bytes, ALLOC_SIZE);
    assert_eq!(stats.max_blocks, 1);
    assert_eq!(stats.max_bytes, ALLOC_SIZE);

    let before = DEALLOC_CALLED.load(Ordering::Relaxed);
    drop(x);
    assert_eq!(DEALLOC_CALLED.load(Ordering::Relaxed), before + 1);

    let stats = dhat::HeapStats::get();
    assert_eq!(stats.total_blocks, 1);
    assert_eq!(stats.total_bytes, ALLOC_SIZE as u64);
    assert_eq!(stats.curr_blocks, 0);
    assert_eq!(stats.curr_bytes, 0);
    assert_eq!(stats.max_blocks, 1);
    assert_eq!(stats.max_bytes, ALLOC_SIZE);
}

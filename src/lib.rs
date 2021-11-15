//! This crate provides heap profiling and ad hoc profiling capabilities to
//! Rust programs, similar to those provided by [DHAT].
//!
//! [DHAT]: https://www.valgrind.org/docs/manual/dh-manual.html
//!
//! The heap profiling works by using a global allocator that wraps the system
//! allocator, tracks all heap allocations, and on program exit writes data to
//! file so it can be viewed with DHAT's viewer. This corresponds to DHAT's
//! `--mode=heap` mode.
//!
//! The ad hoc profiling is via a second mode of operation, where ad hoc events
//! can be manually inserted into a Rust program for aggregation and viewing.
//! This corresponds to DHAT's `--mode=ad-hoc` mode.
//!
//! # Motivation
//!
//! DHAT is a powerful heap profiler that comes with Valgrind. This crate is a
//! related but alternative choice for heap profiling Rust programs. DHAT and
//! this crate have the following differences.
//! - This crate works on any platform, while DHAT only works on some platforms
//!   (Linux, mostly). (Note that DHAT's viewer is just HTML+JS+CSS and should
//!   work in any modern web browser on any platform.)
//! - This crate causes a much smaller slowdown than DHAT.
//! - This crate requires some modifications to a program's source code and
//!   recompilation, while DHAT does not.
//! - This crate cannot track memory accesses the way DHAT does, because it does
//!   not instrument all memory loads and stores.
//! - This crate does not provide profiling of copy functions such as `memcpy`
//!   and `strcpy`, unlike DHAT.
//! - The backtraces produced by this crate may be better than those produced
//!   by DHAT.
//! - DHAT measures a program's entire execution, but this crate only measures
//!   what happens within the scope of `main`. It will miss the small number of
//!   allocations that occur before or after `main`, within the Rust runtime.
//!
//! # Configuration
//!
//! In your `Cargo.toml` file, as well as specifying `dhat` as a dependency,
//! you should enable source line debug info:
//! ```toml
//! [profile.release]
//! debug = 1
//! ```
//!
//! # Usage (heap profiling)
//!
//! For heap profiling, enable the global allocator by adding this code to your
//! program:
//! ```
//! use dhat::{Dhat, DhatAlloc};
//!
//! #[global_allocator]
//! static ALLOCATOR: DhatAlloc = DhatAlloc;
//! ```
//! Then add the following code to the very start of your `main` function:
//! ```
//! # use dhat::Dhat;
//! let _dhat = Dhat::start_heap_profiling();
//! ```
//! `DhatAlloc` is slower than the system allocator, so it should only be
//! enabled while profiling.
//!
//! # Usage (ad hoc profiling)
//!
//! [Ad hoc profiling] involves manually annotating hot code points and then
//! aggregating the executed annotations in some fashion.
//!
//! [Ad hoc profiling]: https://github.com/nnethercote/counts/#ad-hoc-profiling
//!
//! To do this, add the following code to the very start of your `main`
//! function:
//!```
//! # use dhat::Dhat;
//! let _dhat = Dhat::start_ad_hoc_profiling();
//! ```
//! Then insert calls like this at points of interest:
//! ```
//! dhat::ad_hoc_event(100);
//! ```
//! For example, imagine you have a hot function that is called from many call
//! sites. You might want to know how often it is called and which other
//! functions called it the most. In that case, you would add a `ad_hoc_event`
//! call to that function, and the data collected by this crate and viewed with
//! DHAT's viewer would show you exactly what you want to know.
//!
//! The meaning of the integer argument to `ad_hoc_event` will depend on
//! exactly what you are measuring. If there is no meaningful weight to give to
//! an event, you can just use `1`.
//!
//! # Running
//!
//! For both heap profiling and ad hoc profiling, the program will run
//! normally. When the `Dhat` value is dropped at the end of `main`, some basic
//! information will be printed to `stderr`. For heap profiling it will look
//! like the following.
//! ```text
//! dhat: Total:     1,256 bytes in 6 blocks
//! dhat: At t-gmax: 1,256 bytes in 6 blocks
//! dhat: At t-end:  1,256 bytes in 6 blocks
//! dhat: The data in dhat-heap.json is viewable with dhat/dh_view.html
//! ```
//! For ad hoc profiling it will look like the following.
//! ```text
//! dhat: Total:     141 units in 11 events
//! dhat: The data in dhat-ad-hoc.json is viewable with dhat/dh_view.html
//! ```
//! A file called `dhat-heap.json` (for heap profiling) or `dhat-ad-hoc.json`
//! (for ad hoc profiling) will be written. It can be viewed in DHAT's viewer.
//!
//! If you don't see this output, it may be because your program called
//! `std::process::exit`, which terminates a program without running any
//! destructors. To work around this, explicitly call `drop` on the `Dhat`
//! value just before the call to `std::process:exit`.
//!
//! # Viewing
//!
//! Open a copy of DHAT's viewer, version 3.17 or later. There are two ways to
//! do this.
//! - Easier: Use the [online version].
//! - Harder: Clone the [Valgrind repository] with `git clone
//!   git://sourceware.org/git/valgrind.git` and open `dhat/dh_view.html`.
//!   (There is no need to build any code in this repository.)
//!
//! [online version]: https://nnethercote.github.io/dh_view/dh_view.html
//! [Valgrind repository]: https://www.valgrind.org/downloads/repository.html
//!
//! Then click on the "Load…" button to load `dhat-heap.json` or
//! `dhat-ad-hoc.json`.
//!
//! DHAT's viewer shows a tree with nodes that look like this.
//! ```text
//! PP 1.1/6 {
//!   Total:     1,024 bytes (81.53%, 3,335,504.89/s) in 1 blocks (16.67%, 3,257.33/s), avg size 1,024 bytes, avg lifetime 61 µs (19.87% of program duration)
//!   Max:       1,024 bytes in 1 blocks, avg size 1,024 bytes
//!   At t-gmax: 1,024 bytes (81.53%) in 1 blocks (16.67%), avg size 1,024 bytes
//!   At t-end:  1,024 bytes (81.53%) in 1 blocks (16.67%), avg size 1,024 bytes
//!   Allocated at {
//!     #1: 0x10c1e4108: <alloc::alloc::Global as core::alloc::AllocRef>::alloc (alloc.rs:203:9)
//!     #2: 0x10c1e4108: alloc::raw_vec::RawVec<T,A>::allocate_in (raw_vec.rs:186:45)
//!     #3: 0x10c1e4108: alloc::raw_vec::RawVec<T,A>::with_capacity_in (raw_vec.rs:161:9)
//!     #4: 0x10c1e4108: alloc::raw_vec::RawVec<T>::with_capacity (raw_vec.rs:92:9)
//!     #5: 0x10c1e4108: alloc::vec::Vec<T>::with_capacity (vec.rs:355:20)
//!     #6: 0x10c1e4108: std::io::buffered::BufWriter<W>::with_capacity (buffered.rs:517:46)
//!     #7: 0x10c1e4108: std::io::buffered::LineWriter<W>::with_capacity (buffered.rs:925:29)
//!     #8: 0x10c1e4108: std::io::buffered::LineWriter<W>::new (buffered.rs:905:9)
//!     #9: 0x10c1e4108: std::io::stdio::stdout::stdout_init (stdio.rs:543:65)
//!     #10: 0x10c1e4108: std::io::lazy::Lazy<T>::init (lazy.rs:57:19)
//!     #11: 0x10c1e4108: std::io::lazy::Lazy<T>::get (lazy.rs:33:18)
//!     #12: 0x10c1e4108: std::io::stdio::stdout (stdio.rs:536:25)
//!     #13: 0x10c1e4ccb: std::io::stdio::print_to::{{closure}} (stdio.rs:890:13)
//!     #14: 0x10c1e4ccb: std::thread::local::LocalKey<T>::try_with (local.rs:265:16)
//!     #15: 0x10c1e4ccb: std::io::stdio::print_to (stdio.rs:879:18)
//!     #16: 0x10c1e4ccb: std::io::stdio::_print (stdio.rs:907:5)
//!     #17: 0x10c0d6826: heap::main (heap.rs:9:5)
//!   }
//! }
//! ```
//! Full details about the output are in the [DHAT documentation].
//!
//! [DHAT documentation]: https://valgrind.org/docs/manual/dh-manual.html
//!
//! Note that DHAT uses the word "block" rather than "allocation" to refer to
//! the memory allocated by a single heap allocation operation.
//!
//! When heap profiling, this crate doesn't track memory accesses (unlike DHAT)
//! and so the "reads" and "writes" measurements are not shown within DHAT's
//! viewer, and "sort metric" views involving reads, writes, or accesses are
//! not available.
//!
//! The backtraces produced by this crate are trimmed to reduce output file
//! sizes and improve readability in DHAT's viewer.
//! - Only one allocation-related frame will be shown at the top of the
//!   backtrace. That frame may be a function within `alloc::alloc`, a function
//!   within this crate, or a global allocation function like `__rg_alloc`.
//! - Common frames at the bottom of backtraces, below `main`, are omitted.

use backtrace::SymbolName;
use lazy_static::lazy_static;
use rustc_hash::FxHashMap;
use serde::Serialize;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::ops::AddAssign;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use thousands::Separable;

lazy_static! {
    static ref TRI_GLOBALS: Mutex<Tri<Globals>> = Mutex::new(Tri::Pre);
}

#[derive(PartialEq)]
enum Tri<T> {
    // Before the first `Dhat` value is created.
    Pre,

    // During the lifetime of the first `Dhat` value.
    During(T),

    // After the lifetime of the first `Dhat` value.
    Post(Stats),
}

impl<T> Tri<T> {
    #[cfg(test)]
    #[track_caller]
    fn as_ref_unwrap(&self) -> &T {
        if let Tri::During(v) = self {
            &v
        } else {
            panic!("bad Tri");
        }
    }
}

// Global state that can be accessed from any thread and is therefore protected
// by a `Mutex`.
struct Globals {
    // When `Globals` is created, which is when `Dhat::start_heap_profiling` or
    // `Dhat::start_ad_hoc_profiling` is called.
    start_instant: Instant,

    // All the `PpInfos` gathered during execution. Elements are never deleted.
    // Each element is referred to by exactly one `Backtrace` from
    // `backtraces`, and referred to by any number of live blocks from
    // `live_blocks`. Storing all the `PpInfos` in a `Vec` is a bit clumsy, but
    // allows multiple references from `backtraces` and `live_blocks` without
    // requiring any unsafety, because the references are just indices rather
    // than `Rc`s or raw pointers or whatever.
    pp_infos: Vec<PpInfo>,

    // Each `Backtrace` is associated with a `PpInfo`. The `usize` is an index
    // into `pp_infos`. Entries are not deleted during execution.
    backtraces: FxHashMap<Backtrace, usize>,

    // Counts for the entire run.
    total_blocks: u64,
    total_bytes: u64,

    // Extra things kept when heap profiling.
    heap: Option<HeapGlobals>,
}

struct HeapGlobals {
    // Each live block is associated with a `PpInfo`. Each key is the address
    // of a live block, and thus actually a `*mut u8`, but we store it as a
    // `usize` because we never dereference it, and using `*mut u8` leads to
    // compile errors because raw pointers don't implement `Send`. An element
    // is deleted when the corresponding allocation is freed.
    live_blocks: FxHashMap<usize, LiveBlock>,

    // Current counts.
    curr_blocks: usize,
    curr_bytes: usize,

    // Counts at the global max, i.e. when `curr_bytes` peaks.
    max_blocks: usize,
    max_bytes: usize,

    // Time of the global max.
    tgmax_instant: Instant,
}

impl Globals {
    fn new(heap: Option<HeapGlobals>) -> Self {
        Self {
            start_instant: Instant::now(),
            pp_infos: Vec::default(),
            backtraces: FxHashMap::default(),
            total_blocks: 0,
            total_bytes: 0,
            heap,
        }
    }

    // Get the PpInfo for this backtrace, creating it if necessary.
    fn get_pp_info<F: FnOnce() -> PpInfo>(&mut self, bt: Backtrace, new: F) -> usize {
        let pp_infos = &mut self.pp_infos;
        *self.backtraces.entry(bt).or_insert_with(|| {
            let pp_info_idx = pp_infos.len();
            pp_infos.push(new());
            pp_info_idx
        })
    }

    fn record_block(&mut self, ptr: *mut u8, pp_info_idx: usize, now: Instant) {
        let h = self.heap.as_mut().unwrap();
        let old = h.live_blocks.insert(
            ptr as usize,
            LiveBlock {
                pp_info_idx,
                allocation_instant: now,
            },
        );
        assert!(matches!(old, None));
    }

    fn update_counts_for_alloc(
        &mut self,
        pp_info_idx: usize,
        size: usize,
        delta: Option<Delta>,
        now: Instant,
    ) {
        self.total_blocks += 1;
        self.total_bytes += size as u64;

        let h = self.heap.as_mut().unwrap();
        if let Some(delta) = delta {
            // realloc
            h.curr_blocks += 0; // unchanged
            h.curr_bytes += delta;
        } else {
            // alloc
            h.curr_blocks += 1;
            h.curr_bytes += size;
        }

        // The use of `>=` not `>` means that if there are multiple equal peaks
        // we record the latest one, like `check_for_global_peak` does.
        if h.curr_bytes >= h.max_bytes {
            h.max_blocks = h.curr_blocks;
            h.max_bytes = h.curr_bytes;
            h.tgmax_instant = now;
        }

        self.pp_infos[pp_info_idx].update_counts_for_alloc(size, delta);
    }

    fn update_counts_for_dealloc(
        &mut self,
        pp_info_idx: usize,
        size: usize,
        alloc_duration: Duration,
    ) {
        let h = self.heap.as_mut().unwrap();
        h.curr_blocks -= 1;
        h.curr_bytes -= size;

        self.pp_infos[pp_info_idx].update_counts_for_dealloc(size, alloc_duration);
    }

    fn update_counts_for_ad_hoc_event(&mut self, weight: usize) {
        assert!(self.heap.is_none());
        self.total_blocks += 1;
        self.total_bytes += weight as u64;
    }

    // If we are at peak memory, update `at_tgmax_{blocks,bytes}` in all
    // `PpInfo`s. This is somewhat expensive so we avoid calling it on every
    // allocation; instead we call it upon a deallocation (when we might be
    // coming down from a global peak) and at termination (when we might be at
    // a global peak).
    fn check_for_global_peak(&mut self) {
        let h = self.heap.as_mut().unwrap();
        if h.curr_bytes == h.max_bytes {
            // It's a peak. (If there are multiple equal peaks we record the
            // latest one.) Record it in every PpInfo.
            for pp_info in self.pp_infos.iter_mut() {
                let h = pp_info.heap.as_mut().unwrap();
                h.at_tgmax_blocks = h.curr_blocks;
                h.at_tgmax_bytes = h.curr_bytes;
            }
        }
    }

    fn get_stats(&self) -> Stats {
        Stats {
            total_blocks: self.total_blocks,
            total_bytes: self.total_bytes,
            heap: self.heap.as_ref().map(|heap| HeapStats {
                curr_blocks: heap.curr_blocks,
                curr_bytes: heap.curr_bytes,
                max_blocks: heap.max_blocks,
                max_bytes: heap.max_bytes,
            }),
        }
    }
}

impl HeapGlobals {
    fn new() -> Self {
        Self {
            live_blocks: FxHashMap::default(),
            curr_blocks: 0,
            curr_bytes: 0,
            max_blocks: 0,
            max_bytes: 0,
            tgmax_instant: Instant::now(),
        }
    }
}

struct PpInfo {
    // The total number of blocks and bytes allocated by this PP.
    total_blocks: u64,
    total_bytes: u64,

    heap: Option<HeapPpInfo>,
}

#[derive(Default)]
struct HeapPpInfo {
    // The current number of blocks and bytes allocated by this PP.
    curr_blocks: usize,
    curr_bytes: usize,

    // The number of blocks and bytes at the PP max, i.e. when this PP's
    // `curr_bytes` peaks.
    max_blocks: usize,
    max_bytes: usize,

    // The number of blocks and bytes at the global max, i.e. when
    // `Globals::curr_bytes` peaks.
    at_tgmax_blocks: usize,
    at_tgmax_bytes: usize,

    // Total lifetimes of all blocks allocated by this PP. Includes blocks
    // explicitly freed and blocks implicitly freed at termination.
    total_lifetimes_duration: Duration,
}

impl PpInfo {
    fn new_heap() -> Self {
        Self {
            total_blocks: 0,
            total_bytes: 0,
            heap: Some(HeapPpInfo::default()),
        }
    }

    fn new_ad_hoc() -> Self {
        Self {
            total_blocks: 0,
            total_bytes: 0,
            heap: None,
        }
    }

    fn update_counts_for_alloc(&mut self, size: usize, delta: Option<Delta>) {
        self.total_blocks += 1;
        self.total_bytes += size as u64;

        let h = self.heap.as_mut().unwrap();
        if let Some(delta) = delta {
            // realloc
            h.curr_blocks += 0; // unchanged
            h.curr_bytes += delta;
        } else {
            // alloc
            h.curr_blocks += 1;
            h.curr_bytes += size;
        }

        // The use of `>=` not `>` means that if there are multiple equal peaks
        // we record the latest one, like `check_for_global_peak` does.
        if h.curr_bytes >= h.max_bytes {
            h.max_blocks = h.curr_blocks;
            h.max_bytes = h.curr_bytes;
        }
    }

    fn update_counts_for_dealloc(&mut self, size: usize, alloc_duration: Duration) {
        let h = self.heap.as_mut().unwrap();
        h.curr_blocks -= 1;
        h.curr_bytes -= size;
        h.total_lifetimes_duration += alloc_duration;
    }

    fn update_counts_for_ad_hoc_event(&mut self, weight: usize) {
        assert!(self.heap.is_none());
        self.total_blocks += 1;
        self.total_bytes += weight as u64;
    }
}

struct LiveBlock {
    // The index of the PpInfo for this block.
    pp_info_idx: usize,

    // When the block was allocated.
    allocation_instant: Instant,
}

// We record info about allocations and deallocations. A wrinkle: the recording
// done may trigger additional allocations. We must ignore these because (a)
// they're part of `dhat`'s execution, not the original program's execution,
// and (b) they would be intercepted and trigger additional allocations, which
// would be intercepted and trigger additional allocations, and so on, leading
// to infinite loops.
//
// This function runs `f1` if we are ignoring allocations, and `f2` otherwise.
//
// WARNING: This function must be used for any code within this crate that can
// trigger allocations.
fn if_ignoring_allocs_else<F1, F2, R>(f1: F1, f2: F2) -> R
where
    F1: FnOnce() -> R,
    F2: FnOnce() -> R,
{
    thread_local!(static IGNORE_ALLOCS: Cell<bool> = Cell::new(false));

    /// If `F` panics, then `ResetOnDrop` will still reset `IGNORE_ALLOCS`
    /// so that it can be used again.
    struct ResetOnDrop;

    impl Drop for ResetOnDrop {
        fn drop(&mut self) {
            IGNORE_ALLOCS.with(|b| b.set(false));
        }
    }

    if IGNORE_ALLOCS.with(|b| b.replace(true)) {
        f1()
    } else {
        let _reset_on_drop = ResetOnDrop;
        f2()
    }
}

/// A type whose scope dictates the start and end of profiling.
///
/// When the first value of this type is dropped, profiling data is written to
/// file. Only one value of this type should be created; if subsequent values
/// of this type are created they will have no effect.
#[derive(Debug)]
pub struct Dhat {
    start_bt: Backtrace,
}

impl Dhat {
    /// Initiate allocation profiling. This should be the first thing in
    /// `main`, and its result should be assigned to a variable whose scope
    /// ends at the end of `main`.
    pub fn start_heap_profiling() -> Self {
        Dhat::start_impl(Some(HeapGlobals::new()))
    }

    /// Initiate ad hoc profiling. This should be the first thing in `main`,
    /// and its result should be assigned to a variable whose scope ends at the
    /// end of `main`.
    pub fn start_ad_hoc_profiling() -> Self {
        Dhat::start_impl(None)
    }

    fn start_impl(h: Option<HeapGlobals>) -> Self {
        if_ignoring_allocs_else(
            || panic!("start_impl"),
            || {
                let tri: &mut Tri<Globals> = &mut TRI_GLOBALS.lock().unwrap();
                if let Tri::Pre = tri {
                    *tri = Tri::During(Globals::new(h));
                } else {
                    eprintln!("dhat: error: A second `Dhat` object was initialized");
                }
                let start_bt = Backtrace(backtrace::Backtrace::new_unresolved());
                Dhat { start_bt }
            },
        )
    }
}

impl Drop for Dhat {
    fn drop(&mut self) {
        finish(self);
    }
}

/// A global allocator that tracks allocations and deallocations on behalf of
/// the `Dhat` type.
#[derive(Debug)]
pub struct DhatAlloc;

unsafe impl GlobalAlloc for DhatAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if_ignoring_allocs_else(
            || System.alloc(layout),
            || {
                let tri: &mut Tri<Globals> = &mut TRI_GLOBALS.lock().unwrap();
                let ptr = System.alloc(layout);
                if ptr.is_null() {
                    return ptr;
                }

                if let Tri::During(g @ Globals { heap: Some(_), .. }) = tri {
                    let size = layout.size();
                    let bt = Backtrace(backtrace::Backtrace::new_unresolved());
                    let pp_info_idx = g.get_pp_info(bt, PpInfo::new_heap);

                    let now = Instant::now();
                    g.record_block(ptr, pp_info_idx, now);
                    g.update_counts_for_alloc(pp_info_idx, size, None, now);
                }
                ptr
            },
        )
    }

    unsafe fn realloc(&self, old_ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if_ignoring_allocs_else(
            || System.realloc(old_ptr, layout, new_size),
            || {
                let tri: &mut Tri<Globals> = &mut TRI_GLOBALS.lock().unwrap();
                let new_ptr = System.realloc(old_ptr, layout, new_size);
                if new_ptr.is_null() {
                    return new_ptr;
                }

                if let Tri::During(g @ Globals { heap: Some(_), .. }) = tri {
                    let old_size = layout.size();
                    let delta = Delta::new(old_size, new_size);

                    if delta.shrinking {
                        // Total bytes is coming down from a possible peak.
                        g.check_for_global_peak();
                    }

                    // Remove the record of the existing live block and get the
                    // `PpInfo`. If it's not in the live block table, it must
                    // have been allocated before `TRI_GLOBALS` was set up, and
                    // we treat it like an `alloc`.
                    let h = g.heap.as_mut().unwrap();
                    let live_block = h.live_blocks.remove(&(old_ptr as usize));
                    let (pp_info_idx, delta) = if let Some(live_block) = live_block {
                        (live_block.pp_info_idx, Some(delta))
                    } else {
                        let bt = Backtrace(backtrace::Backtrace::new_unresolved());
                        let pp_info_idx = g.get_pp_info(bt, PpInfo::new_heap);
                        (pp_info_idx, None)
                    };

                    let now = Instant::now();
                    g.record_block(new_ptr, pp_info_idx, now);
                    g.update_counts_for_alloc(pp_info_idx, new_size, delta, now);
                }
                new_ptr
            },
        )
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if_ignoring_allocs_else(
            || System.dealloc(ptr, layout),
            || {
                let tri: &mut Tri<Globals> = &mut TRI_GLOBALS.lock().unwrap();
                System.dealloc(ptr, layout);

                if let Tri::During(g @ Globals { heap: Some(_), .. }) = tri {
                    let size = layout.size();

                    // Remove the record of the live block and get the
                    // `PpInfo`. If it's not in the live block table, it must
                    // have been allocated before `TRI_GLOBALS` was set up, and
                    // we just ignore it.
                    let h = g.heap.as_mut().unwrap();
                    if let Some(LiveBlock {
                        pp_info_idx,
                        allocation_instant,
                    }) = h.live_blocks.remove(&(ptr as usize))
                    {
                        // Total bytes is coming down from a possible peak.
                        g.check_for_global_peak();

                        let alloc_duration = allocation_instant.elapsed();
                        g.update_counts_for_dealloc(pp_info_idx, size, alloc_duration);
                    }
                }
            },
        );
    }
}

/// Register an event during ad hoc profiling. Has no effect unless a `Dhat`
/// value that was created with `Dhat::start_ad_hoc_profiling` is in scope. The
/// meaning of the weight argument is determined by the user.
pub fn ad_hoc_event(weight: usize) {
    if_ignoring_allocs_else(
        || panic!("ad_hoc_event"),
        || {
            let tri: &mut Tri<Globals> = &mut TRI_GLOBALS.lock().unwrap();
            if let Tri::During(g @ Globals { heap: None, .. }) = tri {
                let bt = Backtrace(backtrace::Backtrace::new_unresolved());

                let pp_info_idx = g.get_pp_info(bt, PpInfo::new_ad_hoc);

                // Update counts.
                g.pp_infos[pp_info_idx].update_counts_for_ad_hoc_event(weight);
                g.update_counts_for_ad_hoc_event(weight);
            }
        },
    );
}

// Finish tracking allocations and deallocations, print a summary message
// to `stderr` and write output to `dhat-alloc.json`. If called more than
// once, the second and subsequent calls will print an error message to
// `stderr` and return `Ok(())`.
//
// Note: this is only separate from `drop` for testing purposes. If
// `TRI_GLOBALS` is `Tri::During(g)` on entry then the return value will be
// `Some(g)`, otherwise it will be `None`.
fn finish(dhat: &mut Dhat) -> Option<Globals> {
    let mut filename = None;

    let r: std::io::Result<Option<Globals>> = if_ignoring_allocs_else(
        || panic!("finish"),
        || {
            let tri: &mut Tri<Globals> = &mut TRI_GLOBALS.lock().unwrap();
            let stats = match tri {
                Tri::Pre => unreachable!(),
                Tri::During(g) => g.get_stats(),
                Tri::Post(_) => {
                    // Don't print an error message because `Dhat::new` will have
                    // already printed one.
                    return Ok(None);
                }
            };
            let tri = std::mem::replace(tri, Tri::Post(stats));
            let mut g = if let Tri::During(g) = tri {
                g
            } else {
                unreachable!()
            };

            let now = Instant::now();

            if g.heap.is_some() {
                // Total bytes is at a possible peak.
                g.check_for_global_peak();

                let h = g.heap.as_ref().unwrap();

                // Account for the lifetimes of all remaining live blocks.
                for &LiveBlock {
                    pp_info_idx,
                    allocation_instant,
                } in h.live_blocks.values()
                {
                    g.pp_infos[pp_info_idx]
                        .heap
                        .as_mut()
                        .unwrap()
                        .total_lifetimes_duration += now.duration_since(allocation_instant);
                }
            }

            // We give each unique frame an index into `ftbl`, starting with 0
            // for the special frame "[root]".
            let mut ftbl_indices: FxHashMap<String, usize> = FxHashMap::default();
            ftbl_indices.insert("[root]".to_string(), 0);
            let mut next_ftbl_idx = 1;

            // Because `g` is being consumed, we can consume `g.backtraces` and
            // replace it with an empty `FxHashMap`. (This is necessary because
            // we modify the *keys* here with `resolve`, which isn't allowed
            // with a non-consuming iterator.)
            let pps: Vec<_> = std::mem::take(&mut g.backtraces)
                .into_iter()
                .map(|(mut bt, pp_info_idx)| {
                    // Do the potentially expensive debug info lookups to get
                    // symbol names, line numbers, etc.
                    bt.0.resolve();

                    let first_symbol_to_show = first_symbol_to_show(&bt);
                    let last_frame_ip_to_show = last_frame_ip_to_show(&bt, &dhat.start_bt);

                    // Determine the frame indices for this backtrace. This
                    // involves getting the string for each frame and adding a
                    // new entry to `ftbl_indices` if it hasn't been seen
                    // before.
                    let mut fs = vec![];
                    let mut i = 0;
                    'outer: for frame in bt.0.frames().iter() {
                        for symbol in frame.symbols().iter() {
                            i += 1;
                            if (i - 1) < first_symbol_to_show {
                                continue;
                            }
                            let s = format!(
                                // Use `{:#}` rather than `{}` to print the
                                // "alternate" form of the symbol name, which
                                // omits the trailing hash (e.g.
                                // `::ha68e4508a38cc95a`).
                                "{:?}: {:#} ({:#}:{}:{})",
                                frame.ip(),
                                symbol.name().unwrap_or_else(|| SymbolName::new(b"???")),
                                // We have the full path, but that's typically
                                // very long and clogs up the output greatly.
                                // So just use the filename, which is usually
                                // good enough.
                                symbol
                                    .filename()
                                    .and_then(|path| path.file_name())
                                    .and_then(|file_name| file_name.to_str())
                                    .unwrap_or("???"),
                                symbol.lineno().unwrap_or(0),
                                symbol.colno().unwrap_or(0),
                            );

                            let &mut ftbl_idx = ftbl_indices.entry(s).or_insert_with(|| {
                                next_ftbl_idx += 1;
                                next_ftbl_idx - 1
                            });
                            fs.push(ftbl_idx);

                            if Some(frame.ip()) == last_frame_ip_to_show {
                                break 'outer;
                            }
                        }
                    }

                    PpInfoJson::new(&g.pp_infos[pp_info_idx], fs)
                })
                .collect();

            // We pre-allocate `ftbl` with empty strings, and then fill it in.
            let mut ftbl = vec![String::new(); ftbl_indices.len()];
            for (frame, ftbl_idx) in ftbl_indices.into_iter() {
                ftbl[ftbl_idx] = frame;
            }

            let h = g.heap.as_ref();
            let is_heap = h.is_some();
            let json = DhatJson {
                dhatFileVersion: 2,
                mode: if is_heap { "rust-heap" } else { "rust-ad-hoc" },
                verb: "Allocated",
                bklt: is_heap,
                bkacc: false,
                bu: if is_heap { None } else { Some("unit") },
                bsu: if is_heap { None } else { Some("units") },
                bksu: if is_heap { None } else { Some("events") },
                tu: "µs",
                Mtu: "s",
                tuth: if is_heap { Some(10) } else { None },
                cmd: std::env::args().collect::<Vec<_>>().join(" "),
                pid: std::process::id(),
                tg: h.map(|h| {
                    h.tgmax_instant
                        .saturating_duration_since(g.start_instant)
                        .as_micros()
                }),
                te: now.duration_since(g.start_instant).as_micros(),
                pps,
                ftbl,
            };

            eprintln!(
                "dhat: Total:     {} {} in {} {}",
                g.total_bytes.separate_with_commas(),
                json.bsu.unwrap_or("bytes"),
                g.total_blocks.separate_with_commas(),
                json.bksu.unwrap_or("blocks"),
            );
            if let Some(h) = &g.heap {
                eprintln!(
                    "dhat: At t-gmax: {} bytes in {} blocks",
                    h.max_bytes.separate_with_commas(),
                    h.max_blocks.separate_with_commas(),
                );
                eprintln!(
                    "dhat: At t-end:  {} bytes in {} blocks",
                    h.curr_bytes.separate_with_commas(),
                    h.curr_blocks.separate_with_commas(),
                );
            }

            // `to_writer` produces JSON that is compact, and
            // `to_writer_pretty` produces JSON that is readable. Ideally we'd
            // have something between the two (e.g. 1-space indents instead of
            // 2-space, no spaces after `:`, `fs` arrays on a single line) more
            // like what DHAT produces. But in the absence of such an
            // intermediate option, readability trumps compactness.
            filename = Some(if g.heap.is_some() {
                "dhat-heap.json"
            } else {
                "dhat-ad-hoc.json"
            });
            let filename = filename.unwrap();
            let file = File::create(filename)?;
            serde_json::to_writer_pretty(&file, &json)?;

            eprintln!(
                "dhat: The data in {} is viewable with dhat/dh_view.html",
                filename
            );

            Ok(Some(g))
        },
    );

    match r {
        Ok(globals) => globals,
        Err(e) => {
            eprintln!(
                "dhat: error: Writing to {} failed: {}",
                filename.unwrap(),
                e
            );
            None
        }
    }
}

// The top frame symbols in a backtrace vary significantly (depending on build
// configuration, platform, and program point) but they typically look
// something like this:
// - backtrace::backtrace::libunwind::trace
// - backtrace::backtrace::trace_unsynchronized
// - backtrace::backtrace::trace
// - backtrace::capture::Backtrace::create
// - backtrace::capture::Backtrace::new_unresolved
// - <dhat::DhatAlloc as core::alloc::global::GlobalAlloc>::alloc::{{closure}}
// - dhat::if_ignoring_allocs_else::{{closure}}
// - std::thread::local::LocalKey<T>::try_with
// - std::thread::local::LocalKey<T>::with
// - dhat::if_ignoring_allocs_else
// - <dhat::DhatAlloc as core::alloc::global::GlobalAlloc>::alloc
// - __rg_alloc
// - alloc::alloc::alloc
// - alloc::alloc::Global::alloc_impl
// - <alloc::alloc::Global as core::alloc::AllocRef>::alloc
//
// Such frames are boring and clog up the output. So we scan backwards for the
// first frame that looks like it comes from allocator code or this crate's
// code. We keep that frame, but discard everything before it. If we don't find
// any such frames, we show from frame 0, i.e. all frames.
fn first_symbol_to_show(bt: &Backtrace) -> usize {
    // Get the symbols into a vector so we can reverse iterate over them.
    let symbols: Vec<_> =
        bt.0.frames()
            .iter()
            .map(|f| f.symbols().iter())
            .flatten()
            .collect();

    for (i, symbol) in symbols.iter().enumerate().rev() {
        if let Some(s) = symbol.name().map(|name| name.to_string()) {
            // Examples of symbols that this search will match:
            // - <dhat::DhatAlloc as core::alloc::global::GlobalAlloc>::alloc
            // - <alloc::alloc::Global as core::alloc::AllocRef>::{alloc,grow}
            // - __rg_{alloc,realloc}
            // - alloc::alloc::{alloc,realloc}
            // - alloc::alloc::exchange_malloc
            if s.starts_with("alloc::alloc::")
                || s.starts_with("<alloc::alloc::")
                || s.starts_with("dhat::")
                || s.starts_with("<dhat::")
                || s.starts_with("__rg_")
            {
                return i;
            }
        }
    }
    0
}

// The bottom frame symbols in a backtrace (those below `main`) are typically
// the same, and look something like this:
// - core::ops::function::FnOnce::call_once (function.rs:227:5)
// - std::sys_common::backtrace::__rust_begin_short_backtrace (backtrace.rs:137:18)
// - std::rt::lang_start::{{closure}} (rt.rs:66:18)
// - core::ops::function::impls::<impl core::ops::function::FnOnce<A> for &F>::call_once (function.rs:259:13)
// - std::panicking::try::do_call (panicking.rs:373:40)
// - std::panicking::try (panicking.rs:337:19)
// - std::panic::catch_unwind (panic.rs:379:14)
// - std::rt::lang_start_internal (rt.rs:51:25)
// - std::rt::lang_start (rt.rs:65:5)
// - _main (???:0:0)
//
// Such frames are boring and clog up the output. So we compare the bottom
// frames with those obtained when the `Dhat` value was created. Those that
// overlap in the two cases are the common, uninteresting ones, and we discard
// them.
fn last_frame_ip_to_show(bt: &Backtrace, start_bt: &Backtrace) -> Option<*mut std::ffi::c_void> {
    let bt_frames = bt.0.frames();
    let start_bt_frames = start_bt.0.frames();
    let (mut i, mut j) = (bt_frames.len() - 1, start_bt_frames.len() - 1);
    loop {
        if bt_frames[i].ip() != start_bt_frames[j].ip() {
            return Some(bt_frames[i].ip());
        }
        if i == 0 || j == 0 {
            return None;
        }
        i -= 1;
        j -= 1;
    }
}

// A wrapper for `backtrace::Backtrace` that implements `Eq` and `Hash`, which
// only look at the frame IPs. This assumes that any two
// `backtrace::Backtrace`s with the same frame IPs are equivalent.
#[derive(Debug)]
struct Backtrace(backtrace::Backtrace);

impl PartialEq for Backtrace {
    fn eq(&self, other: &Self) -> bool {
        let mut frames1 = self.0.frames().iter();
        let mut frames2 = other.0.frames().iter();
        loop {
            let ip1 = frames1.next().map(|f| f.ip());
            let ip2 = frames2.next().map(|f| f.ip());
            if ip1 != ip2 {
                return false;
            }
            if ip1 == None {
                return true;
            }
            // Otherwise, continue.
        }
    }
}

impl Eq for Backtrace {}

impl Hash for Backtrace {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for frame in self.0.frames().iter() {
            frame.ip().hash(state);
        }
    }
}

/// Some stats about execution. For testing purposes, subject to change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stats {
    /// Number of blocks (or events, for ad hoc profiling) for the entire run.
    pub total_blocks: u64,

    /// Number of bytes (or units, for ad hoc profiling) for the entire run.
    pub total_bytes: u64,

    /// Additional stats for heap profiling.
    pub heap: Option<HeapStats>,
}

/// Some heap stats about execution. For testing purposes, subject to change.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeapStats {
    /// Number of blocks currently allocated.
    pub curr_blocks: usize,

    /// Number of bytes currently allocated.
    pub curr_bytes: usize,

    /// Number of blocks allocated at the global peak.
    pub max_blocks: usize,

    /// Number of bytes allocated at the global peak.
    pub max_bytes: usize,
}

/// Get current stats. Returns `None` if called before
/// `Dhat::start_heap_profiling` or `Dhat::start_ad_hoc_profiling` is called.
pub fn get_stats() -> Option<Stats> {
    if_ignoring_allocs_else(
        || panic!("get_stats"),
        || {
            let tri: &mut Tri<Globals> = &mut TRI_GLOBALS.lock().unwrap();
            match tri {
                Tri::Pre => None,
                Tri::During(g) => Some(g.get_stats()),
                Tri::Post(stats) => Some(stats.clone()),
            }
        },
    )
}

// A Rust representation of DHAT's JSON file format, which is described in
// comments in dhat/dh_main.c in Valgrind's source code.
//
// Building this structure in order to serialize does take up some memory. We
// could instead stream the JSON output directly to file ourselves. This would
// be more efficient but make the code uglier.
#[derive(Serialize)]
#[allow(non_snake_case)]
struct DhatJson {
    dhatFileVersion: u32,
    mode: &'static str,
    verb: &'static str,
    bklt: bool,
    bkacc: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    bu: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bsu: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bksu: Option<&'static str>,
    tu: &'static str,
    Mtu: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    tuth: Option<usize>,
    cmd: String,
    pid: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    tg: Option<u128>,
    te: u128,
    pps: Vec<PpInfoJson>,
    ftbl: Vec<String>,
}

// A Rust representation of a PpInfo within DHAT's JSON file format.
#[derive(Serialize)]
struct PpInfoJson {
    // `PpInfo::total_bytes and `PpInfo::total_blocks.
    tb: u64,
    tbk: u64,

    // Derived from `PpInfo::total_lifetimes_duration`.
    #[serde(skip_serializing_if = "Option::is_none")]
    tl: Option<u128>,

    // `PpInfo::max_bytes` and `PpInfo::max_blocks`.
    #[serde(skip_serializing_if = "Option::is_none")]
    mb: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mbk: Option<usize>,

    // `PpInfo::at_tgmax_bytes` and `PpInfo::at_tgmax_blocks`.
    #[serde(skip_serializing_if = "Option::is_none")]
    gb: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gbk: Option<usize>,

    // `PpInfo::curr_bytes` and `PpInfo::curr_blocks` (at termination, i.e.
    // "end").
    #[serde(skip_serializing_if = "Option::is_none")]
    eb: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ebk: Option<usize>,

    // Frames. Each element is an index into `ftbl`.
    fs: Vec<usize>,
}

impl PpInfoJson {
    fn new(pp_info: &PpInfo, fs: Vec<usize>) -> Self {
        if let Some(h) = &pp_info.heap {
            Self {
                tb: pp_info.total_bytes,
                tbk: pp_info.total_blocks,
                tl: Some(h.total_lifetimes_duration.as_micros()),
                mb: Some(h.max_bytes),
                mbk: Some(h.max_blocks),
                gb: Some(h.at_tgmax_bytes),
                gbk: Some(h.at_tgmax_blocks),
                eb: Some(h.curr_bytes),
                ebk: Some(h.curr_blocks),
                fs,
            }
        } else {
            Self {
                tb: pp_info.total_bytes,
                tbk: pp_info.total_blocks,
                tl: None,
                mb: None,
                mbk: None,
                gb: None,
                gbk: None,
                eb: None,
                ebk: None,
                fs,
            }
        }
    }
}

// A change in size. Used for `realloc`.
#[derive(Clone, Copy)]
struct Delta {
    shrinking: bool,
    size: usize,
}

impl Delta {
    fn new(old_size: usize, new_size: usize) -> Delta {
        if new_size < old_size {
            Delta {
                shrinking: true,
                size: old_size - new_size,
            }
        } else {
            Delta {
                shrinking: false,
                size: new_size - old_size,
            }
        }
    }
}

impl AddAssign<Delta> for usize {
    fn add_assign(&mut self, rhs: Delta) {
        if rhs.shrinking {
            *self -= rhs.size;
        } else {
            *self += rhs.size;
        }
    }
}

impl AddAssign<Delta> for u64 {
    fn add_assign(&mut self, rhs: Delta) {
        if rhs.shrinking {
            *self -= rhs.size as u64;
        } else {
            *self += rhs.size as u64;
        }
    }
}

#[cfg(test)]
mod tests;

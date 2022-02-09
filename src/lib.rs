#![deny(missing_docs)]
#![deny(rustdoc::missing_doc_code_examples)]
#![deny(missing_debug_implementations)]

//! This crate provides heap profiling and [ad hoc profiling] capabilities to
//! Rust programs, similar to those provided by [DHAT].
//!
//! [ad hoc profiling]: https://github.com/nnethercote/counts/#ad-hoc-profiling
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
//! `dhat` also supports *heap usage testing*, where you can write tests and
//! then check that they allocated as much heap memory as you expected. This
//! can be useful for performance-sensitive code.
//!
//! # Motivation
//!
//! DHAT is a powerful heap profiler that comes with Valgrind. This crate is a
//! related but alternative choice for heap profiling Rust programs. DHAT and
//! this crate have the following differences.
//! - This crate works on any platform, while DHAT only works on some platforms
//!   (Linux, mostly). (Note that DHAT's viewer is just HTML+JS+CSS and should
//!   work in any modern web browser on any platform.)
//! - This crate typically causes a smaller slowdown than DHAT.
//! - This crate requires some modifications to a program's source code and
//!   recompilation, while DHAT does not.
//! - This crate cannot track memory accesses the way DHAT does, because it does
//!   not instrument all memory loads and stores.
//! - This crate does not provide profiling of copy functions such as `memcpy`
//!   and `strcpy`, unlike DHAT.
//! - The backtraces produced by this crate may be better than those produced
//!   by DHAT.
//! - DHAT measures a program's entire execution, but this crate only measures
//!   what happens within `main`. It will miss the small number of allocations
//!   that occur before or after `main`, within the Rust runtime.
//! - This crate enables heap usage testing.
//!
//! # Configuration (profiling and testing)
//!
//! In your `Cargo.toml` file, as well as specifying `dhat` as a dependency,
//! you should (a) enable source line debug info, and (b) create a feature or
//! two that lets you easily switch profiling on and off:
//! ```toml
//! [profile.release]
//! debug = 1
//!
//! [features]
//! dhat-heap = []    # if you are doing heap profiling
//! dhat-ad-hoc = []  # if you are doing ad hoc profiling
//! ```
//! You should only use `dhat` in release builds. Debug builds are too slow to
//! be useful.
//!
//! # Setup (heap profiling)
//!
//! For heap profiling, enable the global allocator by adding this code to your
//! program:
//! ```
//! # // Tricky: comment out the `cfg` so it shows up in docs but the following
//! # // line is still tsted by `cargo test`.
//! # /*
//! #[cfg(feature = "dhat-heap")]
//! # */
//! #[global_allocator]
//! static ALLOC: dhat::Alloc = dhat::Alloc;
//! ```
//! Then add the following code to the very start of your `main` function:
//! ```
//! # // Tricky: comment out the `cfg` so it shows up in docs but the following
//! # // line is still tsted by `cargo test`.
//! # /*
//! #[cfg(feature = "dhat-heap")]
//! # */
//! let _profiler = dhat::Profiler::new_heap();
//! ```
//! Then run this command to enable heap profiling during the lifetime of the
//! [`Profiler`] instance:
//! ```text
//! cargo run --features dhat-heap
//! ```
//! [`dhat::Alloc`](Alloc) is slower than the normal allocator, so it should
//! only be enabled while profiling.
//!
//! # Setup (ad hoc profiling)
//!
//! [Ad hoc profiling] involves manually annotating hot code points and then
//! aggregating the executed annotations in some fashion.
//!
//! [Ad hoc profiling]: https://github.com/nnethercote/counts/#ad-hoc-profiling
//!
//! To do this, add the following code to the very start of your `main`
//! function:
//!```
//! # // Tricky: comment out the `cfg` so it shows up in docs but the following
//! # // line is still tsted by `cargo test`.
//! # /*
//! #[cfg(feature = "dhat-ad-hoc")]
//! # */
//! let _profiler = dhat::Profiler::new_ad_hoc();
//! ```
//! Then insert calls like this at points of interest:
//! ```
//! # // Tricky: comment out the `cfg` so it shows up in docs but the following
//! # // line is still tsted by `cargo test`.
//! # /*
//! #[cfg(feature = "dhat-ad-hoc")]
//! # */
//! dhat::ad_hoc_event(100);
//! ```
//! Then run this command to enable ad hoc profiling during the lifetime of the
//! [`Profiler`] instance:
//! ```text
//! cargo run --features dhat-ad-hoc
//! ```
//! For example, imagine you have a hot function that is called from many call
//! sites. You might want to know how often it is called and which other
//! functions called it the most. In that case, you would add an
//! [`ad_hoc_event`] call to that function, and the data collected by this
//! crate and viewed with DHAT's viewer would show you exactly what you want to
//! know.
//!
//! The meaning of the integer argument to `ad_hoc_event` will depend on
//! exactly what you are measuring. If there is no meaningful weight to give to
//! an event, you can just use `1`.
//!
//! # Running
//!
//! For both heap profiling and ad hoc profiling, the program will run more
//! slowly than normal. The exact slowdown is hard to predict because it
//! depends greatly on the program being profiled, but it can be large. (Even
//! more so on Windows, because backtrace gathering can be drastically slower
//! on Windows than on other platforms.)
//!
//! When the [`Profiler`] is dropped at the end of `main`, some basic
//! information will be printed to `stderr`. For heap profiling it will look
//! like the following.
//! ```text
//! dhat: Total:     1,256 bytes in 6 blocks
//! dhat: At t-gmax: 1,256 bytes in 6 blocks
//! dhat: At t-end:  1,256 bytes in 6 blocks
//! dhat: The data has been saved to dhat-heap.json, and is viewable with dhat/dh_view.html
//! ```
//! ("Blocks" is a synonym for "allocations".)
//!
//! For ad hoc profiling it will look like the following.
//! ```text
//! dhat: Total:     141 units in 11 events
//! dhat: The data has been saved to dhat-ad-hoc.json, and is viewable with dhat/dh_view.html
//! ```
//! A file called `dhat-heap.json` (for heap profiling) or `dhat-ad-hoc.json`
//! (for ad hoc profiling) will be written. It can be viewed in DHAT's viewer.
//!
//! If you don't see this output, it may be because your program called
//! [`std::process::exit`], which exits a program without running any
//! destructors. To work around this, explicitly call `drop` on the
//! [`Profiler`] value just before exiting.
//!
//! When doing heap profiling, if you unexpectedly see zero allocations in the
//! output it may be because you forgot to set [`dhat::Alloc`](Alloc) as the
//! global allocator.
//!
//! When doing heap profiling it is recommended that the lifetime of the
//! [`Profiler`] value cover all of `main`. But it is still possible for
//! allocations and deallocations to occur outside of its lifetime. Such cases
//! are handled in the following ways.
//! - Allocated before, untouched within: ignored.
//! - Allocated before, freed within: ignored.
//! - Allocated before, reallocated within: treated like a new allocation
//!   within.
//! - Allocated after: ignored.
//!
//! These cases are not ideal, but it is impossible to do better. `dhat`
//! deliberately provides no way to reset the heap profiling state mid-run
//! precisely because it leaves open the possibility of many such occurrences.
//!
//! # Viewing
//!
//! Open a copy of DHAT's viewer, version 3.17 or later. There are two ways to
//! do this.
//! - Easier: Use the [online version].
//! - Harder: Clone the [Valgrind repository] with `git clone
//!   git://sourceware.org/git/valgrind.git` and open `dhat/dh_view.html`.
//!   There is no need to build any code in this repository.
//!
//! [online version]: https://nnethercote.github.io/dh_view/dh_view.html
//! [Valgrind repository]: https://www.valgrind.org/downloads/repository.html
//!
//! Then click on the "Load…" button to load `dhat-heap.json` or
//! `dhat-ad-hoc.json`.
//!
//! DHAT's viewer shows a tree with nodes that look like this.
//! ```text
//! PP 1.1/2 {
//!   Total:     1,024 bytes (98.46%, 14,422,535.21/s) in 1 blocks (50%, 14,084.51/s), avg size 1,024 bytes, avg lifetime 35 µs (49.3% of program duration)
//!   Max:       1,024 bytes in 1 blocks, avg size 1,024 bytes
//!   At t-gmax: 1,024 bytes (98.46%) in 1 blocks (50%), avg size 1,024 bytes
//!   At t-end:  1,024 bytes (100%) in 1 blocks (100%), avg size 1,024 bytes
//!   Allocated at {
//!     #1: 0x10ae8441b: <alloc::alloc::Global as core::alloc::Allocator>::allocate (alloc/src/alloc.rs:226:9)
//!     #2: 0x10ae8441b: alloc::raw_vec::RawVec<T,A>::allocate_in (alloc/src/raw_vec.rs:207:45)
//!     #3: 0x10ae8441b: alloc::raw_vec::RawVec<T,A>::with_capacity_in (alloc/src/raw_vec.rs:146:9)
//!     #4: 0x10ae8441b: alloc::vec::Vec<T,A>::with_capacity_in (src/vec/mod.rs:609:20)
//!     #5: 0x10ae8441b: alloc::vec::Vec<T>::with_capacity (src/vec/mod.rs:470:9)
//!     #6: 0x10ae8441b: std::io::buffered::bufwriter::BufWriter<W>::with_capacity (io/buffered/bufwriter.rs:115:33)
//!     #7: 0x10ae8441b: std::io::buffered::linewriter::LineWriter<W>::with_capacity (io/buffered/linewriter.rs:109:29)
//!     #8: 0x10ae8441b: std::io::buffered::linewriter::LineWriter<W>::new (io/buffered/linewriter.rs:89:9)
//!     #9: 0x10ae8441b: std::io::stdio::stdout::{{closure}} (src/io/stdio.rs:680:58)
//!     #10: 0x10ae8441b: std::lazy::SyncOnceCell<T>::get_or_init_pin::{{closure}} (std/src/lazy.rs:375:25)
//!     #11: 0x10ae8441b: std::sync::once::Once::call_once_force::{{closure}} (src/sync/once.rs:320:40)
//!     #12: 0x10aea564c: std::sync::once::Once::call_inner (src/sync/once.rs:419:21)
//!     #13: 0x10ae81b1b: std::sync::once::Once::call_once_force (src/sync/once.rs:320:9)
//!     #14: 0x10ae81b1b: std::lazy::SyncOnceCell<T>::get_or_init_pin (std/src/lazy.rs:374:9)
//!     #15: 0x10ae81b1b: std::io::stdio::stdout (src/io/stdio.rs:679:16)
//!     #16: 0x10ae81b1b: std::io::stdio::print_to (src/io/stdio.rs:1196:21)
//!     #17: 0x10ae81b1b: std::io::stdio::_print (src/io/stdio.rs:1209:5)
//!     #18: 0x10ae2fe20: dhatter::main (dhatter/src/main.rs:8:5)
//!   }
//! }
//! ```
//! Full details about the output are in the [DHAT documentation]. Note that
//! DHAT uses the word "block" as a synonym for "allocation".
//!
//! [DHAT documentation]: https://valgrind.org/docs/manual/dh-manual.html
//!
//! When heap profiling, this crate doesn't track memory accesses (unlike DHAT)
//! and so the "reads" and "writes" measurements are not shown within DHAT's
//! viewer, and "sort metric" views involving reads, writes, or accesses are
//! not available.
//!
//! The backtraces produced by this crate are trimmed to reduce output file
//! sizes and improve readability in DHAT's viewer, in the following ways.
//! - Only one allocation-related frame will be shown at the top of the
//!   backtrace. That frame may be a function within `alloc::alloc`, a function
//!   within this crate, or a global allocation function like `__rg_alloc`.
//! - Common frames at the bottom of all backtraces, below `main`, are omitted.
//!
//! Backtrace trimming is inexact and if the above heuristics fail more frames
//! will be shown. [`ProfilerBuilder::trim_backtraces`] allows (approximate)
//! control of how deep backtraces will be.
//!
//! # Heap usage testing
//!
//! `dhat` lets you write tests that check that a certain piece of code does a
//! certain amount of heap allocation when it runs. This is sometimes called
//! "high water mark" testing. Sometimes it is precise (e.g. "this code should
//! do exactly 96 allocations" or "this code should free all allocations before
//! finishing") and sometimes it is less precise (e.g. "the peak heap usage of
//! this code should be less than 10 MiB").
//!
//! These tests are somewhat fragile, because heap profiling involves global
//! state (allocation stats), which introduces complications.
//! - `dhat` will panic if more than one `Profiler` is running at a time, but
//!   Rust tests run in parallel by default. So parallel running of heap usage
//!   tests must be prevented.
//! - If you use something like the
//!   [`serial_test`](https://docs.rs/serial_test/) crate to run heap usage
//!   tests in serial, Rust's test runner code by default still runs in
//!   parallel with those tests, and it allocates memory. These allocations
//!   will be counted by the `Profiler` as if they are part of the test, which
//!   will likely cause test failures.
//!
//! Therefore, the best approach is to put each heap usage test in its own
//! integration test file. Each integration test runs in its own process, and
//! so cannot interfere with any other test. Also, if there is only one test in
//! an integration test file, Rust's test runner code does not use any
//! parallelism, and so will not interfere with the test. If you do this, a
//! simple `cargo test` will work as expected.
//!
//! Alternatively, if you really want multiple heap usage tests in a single
//! integration test file you can write your own [custom test harness], which
//! is simpler than it sounds.
//!
//! [custom test harness]: https://www.infinyon.com/blog/2021/04/rust-custom-test-harness/
//!
//! But integration tests have some limits. For example, they only be used to
//! test items from libraries, not binaries. One way to get around this is to
//! restructure things so that most of the functionality is in a library, and
//! the binary is a thin wrapper around the library.
//!
//! Failing that, a blunt fallback is to run `cargo tests -- --test-threads=1`.
//! This disables all parallelism in tests, avoiding all the problems. This
//! allows the use of unit tests and multiples tests per integration test file,
//! at the cost of a non-standard invocation and slower test execution.
//!
//! With all that in mind, configuration of `Cargo.toml` is much the same as
//! for the profiling use case.
//!
//! Here is an example showing what is possible. This code would go in an
//! integration test within a crate's `tests/` directory:
//! ```
//! #[global_allocator]
//! static ALLOC: dhat::Alloc = dhat::Alloc;
//!
//! # // Tricky: comment out the `#[test]` because it's needed in an actual
//! # // test but messes up things here.
//! # /*
//! #[test]
//! # */
//! fn test() {
//!     let _profiler = dhat::Profiler::builder().testing().build();
//!
//!     let _v1 = vec![1, 2, 3, 4];
//!     let v2 = vec![5, 6, 7, 8];
//!     drop(v2);
//!     let v3 = vec![9, 10, 11, 12];
//!     drop(v3);
//!
//!     let stats = dhat::HeapStats::get();
//!
//!     // Three allocations were done in total.
//!     dhat::assert_eq!(stats.total_blocks, 3);
//!     dhat::assert_eq!(stats.total_bytes, 48);
//!
//!     // At the point of peak heap size, two allocations totalling 32 bytes existed.
//!     dhat::assert_eq!(stats.max_blocks, 2);
//!     dhat::assert_eq!(stats.max_bytes, 32);
//!
//!     // Now a single allocation remains alive.
//!     dhat::assert_eq!(stats.curr_blocks, 1);
//!     dhat::assert_eq!(stats.curr_bytes, 16);
//! }
//! # test()
//! ```
//! The [`testing`](ProfilerBuilder::testing) call puts the profiler into
//! testing mode, which allows the stats provided by [`HeapStats::get`] to be
//! checked with [`dhat::assert!`](assert) and similar assertions. These
//! assertions work much the same as normal assertions, except that if any of
//! them fail a heap profile will be saved.
//!
//! When viewing the heap profile after a test failure, the best choice of sort
//! metric in the viewer will depend on which stat was involved in the
//! assertion failure.
//! - `total_blocks`: "Total (blocks)"
//! - `total_bytes`: "Total (bytes)"
//! - `max_blocks` or `max_bytes`: "At t-gmax (bytes)"
//! - `curr_blocks` or `curr_bytes`: "At t-end (bytes)"
//!
//! This should give you a good understanding of why the assertion failed.
//!
//! Note: if you try this example test it may work in a debug build but fail in
//! a release build. This is because the compiler may optimize away some of the
//! allocations that are unused. This is a common problem for contrived
//! examples but less common for real tests. The unstable
//! [`std::hint::black_box`](std::hint::black_box) function may also be helpful
//! in this situation.
//!
//! # Ad hoc usage testing
//!
//! Ad hoc usage testing is also possible. It can be used to ensure certain
//! code points in your program are hit a particular number of times during
//! execution. It works in much the same way as heap usage testing, but
//! [`ProfilerBuilder::ad_hoc`] must be specified, [`AdHocStats::get`] is
//! used instead of [`HeapStats::get`], and there is no possibility of Rust's
//! test runner code interfering with the tests.

use backtrace::SymbolName;
use lazy_static::lazy_static;
use std::sync::Mutex;
use rustc_hash::FxHashMap;
use serde::Serialize;
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::ops::AddAssign;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use thousands::Separable;

lazy_static! {
    static ref TRI_GLOBALS: Mutex<Phase<Globals>> = Mutex::new(Phase::Ready);
}

// State transition diagram:
//
// +---------------> Ready
// |                   |
// | Profiler::        | ProfilerBuilder::
// | drop_inner()      | build()
// |                   v
// +---------------- Running
// |                   |
// |                   | check_assert_condition()
// |                   | [if the check fails]
// |                   v
// +---------------- PostAssert
//
// Note: the use of `std::process::exit` or `std::mem::forget` (on the
// `Profiler`) can result in termination while the profiler is still running,
// i.e. it won't produce output.
#[derive(PartialEq)]
enum Phase<T> {
    // We are ready to start running a `Profiler`.
    Ready,

    // A `Profiler` is running.
    Running(T),

    // The current `Profiler` has stopped due to as assertion failure, but
    // hasn't been dropped yet.
    PostAssert,
}

// Type used in frame trimming.
#[derive(PartialEq)]
enum TB {
    Top,
    Bottom,
}

// Global state that can be accessed from any thread and is therefore protected
// by a `Mutex`.
struct Globals {
    // The file name for the saved data.
    file_name: PathBuf,

    // Are we in testing mode?
    testing: bool,

    // How should we trim backtraces?
    trim_backtraces: Option<usize>,

    // Print the JSON to stderr when saving it?
    eprint_json: bool,

    // The backtrace at startup. Used for backtrace trimmming.
    start_bt: Backtrace,

    // Frames to trim at the top and bottom of backtraces. Computed once the
    // first backtrace is obtained during profiling; that backtrace is then
    // compared to `start_bt`.
    //
    // Each element is the address of a frame, and thus actually a `*mut
    // c_void`, but we store it as a `usize` because (a) we never dereference
    // it, and (b) using `*mut c_void` leads to compile errors because raw
    // pointers don't implement `Send`.
    frames_to_trim: Option<FxHashMap<usize, TB>>,

    // When `Globals` is created, which is when the `Profiler` is created.
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
    total_blocks: u64, // For ad hoc profiling it's actually `total_events`.
    total_bytes: u64,  // For ad hoc profiling it's actually `total_units`.

    // Extra things kept when heap profiling.
    heap: Option<HeapGlobals>,
}

struct HeapGlobals {
    // Each live block is associated with a `PpInfo`. An element is deleted
    // when the corresponding allocation is freed.
    //
    // Each key is the address of a live block, and thus actually a `*mut u8`,
    // but we store it as a `usize` because (a) we never dereference it, and
    // (b) using `*mut u8` leads to compile errors because raw pointers don't
    // implement `Send`.
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
    fn new(
        testing: bool,
        file_name: PathBuf,
        trim_backtraces: Option<usize>,
        eprint_json: bool,
        heap: Option<HeapGlobals>,
    ) -> Self {
        Self {
            testing,
            file_name,
            trim_backtraces,
            eprint_json,
            // `None` here because we don't want any frame trimming for this
            // backtrace.
            start_bt: new_backtrace_inner(None, &FxHashMap::default()),
            frames_to_trim: None,
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
        std::assert!(matches!(old, None));
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

    fn update_counts_for_ad_hoc_event(&mut self, pp_info_idx: usize, weight: usize) {
        std::assert!(self.heap.is_none());
        self.total_blocks += 1;
        self.total_bytes += weight as u64;

        self.pp_infos[pp_info_idx].update_counts_for_ad_hoc_event(weight);
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

    fn get_heap_stats(&self) -> HeapStats {
        match &self.heap {
            Some(heap) => HeapStats {
                total_blocks: self.total_blocks,
                total_bytes: self.total_bytes,
                curr_blocks: heap.curr_blocks,
                curr_bytes: heap.curr_bytes,
                max_blocks: heap.max_blocks,
                max_bytes: heap.max_bytes,
            },
            None => panic!("dhat: getting heap stats while doing ad hoc profiling"),
        }
    }

    fn get_ad_hoc_stats(&self) -> AdHocStats {
        match self.heap {
            None => AdHocStats {
                total_events: self.total_blocks,
                total_units: self.total_bytes,
            },
            Some(_) => panic!("dhat: getting ad hoc stats while doing heap profiling"),
        }
    }

    // Finish tracking allocations and deallocations, print a summary message
    // to `stderr` and save the profile to file/memory if requested.
    fn finish(mut self, memory_output: Option<&mut String>) {
        let now = Instant::now();

        if self.heap.is_some() {
            // Total bytes is at a possible peak.
            self.check_for_global_peak();

            let h = self.heap.as_ref().unwrap();

            // Account for the lifetimes of all remaining live blocks.
            for &LiveBlock {
                pp_info_idx,
                allocation_instant,
            } in h.live_blocks.values()
            {
                self.pp_infos[pp_info_idx]
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

        // Because `self` is being consumed, we can consume `self.backtraces`
        // and replace it with an empty `FxHashMap`. (This is necessary because
        // we modify the *keys* here with `resolve`, which isn't allowed with a
        // non-consuming iterator.)
        let pps: Vec<_> = std::mem::take(&mut self.backtraces)
            .into_iter()
            .map(|(mut bt, pp_info_idx)| {
                // Do the potentially expensive debug info lookups to get
                // symbol names, line numbers, etc.
                bt.0.resolve();

                // Trim boring frames at the top and bottom of the backtrace.
                let first_symbol_to_show = if self.trim_backtraces.is_some() {
                    if self.heap.is_some() {
                        bt.first_heap_symbol_to_show()
                    } else {
                        bt.first_ad_hoc_symbol_to_show()
                    }
                } else {
                    0
                };

                // Determine the frame indices for this backtrace. This
                // involves getting the string for each frame and adding a
                // new entry to `ftbl_indices` if it hasn't been seen
                // before.
                let mut fs = vec![];
                let mut i = 0;
                for frame in bt.0.frames().iter() {
                    for symbol in frame.symbols().iter() {
                        i += 1;
                        if (i - 1) < first_symbol_to_show {
                            continue;
                        }
                        let s = Backtrace::frame_to_string(frame, symbol);
                        let &mut ftbl_idx = ftbl_indices.entry(s).or_insert_with(|| {
                            next_ftbl_idx += 1;
                            next_ftbl_idx - 1
                        });
                        fs.push(ftbl_idx);
                    }
                }

                PpInfoJson::new(&self.pp_infos[pp_info_idx], fs)
            })
            .collect();

        // We pre-allocate `ftbl` with empty strings, and then fill it in.
        let mut ftbl = vec![String::new(); ftbl_indices.len()];
        for (frame, ftbl_idx) in ftbl_indices.into_iter() {
            ftbl[ftbl_idx] = frame;
        }

        let h = self.heap.as_ref();
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
                    .saturating_duration_since(self.start_instant)
                    .as_micros()
            }),
            te: now.duration_since(self.start_instant).as_micros(),
            pps,
            ftbl,
        };

        eprintln!(
            "dhat: Total:     {} {} in {} {}",
            self.total_bytes.separate_with_commas(),
            json.bsu.unwrap_or("bytes"),
            self.total_blocks.separate_with_commas(),
            json.bksu.unwrap_or("blocks"),
        );
        if let Some(h) = &self.heap {
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

        if let Some(memory_output) = memory_output {
            // Default pretty printing is fine here, it's only used for small
            // tests.
            *memory_output = serde_json::to_string_pretty(&json).unwrap();
            eprintln!("dhat: The data has been saved to the memory buffer");
        } else {
            let write = || -> std::io::Result<()> {
                let file = File::create(&self.file_name)?;
                // `to_writer` produces JSON that is compact.
                // `to_writer_pretty` produces JSON that is readable. This code
                // gives us JSON that is fairly compact and fairly readable.
                // Ideally it would be more like what DHAT produces, e.g. one
                // space indents, no spaces after `:` and `,`, and `fs` arrays
                // on a single line, but this is as good as we can easily
                // achieve.
                let formatter = serde_json::ser::PrettyFormatter::with_indent(b"");
                let mut ser = serde_json::Serializer::with_formatter(&file, formatter);
                json.serialize(&mut ser)?;
                Ok(())
            };
            match write() {
                Ok(()) => eprintln!(
                    "dhat: The data has been saved to {}, and is viewable with dhat/dh_view.html",
                    self.file_name.to_string_lossy()
                ),
                Err(e) => eprintln!(
                    "dhat: error: Writing to {} failed: {}",
                    self.file_name.to_string_lossy(),
                    e
                ),
            }
        }
        if self.eprint_json {
            eprintln!(
                "dhat: json = `{}`",
                serde_json::to_string_pretty(&json).unwrap()
            );
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
        std::assert!(self.heap.is_none());
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
// With this type we can run one code path if we are already ignoring
// allocations. Otherwise, we can a second code path while ignoring
// allocations. In practice, the first code path is unreachable except within
// the `GlobalAlloc` methods.
//
// WARNING: This type must be used for any code within this crate that can
// trigger allocations.
struct IgnoreAllocs {
    was_already_ignoring_allocs: bool,
}

thread_local!(static IGNORE_ALLOCS: Cell<bool> = Cell::new(false));

impl IgnoreAllocs {
    fn new() -> Self {
        Self {
            was_already_ignoring_allocs: IGNORE_ALLOCS.with(|b| b.replace(true)),
        }
    }
}

/// If code panics while `IgnoreAllocs` is live, this will still reset
/// `IGNORE_ALLOCS` so that it can be used again.
impl Drop for IgnoreAllocs {
    fn drop(&mut self) {
        if !self.was_already_ignoring_allocs {
            IGNORE_ALLOCS.with(|b| b.set(false));
        }
    }
}

/// A type whose lifetime dictates the start and end of profiling.
///
/// Profiling starts when the first value of this type is created. Profiling
/// stops when (a) this value is dropped or (b) a `dhat` assertion fails,
/// whichever comes first. When that happens, profiling data may be written to
/// file, depending on how the `Profiler` has been configured. Only one
/// `Profiler` can be running at any point in time.
//
// The actual profiler state is stored in `Globals`, so it can be accessed from
// places like `Alloc::alloc` and `ad_hoc_event()` when the `Profiler`
// instance isn't within reach.
#[derive(Debug)]
pub struct Profiler;

impl Profiler {
    /// Initiates allocation profiling.
    ///
    /// Typically the first thing in `main`. Its result should be assigned to a
    /// variable whose lifetime ends at the end of `main`.
    ///
    /// # Panics
    ///
    /// Panics if another `Profiler` is running.
    ///
    /// # Examples
    /// ```
    /// let _profiler = dhat::Profiler::new_heap();
    /// ```
    pub fn new_heap() -> Self {
        Self::builder().build()
    }

    /// Initiates ad hoc profiling.
    ///
    /// Typically the first thing in `main`. Its result should be assigned to a
    /// variable whose lifetime ends at the end of `main`.
    ///
    /// # Panics
    ///
    /// Panics if another `Profiler` is running.
    ///
    /// # Examples
    /// ```
    /// let _profiler = dhat::Profiler::new_ad_hoc();
    /// ```
    pub fn new_ad_hoc() -> Self {
        Self::builder().ad_hoc().build()
    }

    /// Creates a new [`ProfilerBuilder`], which defaults to heap profiling.
    pub fn builder() -> ProfilerBuilder {
        ProfilerBuilder {
            ad_hoc: false,
            testing: false,
            file_name: None,
            trim_backtraces: Some(10),
            eprint_json: false,
        }
    }
}

/// A builder for [`Profiler`], for cases beyond the basic ones provided by
/// [`Profiler`].
///
/// Created with [`Profiler::builder`].
#[derive(Debug)]
pub struct ProfilerBuilder {
    ad_hoc: bool,
    testing: bool,
    file_name: Option<PathBuf>,
    trim_backtraces: Option<usize>,
    eprint_json: bool,
}

impl ProfilerBuilder {
    /// Requests ad hoc profiling.
    ///
    /// # Examples
    /// ```
    /// let _profiler = dhat::Profiler::builder().ad_hoc().build();
    /// ```
    pub fn ad_hoc(mut self) -> Self {
        self.ad_hoc = true;
        self
    }

    /// Requests testing mode, which allows the use of
    /// [`dhat::assert!`](assert) and related macros, and disables saving of
    /// profile data on [`Profiler`] drop.
    ///
    /// # Examples
    /// ```
    /// let _profiler = dhat::Profiler::builder().testing().build();
    /// ```
    pub fn testing(mut self) -> Self {
        self.testing = true;
        self
    }

    /// Sets the name of the file in which profiling data will be saved.
    ///
    /// # Examples
    /// ```
    /// let file_name = format!("heap-{}.json", std::process::id());
    /// let _profiler = dhat::Profiler::builder().file_name(file_name).build();
    /// # std::mem::forget(_profiler); // Don't write the file in `cargo tests`
    /// ```
    pub fn file_name<P: AsRef<Path>>(mut self, file_name: P) -> Self {
        self.file_name = Some(file_name.as_ref().to_path_buf());
        self
    }

    /// Sets how backtrace trimming is performed.
    ///
    /// `dhat` can use heuristics to trim uninteresting frames from the top and
    /// bottom of backtraces, which makes the output easier to read. It can
    /// also limit the number of frames, which improves performance.
    ///
    /// The argument can be specified in several ways.
    /// - `None`: no backtrace trimming will be performed, and there is no
    ///   frame count limit. This makes profiling much slower and increases the
    ///   size of saved data files.
    /// - `Some(n)`: top and bottom trimming will be performed, and the number
    ///   of frames will be limited by `n`. Values of `n` less than 4 will be
    ///   clamped to 4.
    /// - `Some(usize::MAX)`: top and bottom trimming with be performed, but
    ///   there is no frame count limit. This makes profiling much slower and
    ///   increases the size of saved data files.
    ///
    /// The default value (used if this function is not called) is `Some(10)`.
    ///
    /// The number of frames shown in viewed profiles may differ from the
    /// number requested here, for two reasons.
    /// - Inline frames do not count towards this length. In release builds it
    ///   is common for the number of inline frames to equal or even exceed the
    ///   number of "real" frames.
    /// - Backtrace trimming will remove a small number of frames from heap
    ///   profile backtraces. The number removed will likely be more in a debug
    ///   build than in a release build.
    ///
    /// # Examples
    /// ```
    /// let _profiler = dhat::Profiler::builder().trim_backtraces(None).build();
    /// ```
    pub fn trim_backtraces(mut self, max_frames: Option<usize>) -> Self {
        self.trim_backtraces = max_frames.map(|m| std::cmp::max(m, 4));
        self
    }

    // For testing purposes only. Useful for seeing what went wrong if a test
    // fails on CI.
    #[doc(hidden)]
    pub fn eprint_json(mut self) -> Self {
        self.eprint_json = true;
        self
    }

    /// Creates a [`Profiler`] from the builder and initiates profiling.
    ///
    /// # Panics
    ///
    /// Panics if another [`Profiler`] is running.
    pub fn build(self) -> Profiler {
        let ignore_allocs = IgnoreAllocs::new();
        std::assert!(!ignore_allocs.was_already_ignoring_allocs);

        let phase: &mut Phase<Globals> = &mut TRI_GLOBALS.lock().unwrap();
        match phase {
            Phase::Ready => {
                let file_name = if let Some(file_name) = self.file_name {
                    file_name
                } else if !self.ad_hoc {
                    PathBuf::from("dhat-heap.json")
                } else {
                    PathBuf::from("dhat-ad-hoc.json")
                };
                let h = if !self.ad_hoc {
                    Some(HeapGlobals::new())
                } else {
                    None
                };
                *phase = Phase::Running(Globals::new(
                    self.testing,
                    file_name,
                    self.trim_backtraces,
                    self.eprint_json,
                    h,
                ));
            }
            Phase::Running(_) | Phase::PostAssert => {
                panic!("dhat: creating a profiler while a profiler is already running")
            }
        }
        Profiler
    }
}

// Get a backtrace according to `$g`'s settings. A macro rather than a `Global`
// method to avoid putting an extra frame into backtraces.
macro_rules! new_backtrace {
    ($g:expr) => {{
        if $g.frames_to_trim.is_none() {
            // This is the first backtrace from profiling. Work out what we
            // will be trimming from the top and bottom of all backtraces.
            // `None` here because we don't want any frame trimming for this
            // backtrace.
            let bt = new_backtrace_inner(None, &FxHashMap::default());
            $g.frames_to_trim = Some(bt.get_frames_to_trim(&$g.start_bt));
        }

        // Get the backtrace.
        new_backtrace_inner($g.trim_backtraces, $g.frames_to_trim.as_ref().unwrap())
    }};
}

// Get a backtrace, possibly trimmed.
//
// Note: it's crucial that there only be a single call to `backtrace::trace()`
// that is used everywhere, so that all traces will have the same backtrace
// function IPs in their top frames. (With multiple call sites we would have
// multiple closures, giving multiple instances of `backtrace::trace<F>`, and
// monomorphisation would put them into different functions in the binary.)
// Without this, top frame trimming wouldn't work. That's why this is a
// function (with `inline(never)` just to be safe) rather than a macro like
// `new_backtrace`. The frame for this function will be removed by top frame
// trimming.
#[inline(never)]
fn new_backtrace_inner(
    trim_backtraces: Option<usize>,
    frames_to_trim: &FxHashMap<usize, TB>,
) -> Backtrace {
    // Get the backtrace, trimming if necessary at the top and bottom and for
    // length.
    let mut frames = Vec::new();
    backtrace::trace(|frame| {
        let ip = frame.ip() as usize;
        if trim_backtraces.is_some() {
            match frames_to_trim.get(&ip) {
                Some(TB::Top) => return true,     // ignore frame and continue
                Some(TB::Bottom) => return false, // ignore frame and stop
                _ => {}                           // use this frame
            }
        }

        frames.push(frame.clone().into());

        if let Some(max_frames) = trim_backtraces {
            frames.len() < max_frames // stop if we have enough frames
        } else {
            true // continue
        }
    });
    Backtrace(frames.into())
}

/// A global allocator that tracks allocations and deallocations on behalf of
/// the [`Profiler`] type.
///
/// It must be set as the global allocator (via `#[global_allocator]`) when
/// doing heap profiling.
#[derive(Debug)]
pub struct Alloc;

unsafe impl GlobalAlloc for Alloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ignore_allocs = IgnoreAllocs::new();
        if ignore_allocs.was_already_ignoring_allocs {
            System.alloc(layout)
        } else {
            let phase: &mut Phase<Globals> = &mut TRI_GLOBALS.lock().unwrap();
            let ptr = System.alloc(layout);
            if ptr.is_null() {
                return ptr;
            }

            if let Phase::Running(g @ Globals { heap: Some(_), .. }) = phase {
                let size = layout.size();
                let bt = new_backtrace!(g);
                let pp_info_idx = g.get_pp_info(bt, PpInfo::new_heap);

                let now = Instant::now();
                g.record_block(ptr, pp_info_idx, now);
                g.update_counts_for_alloc(pp_info_idx, size, None, now);
            }
            ptr
        }
    }

    unsafe fn realloc(&self, old_ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let ignore_allocs = IgnoreAllocs::new();
        if ignore_allocs.was_already_ignoring_allocs {
            System.realloc(old_ptr, layout, new_size)
        } else {
            let phase: &mut Phase<Globals> = &mut TRI_GLOBALS.lock().unwrap();
            let new_ptr = System.realloc(old_ptr, layout, new_size);
            if new_ptr.is_null() {
                return new_ptr;
            }

            if let Phase::Running(g @ Globals { heap: Some(_), .. }) = phase {
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
                    let bt = new_backtrace!(g);
                    let pp_info_idx = g.get_pp_info(bt, PpInfo::new_heap);
                    (pp_info_idx, None)
                };

                let now = Instant::now();
                g.record_block(new_ptr, pp_info_idx, now);
                g.update_counts_for_alloc(pp_info_idx, new_size, delta, now);
            }
            new_ptr
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let ignore_allocs = IgnoreAllocs::new();
        if ignore_allocs.was_already_ignoring_allocs {
            System.dealloc(ptr, layout)
        } else {
            let phase: &mut Phase<Globals> = &mut TRI_GLOBALS.lock().unwrap();
            System.dealloc(ptr, layout);

            if let Phase::Running(g @ Globals { heap: Some(_), .. }) = phase {
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
        }
    }
}

/// Registers an event during ad hoc profiling.
///
/// The meaning of the weight argument is determined by the user. A call to
/// this function has no effect if a [`Profiler`] is not running or not doing ad
/// hoc profiling.
pub fn ad_hoc_event(weight: usize) {
    let ignore_allocs = IgnoreAllocs::new();
    std::assert!(!ignore_allocs.was_already_ignoring_allocs);

    let phase: &mut Phase<Globals> = &mut TRI_GLOBALS.lock().unwrap();
    if let Phase::Running(g @ Globals { heap: None, .. }) = phase {
        let bt = new_backtrace!(g);
        let pp_info_idx = g.get_pp_info(bt, PpInfo::new_ad_hoc);

        // Update counts.
        g.update_counts_for_ad_hoc_event(pp_info_idx, weight);
    }
}

impl Profiler {
    fn drop_inner(&mut self, memory_output: Option<&mut String>) {
        let ignore_allocs = IgnoreAllocs::new();
        std::assert!(!ignore_allocs.was_already_ignoring_allocs);

        let phase: &mut Phase<Globals> = &mut TRI_GLOBALS.lock().unwrap();
        match std::mem::replace(phase, Phase::Ready) {
            Phase::Ready => unreachable!(),
            Phase::Running(g) => {
                if !g.testing {
                    g.finish(memory_output)
                }
            }
            Phase::PostAssert => {}
        }
    }

    // For testing purposes only.
    #[doc(hidden)]
    pub fn drop_and_get_memory_output(&mut self) -> String {
        let mut memory_output = String::new();
        self.drop_inner(Some(&mut memory_output));
        memory_output
    }
}

impl Drop for Profiler {
    fn drop(&mut self) {
        self.drop_inner(None);
    }
}

// A wrapper for `backtrace::Backtrace` that implements `Eq` and `Hash`, which
// only look at the frame IPs. This assumes that any two
// `backtrace::Backtrace`s with the same frame IPs are equivalent.
#[derive(Debug)]
struct Backtrace(backtrace::Backtrace);

impl Backtrace {
    // The top frame symbols in a backtrace (those relating to backtracing
    // itself) are typically the same, and look something like this (Mac or
    // Linux release build, Dec 2021):
    // - 0x10fca200a: backtrace::backtrace::libunwind::trace
    // - 0x10fca200a: backtrace::backtrace::trace_unsynchronized
    // - 0x10fca200a: backtrace::backtrace::trace
    // - 0x10fc97350: dhat::new_backtrace_inner
    // - 0x10fc97984: [interesting function]
    //
    // We compare the top frames of a stack obtained while profiling with those
    // in `start_bt`. Those that overlap are the frames relating to backtracing
    // that can be discarded.
    //
    // The bottom frame symbols in a backtrace (those below `main`) are
    // typically the same, and look something like this (Mac or Linux release
    // build, Dec 2021):
    // - 0x1060f70e8: dhatter::main
    // - 0x1060f7026: core::ops::function::FnOnce::call_once
    // - 0x1060f7026: std::sys_common::backtrace::__rust_begin_short_backtrace
    // - 0x1060f703c: std::rt::lang_start::{{closure}}
    // - 0x10614b79a: core::ops::function::impls::<impl core::ops::function::FnOnce<A> for &F>::call_once
    // - 0x10614b79a: std::panicking::try::do_call
    // - 0x10614b79a: std::panicking::try
    // - 0x10614b79a: std::panic::catch_unwind
    // - 0x10614b79a: std::rt::lang_start_internal::{{closure}}
    // - 0x10614b79a: std::panicking::try::do_call
    // - 0x10614b79a: std::panicking::try
    // - 0x10614b79a: std::panic::catch_unwind
    // - 0x10614b79a: std::rt::lang_start_internal
    // - 0x1060f7259: ???
    //
    // We compare the bottom frames of a stack obtained while profiling with
    // those in `start_bt`. Those that overlap are the frames below main that
    // can be discarded.
    fn get_frames_to_trim(&self, start_bt: &Backtrace) -> FxHashMap<usize, TB> {
        let mut frames_to_trim = FxHashMap::default();
        let frames1 = self.0.frames();
        let frames2 = start_bt.0.frames();

        let (mut i1, mut i2) = (0, 0);
        loop {
            if i1 == frames1.len() - 1 || i2 == frames2.len() - 1 {
                // This should never happen in practice, it's too much
                // similarity between the backtraces. If it does happen,
                // abandon top trimming entirely.
                frames_to_trim.retain(|_, v| *v == TB::Bottom);
                break;
            }
            if frames1[i1].ip() != frames2[i2].ip() {
                break;
            }
            frames_to_trim.insert(frames1[i1].ip() as usize, TB::Top);
            i1 += 1;
            i2 += 1;
        }

        let (mut i1, mut i2) = (frames1.len() - 1, frames2.len() - 1);
        loop {
            if i1 == 0 || i2 == 0 {
                // This should never happen in practice, it's too much
                // similarity between the backtraces. If it does happen,
                // abandon bottom trimming entirely.
                frames_to_trim.retain(|_, v| *v == TB::Top);
                break;
            }
            if frames1[i1].ip() != frames2[i2].ip() {
                break;
            }
            frames_to_trim.insert(frames1[i1].ip() as usize, TB::Bottom);
            i1 -= 1;
            i2 -= 1;
        }

        frames_to_trim
    }

    // The top frame symbols in a trimmed heap profiling backtrace vary
    // significantly, depending on build configuration, platform, and program
    // point, and look something like this (Mac or Linux release build, Dec
    // 2021):
    // - 0x103ad464c: <dhat::Alloc as core::alloc::global::GlobalAlloc>::alloc
    // - 0x103acac99: __rg_alloc                    // sometimes missing
    // - 0x103acfe47: alloc::alloc::alloc           // sometimes missing
    // - 0x103acfe47: alloc::alloc::Global::alloc_impl
    // - 0x103acfe47: <alloc::alloc::Global as core::alloc::Allocator>::allocate
    // - 0x103acfe47: alloc::alloc::exchange_malloc // sometimes missing
    // - 0x103acfe47: [allocation point in program being profiled]
    //
    // We scan backwards for the first frame that looks like it comes from
    // allocator code, and all frames before it. If we don't find any such
    // frames, we show from frame 0, i.e. all frames.
    //
    // Note: this is a little dangerous. When deciding if a new backtrace has
    // been seen before, we consider all the IP addresses within it. And then
    // we trim some of those. It's possible that this will result in some
    // previously distinct traces becoming the same, which makes dh_view.html
    // abort. If that ever happens, look to see if something is going wrong
    // here.
    fn first_heap_symbol_to_show(&self) -> usize {
        // Examples of symbols that this search will match:
        // - alloc::alloc::{alloc,realloc,exchange_malloc}
        // - <alloc::alloc::Global as core::alloc::Allocator>::{allocate,grow}
        // - <dhat::Alloc as core::alloc::global::GlobalAlloc>::alloc
        // - __rg_{alloc,realloc}
        //
        // Be careful when changing this, because to do it properly requires
        // testing both debug and release builds on multiple platforms.
        self.first_symbol_to_show(|s| {
            s.starts_with("alloc::alloc::")
                || s.starts_with("<alloc::alloc::")
                || s.starts_with("<dhat::Alloc")
                || s.starts_with("__rg_")
        })
    }

    // The top frame symbols in a trimmed ad hoc profiling backtrace are always
    // the same, something like this (Mac or Linux release build, Dec 2021):
    // - 0x10cc1f504: dhat::ad_hoc_event
    // - 0x10cc1954d: [dhat::ad_hoc_event call site in program being profiled]
    //
    // So need not trim frames, and can show from frame 0 onward.
    fn first_ad_hoc_symbol_to_show(&self) -> usize {
        0
    }

    // Find the first symbol to show, based on the predicate `p`.
    fn first_symbol_to_show<P: Fn(&str) -> bool>(&self, p: P) -> usize {
        // Get the symbols into a vector so we can reverse iterate over them.
        let symbols: Vec<_> = self
            .0
            .frames()
            .iter()
            .map(|f| f.symbols().iter())
            .flatten()
            .collect();

        for (i, symbol) in symbols.iter().enumerate().rev() {
            // Use `{:#}` to print the "alternate" form of the symbol name,
            // which omits the trailing hash (e.g. `::ha68e4508a38cc95a`).
            if let Some(s) = symbol.name().map(|name| format!("{:#}", name)) {
                if p(&s) {
                    return i;
                }
            }
        }
        0
    }

    // Useful for debugging.
    #[allow(dead_code)]
    fn eprint(&self) {
        for frame in self.0.frames().iter() {
            for symbol in frame.symbols().iter() {
                eprintln!("{}", Backtrace::frame_to_string(frame, symbol));
            }
        }
    }

    fn frame_to_string(
        frame: &backtrace::BacktraceFrame,
        symbol: &backtrace::BacktraceSymbol,
    ) -> String {
        format!(
            // Use `{:#}` to print the "alternate" form of the symbol name,
            // which omits the trailing hash (e.g. `::ha68e4508a38cc95a`).
            "{:?}: {:#} ({:#}:{}:{})",
            frame.ip(),
            symbol.name().unwrap_or_else(|| SymbolName::new(b"???")),
            match symbol.filename() {
                Some(path) => trim_path(path),
                None => Path::new("???"),
            }
            .display(),
            symbol.lineno().unwrap_or(0),
            symbol.colno().unwrap_or(0),
        )
    }
}

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

// Trims a path with more than three components down to three (e.g.
// `/aa/bb/cc/dd.rs` becomes `bb/cc/dd.rs`), otherwise returns `path`
// unchanged.
fn trim_path(path: &Path) -> &Path {
    const N: usize = 3;
    let len = path.components().count();
    if len > N {
        let mut c = path.components();
        c.nth(len - (N + 1));
        c.as_path()
    } else {
        path
    }
}

/// Stats from heap profiling.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct HeapStats {
    /// Number of blocks (a.k.a. allocations) allocated over the entire run.
    pub total_blocks: u64,

    /// Number of bytes allocated over the entire run.
    pub total_bytes: u64,

    /// Number of blocks (a.k.a. allocations) currently allocated.
    pub curr_blocks: usize,

    /// Number of bytes currently allocated.
    pub curr_bytes: usize,

    /// Number of blocks (a.k.a. allocations) allocated at the global peak,
    /// i.e. when `curr_bytes` peaked.
    pub max_blocks: usize,

    /// Number of bytes allocated at the global peak, i.e. when `curr_bytes`
    /// peaked.
    pub max_bytes: usize,
}

/// Stats from ad hoc profiling.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct AdHocStats {
    /// Number of events recorded for the entire run.
    pub total_events: u64,

    /// Number of units recorded for the entire run.
    pub total_units: u64,
}

impl HeapStats {
    /// Gets the current heap stats.
    ///
    /// # Panics
    ///
    /// Panics if called when a [`Profiler`] is not running or not doing heap
    /// profiling.
    pub fn get() -> Self {
        let ignore_allocs = IgnoreAllocs::new();
        std::assert!(!ignore_allocs.was_already_ignoring_allocs);

        let phase: &mut Phase<Globals> = &mut TRI_GLOBALS.lock().unwrap();
        match phase {
            Phase::Ready => {
                panic!("dhat: getting heap stats when no profiler is running")
            }
            Phase::Running(g) => g.get_heap_stats(),
            Phase::PostAssert => {
                panic!("dhat: getting heap stats after the profiler has asserted")
            }
        }
    }
}

impl AdHocStats {
    /// Gets the current ad hoc stats.
    ///
    /// # Panics
    ///
    /// Panics if called when a [`Profiler`] is not running or not doing ad hoc
    /// profiling.
    pub fn get() -> Self {
        let ignore_allocs = IgnoreAllocs::new();
        std::assert!(!ignore_allocs.was_already_ignoring_allocs);

        let phase: &mut Phase<Globals> = &mut TRI_GLOBALS.lock().unwrap();
        match phase {
            Phase::Ready => {
                panic!("dhat: getting ad hoc stats when no profiler is running")
            }
            Phase::Running(g) => g.get_ad_hoc_stats(),
            Phase::PostAssert => {
                panic!("dhat: getting ad hoc stats after the profiler has asserted")
            }
        }
    }
}

// Just an implementation detail of the assert macros.
// njn: invert sense of the return value?
#[doc(hidden)]
pub fn check_assert_condition<F>(cond: F) -> bool
where
    F: FnOnce() -> bool,
{
    // We do the test within `check_assert_condition` (as opposed to within the
    // `assert*` macros) so that we'll always detect if the profiler isn't
    // running.
    let ignore_allocs = IgnoreAllocs::new();
    std::assert!(!ignore_allocs.was_already_ignoring_allocs);

    let phase: &mut Phase<Globals> = &mut TRI_GLOBALS.lock().unwrap();
    match phase {
        Phase::Ready => panic!("dhat: asserting when no profiler is running"),
        Phase::Running(g) => {
            if !g.testing {
                panic!("dhat: asserting while not in testing mode");
            }
            if cond() {
                return false;
            }
        }
        Phase::PostAssert => panic!("dhat: asserting after the profiler has asserted"),
    }

    // Failure.
    match std::mem::replace(phase, Phase::PostAssert) {
        Phase::Ready => unreachable!(),
        Phase::Running(g) => {
            g.finish(None);
            true
        }
        Phase::PostAssert => unreachable!(),
    }
}

/// Asserts that an expression is true.
///
/// Like [`std::assert!`], additional format arguments are supported. On
/// failure, this macro will save the profile data and panic.
///
/// # Panics
///
/// Panics immediately (without saving the profile data) in the following
/// circumstances.
/// - If called when a [`Profiler`] is not running or is not in testing mode.
/// - If called after a previous `dhat` assertion has failed with the current
///   [`Profiler`]. This is possible if [`std::panic::catch_unwind`] is used.
#[macro_export]
macro_rules! assert {
    ($cond:expr) => ({
        if dhat::check_assert_condition(|| $cond) {
            panic!("dhat: assertion failed: {}", stringify!($cond));
        }
    });
    ($cond:expr, $($arg:tt)+) => ({
        if dhat::check_assert_condition(|| $cond) {
            panic!("dhat: assertion failed: {}: {}", stringify!($cond), format_args!($($arg)+));
        }
    });
}

/// Asserts that two expressions are equal.
///
/// Like [`std::assert_eq!`], additional format arguments are supported. On
/// failure, this macro will save the profile data and panic.
///
/// # Panics
///
/// Panics immediately (without saving the profile data) in the following
/// circumstances.
/// - If called when a [`Profiler`] is not running or is not in testing mode.
/// - If called after a previous `dhat` assertion has failed with the current
///   [`Profiler`]. This is possible if [`std::panic::catch_unwind`] is used.
#[macro_export]
macro_rules! assert_eq {
    ($left:expr, $right:expr $(,)?) => ({
        if dhat::check_assert_condition( || $left == $right) {
            panic!(
                "dhat: assertion failed: `(left == right)`\n  left: `{:?}`,\n right: `{:?}`",
                $left, $right
            );
        }
    });
    ($left:expr, $right:expr, $($arg:tt)+) => ({
        if dhat::check_assert_condition(|| $left == $right) {
            panic!(
                "dhat: assertion failed: `(left == right)`\n  left: `{:?}`,\n right: `{:?}`: {}",
                $left, $right, format_args!($($arg)+)
            );
        }
    });
}

/// Asserts that two expressions are not equal.
///
/// Like [`std::assert_ne!`], additional format arguments are supported. On
/// failure, this macro will save the profile data and panic.
///
/// # Panics
///
/// Panics immediately (without saving the profile data) in the following
/// circumstances.
/// - If called when a [`Profiler`] is not running or is not in testing mode.
/// - If called after a previous `dhat` assertion has failed with the current
///   [`Profiler`]. This is possible if [`std::panic::catch_unwind`] is used.
#[macro_export]
macro_rules! assert_ne {
    ($left:expr, $right:expr) => ({
        if dhat::check_assert_condition(|| $left != $right) {
            panic!(
                "dhat: assertion failed: `(left != right)`\n  left: `{:?}`,\n right: `{:?}`",
                $left, $right
            );
        }
    });
    ($left:expr, $right:expr, $($arg:tt)+) => ({
        if dhat::check_assert_condition(|| $left != $right) {
            panic!(
                "dhat: assertion failed: `(left != right)`\n  left: `{:?}`,\n right: `{:?}`: {}",
                $left, $right, format_args!($($arg)+)
            );
        }
    });
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

// For testing purposes only.
#[doc(hidden)]
pub fn assert_is_panic<R, F: FnOnce() -> R + std::panic::UnwindSafe>(f: F, expected: &str) {
    let res = std::panic::catch_unwind(f);
    if let Err(err) = res {
        if let Some(actual) = err.downcast_ref::<&str>() {
            std::assert_eq!(expected, *actual);
        } else if let Some(actual) = err.downcast_ref::<String>() {
            std::assert_eq!(expected, actual);
        } else {
            panic!("match_panic: Not a string: {:?}", err);
        }
    } else {
        panic!("match_panic: Not an error");
    }
}

#[cfg(test)]
mod test {
    use super::trim_path;
    use std::path::Path;

    #[test]
    fn test_trim_path() {
        std::assert_eq!(trim_path(Path::new("")), Path::new(""));
        std::assert_eq!(trim_path(Path::new("/")), Path::new("/"));
        std::assert_eq!(trim_path(Path::new("aa.rs")), Path::new("aa.rs"));
        std::assert_eq!(trim_path(Path::new("/aa.rs")), Path::new("/aa.rs"));
        std::assert_eq!(trim_path(Path::new("bb/aa.rs")), Path::new("bb/aa.rs"));
        std::assert_eq!(trim_path(Path::new("/bb/aa.rs")), Path::new("/bb/aa.rs"));
        std::assert_eq!(
            trim_path(Path::new("cc/bb/aa.rs")),
            Path::new("cc/bb/aa.rs")
        );
        std::assert_eq!(
            trim_path(Path::new("/cc/bb/aa.rs")),
            Path::new("cc/bb/aa.rs")
        );
        std::assert_eq!(
            trim_path(Path::new("dd/cc/bb/aa.rs")),
            Path::new("cc/bb/aa.rs")
        );
        std::assert_eq!(
            trim_path(Path::new("/dd/cc/bb/aa.rs")),
            Path::new("cc/bb/aa.rs")
        );
    }
}

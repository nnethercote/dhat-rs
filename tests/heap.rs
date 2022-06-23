#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
fn main() {
    use serde_json::Value::{self, *};

    // Allocated before heap profiling starts.
    let v1 = vec![1u32, 2, 3, 4];
    let v2 = vec![1u32, 2, 3, 4];
    let mut v3 = vec![1u32, 2, 3, 4];
    let mut v4 = vec![1u32, 2, 3, 4];

    let mem = {
        let mut profiler = std::mem::ManuallyDrop::new(
            dhat::Profiler::builder()
                .trim_backtraces(Some(usize::MAX))
                .eprint_json()
                .build(),
        );

        // Things allocated beforehand aren't counted.
        let stats = dhat::HeapStats::get();
        assert_eq!(stats.total_blocks, 0);
        assert_eq!(stats.total_bytes, 0);
        assert_eq!(stats.curr_blocks, 0);
        assert_eq!(stats.curr_bytes, 0);
        assert_eq!(stats.max_blocks, 0);
        assert_eq!(stats.max_bytes, 0);

        // Allocated before, freed during.
        drop(v1);

        // Allocated before, reallocated during. Treated like a fresh alloc.
        v3.push(5);

        // Allocated during, reallocated/dropped during.
        let v5 = vec![0u8; 1000];
        let mut v6 = vec![0u8; 200];
        v6.reserve(200); // global peak
        drop(v5);

        // Need to play some games with variable addresses here to prevent
        // rustc from either (a) constant folding this loop into nothing, or
        // (b) unrolling it.
        let mut x = 0usize;
        let addr = &x as *const usize as usize; // always be bigger than 10
        for i in 0..std::cmp::min(10, addr) {
            let v7 = vec![i as u8];
            x += v7.as_ptr() as usize; // can't unroll this
        }
        assert_ne!(x, 0);

        let stats = dhat::HeapStats::get();
        assert_eq!(stats.total_blocks, 14);
        assert_eq!(stats.total_bytes, 1642);
        assert_eq!(stats.curr_blocks, 2);
        assert_eq!(stats.curr_bytes, 432);
        assert_eq!(stats.max_blocks, 3);
        assert_eq!(stats.max_bytes, 1432);

        drop(v6);
        profiler.drop_and_get_memory_output()
    };

    // Allocated before, freed after.
    drop(v2);

    // Allocated before, reallocated after.
    v4.push(5);

    // Check basics.
    let mut v: Value = serde_json::from_str(&mem).unwrap();
    assert_eq!(v["dhatFileVersion"].as_i64().unwrap(), 2);
    assert_eq!(v["mode"], "rust-heap");
    assert_eq!(v["verb"], "Allocated");
    assert_eq!(v["bklt"], Bool(true));
    assert_eq!(v["bkacc"], Bool(false));
    assert_eq!(v["bu"], Null);
    assert_eq!(v["bsu"], Null);
    assert_eq!(v["bksu"], Null);
    assert_eq!(v["tu"], "µs");
    assert_eq!(v["Mtu"], "s");
    assert_eq!(v["tuth"].as_i64().unwrap(), 10);
    assert!(matches!(&v["cmd"], String(s) if s.contains("heap")));
    assert!(matches!(v["pid"], Number(_)));
    assert!(v["te"].as_i64().unwrap() >= v["tg"].as_i64().unwrap());

    // Order PPs by "tb" field.
    let pps = v["pps"].as_array_mut().unwrap();
    pps.sort_unstable_by_key(|pp| pp["tb"].as_i64().unwrap());
    pps.reverse();

    // v5
    let pp0 = &pps[0];
    assert_eq!(pp0["tb"].as_i64().unwrap(), 1000);
    assert_eq!(pp0["tbk"].as_i64().unwrap(), 1);
    assert!(pp0["tl"].as_i64().unwrap() > 0);
    assert_eq!(pp0["mb"].as_i64().unwrap(), 1000);
    assert_eq!(pp0["mbk"].as_i64().unwrap(), 1);
    assert_eq!(pp0["gb"].as_i64().unwrap(), 1000);
    assert_eq!(pp0["gbk"].as_i64().unwrap(), 1);
    assert_eq!(pp0["eb"].as_i64().unwrap(), 0);
    assert_eq!(pp0["ebk"].as_i64().unwrap(), 0);
    assert!(matches!(pp0["fs"], Array(_)));

    // v6
    let pp1 = &pps[1];
    assert_eq!(pp1["tb"].as_i64().unwrap(), 600);
    assert_eq!(pp1["tbk"].as_i64().unwrap(), 2);
    assert!(pp1["tl"].as_i64().unwrap() > 0);
    assert_eq!(pp1["mb"].as_i64().unwrap(), 400);
    assert_eq!(pp1["mbk"].as_i64().unwrap(), 1);
    assert_eq!(pp1["gb"].as_i64().unwrap(), 400);
    assert_eq!(pp1["gbk"].as_i64().unwrap(), 1);
    assert_eq!(pp1["eb"].as_i64().unwrap(), 0);
    assert_eq!(pp1["ebk"].as_i64().unwrap(), 0);
    assert!(matches!(pp1["fs"], Array(_)));

    // v3
    let pp2 = &pps[2];
    assert_eq!(pp2["tb"].as_i64().unwrap(), 32);
    assert_eq!(pp2["tbk"].as_i64().unwrap(), 1);
    assert!(pp2["tl"].as_i64().unwrap() > 0);
    assert_eq!(pp2["mb"].as_i64().unwrap(), 32);
    assert_eq!(pp2["mbk"].as_i64().unwrap(), 1);
    assert_eq!(pp2["gb"].as_i64().unwrap(), 32);
    assert_eq!(pp2["gbk"].as_i64().unwrap(), 1);
    assert_eq!(pp2["eb"].as_i64().unwrap(), 32);
    assert_eq!(pp2["ebk"].as_i64().unwrap(), 1);
    assert!(matches!(pp2["fs"], Array(_)));

    // _v7
    let pp3 = &pps[3];
    assert_eq!(pp3["tb"].as_i64().unwrap(), 10);
    assert_eq!(pp3["tbk"].as_i64().unwrap(), 10);
    assert!(pp3["tl"].as_i64().unwrap() >= 0);
    assert_eq!(pp3["mb"].as_i64().unwrap(), 1);
    assert_eq!(pp3["mbk"].as_i64().unwrap(), 1);
    assert_eq!(pp3["gb"].as_i64().unwrap(), 0);
    assert_eq!(pp3["gbk"].as_i64().unwrap(), 0);
    assert_eq!(pp3["eb"].as_i64().unwrap(), 0);
    assert_eq!(pp3["ebk"].as_i64().unwrap(), 0);
    assert!(matches!(pp3["fs"], Array(_)));

    // Look for parts of some expected frames.
    let ftbl = &v["ftbl"].as_array().unwrap();
    let y = |s| {
        ftbl.iter()
            .find(|&f| f.as_str().unwrap().contains(s))
            .is_some()
    };
    let n = |s| {
        ftbl.iter()
            .find(|&f| f.as_str().unwrap().contains(s))
            .is_none()
    };
    assert!(y("[root]"));

    // These tests will fail if the repo directory isn't called `dhat-rs`.
    if cfg!(windows) {
        if cfg!(debug_assertions) {
            assert!(y(
                "alloc::vec::Vec<u32,alloc::alloc::Global>::push<u32,alloc::alloc::Global>"
            ));
            assert!(y("heap::main (dhat-rs\\tests\\heap.rs:35:0)")); // v3
            assert!(y("heap::main (dhat-rs\\tests\\heap.rs:38:0)")); // v5
            assert!(y("heap::main (dhat-rs\\tests\\heap.rs:39:0)")); // v6
            assert!(y("heap::main (dhat-rs\\tests\\heap.rs:49:0)")); // _v7
        } else {
            // Stack traces are terrible in Windows release builds.
            assert!(y("RawVec"));
        }
    } else {
        assert!(y("alloc::vec::Vec<T,A>::push"));
        assert!(y("heap::main (dhat-rs/tests/heap.rs:35:9)")); // v3
        assert!(y("heap::main (dhat-rs/tests/heap.rs:38:18)")); // v5
        assert!(y("heap::main (dhat-rs/tests/heap.rs:39:22)")); // v6
        assert!(y("heap::main (dhat-rs/tests/heap.rs:49:22)")); // _v7
    }

    // This stuff should be removed by backtrace trimming.
    assert!(n("backtrace::"));
    assert!(n("alloc::alloc::Global::alloc_impl"));
    assert!(n("<dhat::Alloc as core::alloc::global::GlobalAlloc>"));
    assert!(n("alloc::alloc::Global::alloc_impl"));
    assert!(n("__rg_"));
    assert!(n("lang_start::"));
    assert!(n("call_once::"));
    assert!(n("catch_unwind::"));
    assert!(n("panic"));

    // A trivial second profiler in the same run, albeit an ad hoc profiler.
    let _profiler = dhat::Profiler::new_ad_hoc();
}

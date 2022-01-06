#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn f2() {
    dhat::ad_hoc_event(1);
    dhat::ad_hoc_event(2);
}

fn f1() {
    f2();
    dhat::ad_hoc_event(3);
}

#[test]
fn main() {
    use serde_json::Value::{self, *};

    // Ignored because profiling hasn't started yet.
    dhat::ad_hoc_event(3);

    let mem = {
        let mut profiler = std::mem::ManuallyDrop::new(
            dhat::Profiler::builder()
                .ad_hoc()
                .trim_backtraces(Some(usize::MAX))
                .eprint_json()
                .build(),
        );

        let stats = dhat::AdHocStats::get();
        assert_eq!(stats.total_events, 0);
        assert_eq!(stats.total_units, 0);

        dhat::ad_hoc_event(100);
        f1();

        // This should have no effect, because we're ad hoc profiling.
        let _v: Vec<u8> = Vec::with_capacity(100);

        let stats = dhat::AdHocStats::get();
        assert_eq!(stats.total_events, 4);
        assert_eq!(stats.total_units, 106);

        profiler.drop_and_get_memory_output()
    };

    // Ignored because profiling has finished.
    dhat::ad_hoc_event(5);

    // Check basics.
    let mut v: Value = serde_json::from_str(&mem).unwrap();
    assert_eq!(v["dhatFileVersion"].as_i64().unwrap(), 2);
    assert_eq!(v["mode"], "rust-ad-hoc");
    assert_eq!(v["verb"], "Allocated");
    assert_eq!(v["bklt"], Bool(false));
    assert_eq!(v["bkacc"], Bool(false));
    assert!(matches!(&v["bu"], String(s) if s == "unit"));
    assert!(matches!(&v["bsu"], String(s) if s == "units"));
    assert!(matches!(&v["bksu"], String(s) if s == "events"));
    assert_eq!(v["tu"], "µs");
    assert_eq!(v["Mtu"], "s");
    assert!(matches!(&v["cmd"], String(s) if s.contains("ad_hoc")));
    assert!(matches!(v["pid"], Number(_)));
    assert_ne!(v["te"].as_i64().unwrap(), 0);
    assert_eq!(v["tg"], Null);

    // Order PPs by "tb" field.
    let pps = v["pps"].as_array_mut().unwrap();
    pps.sort_unstable_by_key(|pp| pp["tb"].as_i64().unwrap());
    pps.reverse();

    // main
    let pp0 = &pps[0];
    assert_eq!(pp0["tb"].as_i64().unwrap(), 100);
    assert_eq!(pp0["tbk"].as_i64().unwrap(), 1);
    assert_eq!(pp0["tl"], Null);
    assert_eq!(pp0["mb"], Null);
    assert_eq!(pp0["mbk"], Null);
    assert_eq!(pp0["gb"], Null);
    assert_eq!(pp0["gbk"], Null);
    assert_eq!(pp0["eb"], Null);
    assert_eq!(pp0["ebk"], Null);
    assert!(matches!(pp0["fs"], Array(_)));

    // f1 (second)
    let pp1 = &pps[1];
    assert_eq!(pp1["tb"].as_i64().unwrap(), 3);
    assert_eq!(pp1["tbk"].as_i64().unwrap(), 1);
    assert_eq!(pp1["tl"], Null);
    assert_eq!(pp1["mb"], Null);
    assert_eq!(pp1["mbk"], Null);
    assert_eq!(pp1["gb"], Null);
    assert_eq!(pp1["gbk"], Null);
    assert_eq!(pp1["eb"], Null);
    assert_eq!(pp1["ebk"], Null);
    assert!(matches!(pp1["fs"], Array(_)));

    // f2 (second)
    let pp2 = &pps[2];
    assert_eq!(pp2["tb"].as_i64().unwrap(), 2);
    assert_eq!(pp2["tbk"].as_i64().unwrap(), 1);
    assert_eq!(pp2["tl"], Null);
    assert_eq!(pp2["mb"], Null);
    assert_eq!(pp2["mbk"], Null);
    assert_eq!(pp2["gb"], Null);
    assert_eq!(pp2["gbk"], Null);
    assert_eq!(pp2["eb"], Null);
    assert_eq!(pp2["ebk"], Null);
    assert!(matches!(pp2["fs"], Array(_)));

    // f2 (first)
    let pp3 = &pps[3];
    assert_eq!(pp3["tb"].as_i64().unwrap(), 1);
    assert_eq!(pp3["tbk"].as_i64().unwrap(), 1);
    assert_eq!(pp3["tl"], Null);
    assert_eq!(pp3["mb"], Null);
    assert_eq!(pp3["mbk"], Null);
    assert_eq!(pp3["gb"], Null);
    assert_eq!(pp3["gbk"], Null);
    assert_eq!(pp3["eb"], Null);
    assert_eq!(pp3["ebk"], Null);
    assert!(matches!(pp3["fs"], Array(_)));

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
    assert!(y("dhat::ad_hoc_event"));

    // These tests will fail if the repo directory isn't called `dhat-rs`.
    if cfg!(windows) {
        // Some frames are missing on Windows, not sure why.
        assert!(y("ad_hoc::f2 (dhat-rs\\tests\\ad-hoc.rs:5:0)"));
        assert!(y("ad_hoc::f2 (dhat-rs\\tests\\ad-hoc.rs:6:0)"));
        //assert!(y("ad_hoc::f1 (dhat-rs\\tests\\ad-hoc.rs:10:0)"));
        assert!(y("ad_hoc::f1 (dhat-rs\\tests\\ad-hoc.rs:11:0)"));
        assert!(y("ad_hoc::main (dhat-rs\\tests\\ad-hoc.rs:34:0)"));
        //assert!(y("ad_hoc::main (dhat-rs\\tests\\ad-hoc.rs:35:0)"));
    } else {
        assert!(y("ad_hoc::f2 (dhat-rs/tests/ad-hoc.rs:5:5)"));
        assert!(y("ad_hoc::f2 (dhat-rs/tests/ad-hoc.rs:6:5)"));
        assert!(y("ad_hoc::f1 (dhat-rs/tests/ad-hoc.rs:10:5)"));
        assert!(y("ad_hoc::f1 (dhat-rs/tests/ad-hoc.rs:11:5)"));
        assert!(y("ad_hoc::main (dhat-rs/tests/ad-hoc.rs:34:9)"));
        assert!(y("ad_hoc::main (dhat-rs/tests/ad-hoc.rs:35:9)"));
    }

    // This stuff should be removed by backtrace trimming.
    assert!(n("backtrace::"));
    assert!(n("lang_start::"));
    assert!(n("call_once::"));
    assert!(n("catch_unwind::"));
    assert!(n("panic"));

    // A trivial second profiler in the same run, albeit a heap profiler.
    let _profiler = dhat::Profiler::new_heap();
}

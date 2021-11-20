#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn f3() {
    dhat::ad_hoc_event(1);
}

fn f2() {
    f3();
    dhat::ad_hoc_event(2);
    f3();
}

fn f1() {
    f2();
    dhat::ad_hoc_event(3);
    f2();
}

#[test]
fn main() {
    let _dhat = dhat::start_ad_hoc_profiling();

    let empty_stats = dhat::AdHocStats {
        total_events: 0,
        total_units: 0,
    };
    assert_eq!(dhat::get_ad_hoc_stats(), empty_stats);

    f1();
    dhat::ad_hoc_event(100);
    f1();

    let final_stats = dhat::AdHocStats {
        total_events: 15,
        total_units: 122,
    };
    assert_eq!(dhat::get_ad_hoc_stats(), final_stats);
}

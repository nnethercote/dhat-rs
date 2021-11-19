fn main() {
    let _dhat = dhat::start_ad_hoc_profiling();

    dhat::ad_hoc_event(100);
    println!("Hello, world!");
    dhat::ad_hoc_event(200);
}

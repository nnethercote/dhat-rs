fn main() {
    let _profiler = dhat::Profiler::new_ad_hoc();

    dhat::ad_hoc_event(100);
    println!("Hello, world!");
    dhat::ad_hoc_event(200);
}

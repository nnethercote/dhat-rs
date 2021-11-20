fn main() {
    let _profiler = dhat::Profiler::ad_hoc_start();

    dhat::ad_hoc_event(100);
    println!("Hello, world!");
    dhat::ad_hoc_event(200);
}

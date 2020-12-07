use dhat::Dhat;

fn main() {
    let _dhat = Dhat::start_ad_hoc_profiling();

    dhat::ad_hoc_event(100);
    println!("Hello, world!");
    dhat::ad_hoc_event(200);
}

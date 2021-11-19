#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
#[should_panic]
fn stats_panic_1() {
    let _stats = dhat::get_stats(); // panic
}

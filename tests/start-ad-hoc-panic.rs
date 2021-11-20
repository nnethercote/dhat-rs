#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[test]
#[should_panic(expected = "dhat: profiling started a second time")]
fn main() {
    let _dhat = dhat::start_ad_hoc_profiling();

    let _v = vec![1u32, 2, 3, 4];

    let _dhat = dhat::start_ad_hoc_profiling(); // panic
}

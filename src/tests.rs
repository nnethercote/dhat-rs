#[cfg(test)]
mod tests {
    use crate::*;
    use serial_test::serial;
    use std::alloc::GlobalAlloc;
    use std::alloc::Layout;

    // Use this at the start of each test.
    fn reset_tri_globals() {
        let tri: &mut Tri<Globals> = &mut TRI_GLOBALS.lock();
        *tri = Tri::Pre;
    }

    // We can't use a global allocator in a unit test, so this test does
    // explicit calls to a `dhat::Alloc` and checks that `TRI_GLOBALS` looks
    // like it should. This doesn't test the intercept blocking, but it's hard
    // to do better without having a separate test binary.
    #[test]
    #[serial] // because it involves global state
    fn heap() {
        reset_tri_globals();

        let alloc = Alloc;
        let mut dhat = start_heap_profiling();
        let layout256 = Layout::from_size_align(256, 8).unwrap();

        // PpInfo 0
        let ptr0 = unsafe { alloc.alloc(layout256) };

        let tgmax_instant = {
            let tri: &mut Tri<Globals> = &mut TRI_GLOBALS.lock();
            let g = tri.as_ref_unwrap();
            let h = g.heap.as_ref().unwrap();
            assert_eq!(g.pp_infos.len(), 1);
            assert_eq!(g.backtraces.len(), 1);
            assert_eq!(g.total_blocks, 1);
            assert_eq!(g.total_bytes, 256);
            assert_eq!(h.live_blocks.len(), 1);
            assert_eq!(h.curr_blocks, 1);
            assert_eq!(h.curr_bytes, 256);
            assert_eq!(h.max_blocks, 1);
            assert_eq!(h.max_bytes, 256);
            assert!(h.tgmax_instant > g.start_instant);

            let pp_info = &g.pp_infos[0];
            let a = pp_info.heap.as_ref().unwrap();
            assert_eq!(pp_info.total_blocks, 1);
            assert_eq!(pp_info.total_bytes, 256);
            assert_eq!(a.curr_blocks, 1);
            assert_eq!(a.curr_bytes, 256);
            assert_eq!(a.max_blocks, 1);
            assert_eq!(a.max_bytes, 256);
            // These don't get updated until we come down from a peak, or run
            // `Dhat::drop`.
            assert_eq!(a.at_tgmax_blocks, 0);
            assert_eq!(a.at_tgmax_bytes, 0);

            h.tgmax_instant
        };

        // PpInfo 0, again
        let _ptr0 = unsafe { alloc.realloc(ptr0, layout256, 512) };

        let tgmax_instant = {
            let tri: &mut Tri<Globals> = &mut TRI_GLOBALS.lock();
            let g = tri.as_ref_unwrap();
            let h = g.heap.as_ref().unwrap();
            assert_eq!(g.pp_infos.len(), 1);
            assert_eq!(g.backtraces.len(), 1);
            assert_eq!(g.total_blocks, 2);
            assert_eq!(g.total_bytes, 768);
            assert_eq!(h.live_blocks.len(), 1);
            assert_eq!(h.curr_blocks, 1);
            assert_eq!(h.curr_bytes, 512);
            assert_eq!(h.max_blocks, 1);
            assert_eq!(h.max_bytes, 512);
            assert!(h.tgmax_instant > tgmax_instant);

            let pp_info = &g.pp_infos[0];
            let a = pp_info.heap.as_ref().unwrap();
            assert_eq!(pp_info.total_blocks, 2);
            assert_eq!(pp_info.total_bytes, 768);
            assert_eq!(a.curr_blocks, 1);
            assert_eq!(a.curr_bytes, 512);
            assert_eq!(a.max_blocks, 1);
            assert_eq!(a.max_bytes, 512);
            // These don't get updated until we come down from a peak, or run
            // `Dhat::drop`.
            assert_eq!(a.at_tgmax_blocks, 0);
            assert_eq!(a.at_tgmax_bytes, 0);

            h.tgmax_instant
        };

        // PpInfo 1 and 2
        let _ptr1 = unsafe { alloc.alloc(layout256) };
        let ptr2 = unsafe { alloc.alloc(layout256) };

        let tgmax_instant = {
            let tri: &mut Tri<Globals> = &mut TRI_GLOBALS.lock();
            let g = tri.as_ref_unwrap();
            let h = g.heap.as_ref().unwrap();
            assert_eq!(g.pp_infos.len(), 3);
            assert_eq!(g.backtraces.len(), 3);
            assert_eq!(g.total_blocks, 4);
            assert_eq!(g.total_bytes, 1280);
            assert_eq!(h.live_blocks.len(), 3);
            assert_eq!(h.curr_blocks, 3);
            assert_eq!(h.curr_bytes, 1024);
            assert_eq!(h.max_blocks, 3);
            assert_eq!(h.max_bytes, 1024);
            assert!(h.tgmax_instant > tgmax_instant);

            let pp_info = &g.pp_infos[1];
            let a = pp_info.heap.as_ref().unwrap();
            assert_eq!(pp_info.total_blocks, 1);
            assert_eq!(pp_info.total_bytes, 256);
            assert_eq!(a.curr_blocks, 1);
            assert_eq!(a.curr_bytes, 256);
            assert_eq!(a.max_blocks, 1);
            assert_eq!(a.max_bytes, 256);
            // These don't get updated until we come down from a peak, or run
            // `Dhat::drop`.
            assert_eq!(a.at_tgmax_blocks, 0);
            assert_eq!(a.at_tgmax_bytes, 0);

            h.tgmax_instant
        };

        unsafe {
            alloc.dealloc(ptr2, layout256);
        }

        let tgmax_instant = {
            let tri: &mut Tri<Globals> = &mut TRI_GLOBALS.lock();
            let g = tri.as_ref_unwrap();
            let h = g.heap.as_ref().unwrap();
            assert_eq!(g.pp_infos.len(), 3);
            assert_eq!(g.backtraces.len(), 3);
            assert_eq!(g.total_blocks, 4);
            assert_eq!(g.total_bytes, 1280);
            assert_eq!(h.live_blocks.len(), 2);
            assert_eq!(h.curr_blocks, 2);
            assert_eq!(h.curr_bytes, 768);
            assert_eq!(h.max_blocks, 3);
            assert_eq!(h.max_bytes, 1024);
            assert_eq!(h.tgmax_instant, tgmax_instant);

            let pp_info = &g.pp_infos[2];
            let a = pp_info.heap.as_ref().unwrap();
            assert_eq!(pp_info.total_blocks, 1);
            assert_eq!(pp_info.total_bytes, 256);
            assert_eq!(a.curr_blocks, 0);
            assert_eq!(a.curr_bytes, 0);
            assert_eq!(a.max_blocks, 1);
            assert_eq!(a.max_bytes, 256);
            // These have been updated because we just came down from a peak.
            assert_eq!(a.at_tgmax_blocks, 1);
            assert_eq!(a.at_tgmax_bytes, 256);

            h.tgmax_instant
        };

        // PpInfo 3
        let _ptr3 = unsafe { alloc.alloc(layout256) };

        // This should have no effect, because we're heap profiling.
        ad_hoc_event(100);

        {
            // Using `finish` here lets us inspect the state of `Globals` after
            // the final processing has been done.
            let g = finish(&mut dhat).unwrap();
            let h = g.heap.as_ref().unwrap();
            assert_eq!(g.pp_infos.len(), 4);
            assert_eq!(g.backtraces.len(), 0); // was consumed by `finish`
            assert_eq!(g.total_blocks, 5);
            assert_eq!(g.total_bytes, 1536);
            assert_eq!(h.live_blocks.len(), 3);
            assert_eq!(h.curr_blocks, 3);
            assert_eq!(h.curr_bytes, 1024);
            assert_eq!(h.max_blocks, 3);
            assert_eq!(h.max_bytes, 1024);
            assert!(h.tgmax_instant > tgmax_instant);

            let pp_info = &g.pp_infos[0];
            let a = pp_info.heap.as_ref().unwrap();
            // These have been updated because `finish` just ran.
            assert_eq!(a.at_tgmax_blocks, 1);
            assert_eq!(a.at_tgmax_bytes, 512);

            let pp_info = &g.pp_infos[1];
            let a = pp_info.heap.as_ref().unwrap();
            // These have been updated because `finish` just ran.
            assert_eq!(a.at_tgmax_blocks, 1);
            assert_eq!(a.at_tgmax_bytes, 256);

            let pp_info = &g.pp_infos[2];
            let a = pp_info.heap.as_ref().unwrap();
            // These have been updated because `finish` just ran.
            assert_eq!(a.at_tgmax_blocks, 0);
            assert_eq!(a.at_tgmax_bytes, 0);

            let pp_info = &g.pp_infos[3];
            let a = pp_info.heap.as_ref().unwrap();
            assert_eq!(pp_info.total_blocks, 1);
            assert_eq!(pp_info.total_bytes, 256);
            assert_eq!(a.curr_blocks, 1);
            assert_eq!(a.curr_bytes, 256);
            assert_eq!(a.max_blocks, 1);
            assert_eq!(a.max_bytes, 256);
            // These have been updated because `finish` just ran.
            assert_eq!(a.at_tgmax_blocks, 1);
            assert_eq!(a.at_tgmax_bytes, 256);
        }

        // To avoid panic when `dhat` is dropped, yuk.
        std::mem::forget(dhat);
    }

    #[test]
    #[serial] // because it involves global state
    fn ad_hoc() {
        reset_tri_globals();

        ad_hoc_event(50); // no-op
        let _dhat = start_ad_hoc_profiling();
        ad_hoc_event(100); // PpInfo 0
        for _ in 0..3 {
            ad_hoc_event(200); // PpInfo 1
        }

        // This should have no effect, because we're ad hoc profiling.
        let _v: Vec<u8> = Vec::with_capacity(100);

        {
            let tri: &mut Tri<Globals> = &mut TRI_GLOBALS.lock();
            let g = tri.as_ref_unwrap();
            assert!(matches!(g.heap, None));
            assert_eq!(g.pp_infos.len(), 2);
            assert_eq!(g.backtraces.len(), 2);
            assert_eq!(g.total_blocks, 4);
            assert_eq!(g.total_bytes, 700);
            let pp_info = &g.pp_infos[0];
            assert!(matches!(pp_info.heap, None));
            assert_eq!(pp_info.total_blocks, 1);
            assert_eq!(pp_info.total_bytes, 100);

            let pp_info = &g.pp_infos[1];
            assert!(matches!(pp_info.heap, None));
            assert_eq!(pp_info.total_blocks, 3);
            assert_eq!(pp_info.total_bytes, 600);
        }
    }
}

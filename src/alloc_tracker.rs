//use dhat::Alloc;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        ALLOCATED.fetch_sub(layout.size(), Ordering::SeqCst);
        System.dealloc(ptr, layout)
    }
}

pub fn get_allocated() -> usize {
    ALLOCATED.load(Ordering::SeqCst)
}

//Live Alloc Tracker
#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

//Uncomment this to use DHat HeapDump
//#[global_allocator]
//static ALLOCATOR: Alloc = Alloc;
use core::alloc::{GlobalAlloc, Layout};

use winapi::um::heapapi::{GetProcessHeap, HeapAlloc, HeapFree};

struct Allocator;

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        HeapAlloc(GetProcessHeap(), 0, layout.size()) as _
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        HeapFree(GetProcessHeap(), 0, ptr as _);
    }
}

#[global_allocator]
static ALLOCATOR: Allocator = Allocator;

#[alloc_error_handler]
fn error_handler(_: core::alloc::Layout) -> ! {
    super::process::fatal("out of memory");
}

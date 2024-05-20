#![no_std]
#![feature(stdarch_nvptx)]
#![feature(asm_experimental_arch)]

use core::arch::{asm, nvptx};
use core::ffi::CStr;
use core::mem::transmute;
use core::panic::PanicInfo;
use core::ptr::null;

#[repr(C)]
struct PanicFmt<'a>(&'a CStr, u32, u32);

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    unsafe {
        nvptx::vprintf("CUDA code panicked :(".as_ptr(), null());
        if let Some(loc) = info.location() {
            let mut buffer = [0; 64];
            let len = loc.file().len().min(63);
            buffer[..len].copy_from_slice(&loc.file().as_bytes()[..len]);
            let str = CStr::from_bytes_until_nul(&buffer).unwrap();
            nvptx::vprintf(
                " (in %s at %d:%d)".as_ptr(),
                transmute(&PanicFmt(str, loc.line(), loc.column())),
            );
        }
        nvptx::vprintf("\n".as_ptr(), null());
        nvptx::trap();
    }
}

pub mod math {
    // libdevice bindings
    extern "C" {
        #[link_name = "__nv_fmaf"]
        pub fn fma(x: f32, y: f32, z: f32) -> f32;
        #[link_name = "__nv_cbrtf"]
        pub fn cbrt(x: f32) -> f32;
        #[link_name = "__nv_powf"]
        pub fn powf(x: f32, y: f32) -> f32;
        #[link_name = "__nv_fabsf"]
        pub fn abs(x: f32) -> f32;
        #[link_name = "__nv_roundf"]
        pub fn round(x: f32) -> f32;
    }
}

pub const WARP_SIZE: usize = 32;

/// Local id of a thread in a warp
#[inline]
pub fn lane() -> u32 {
    let out;
    unsafe {
        asm!(
        "mov.u32 {out}, %laneid;",
        out = out(reg32) out
        )
    }
    out
}

#[inline]
pub fn syncthreads() {
    unsafe {
        nvptx::_syncthreads();
    }
}

#[inline]
pub fn coords_1d() -> usize {
    unsafe {
        let tx = nvptx::_thread_idx_x() as usize;
        let bx = nvptx::_block_idx_x() as usize;
        let bdx = nvptx::_block_dim_x() as usize;
        bx * bdx + tx
    }
}

#[inline]
pub fn coords_2d() -> (usize, usize) {
    unsafe {
        let tx = nvptx::_thread_idx_x() as usize;
        let ty = nvptx::_thread_idx_y() as usize;
        let bx = nvptx::_block_idx_x() as usize;
        let by = nvptx::_block_idx_y() as usize;
        let bdx = nvptx::_block_dim_x() as usize;
        let bdy = nvptx::_block_dim_y() as usize;
        (bx * bdx + tx, by * bdy + ty)
    }
}

#[inline]
pub fn coords_3d() -> (usize, usize, usize) {
    unsafe {
        let tx = nvptx::_thread_idx_x() as usize;
        let ty = nvptx::_thread_idx_y() as usize;
        let tz = nvptx::_thread_idx_z() as usize;
        let bx = nvptx::_block_idx_x() as usize;
        let by = nvptx::_block_idx_y() as usize;
        let bz = nvptx::_block_idx_z() as usize;
        let bdx = nvptx::_block_dim_x() as usize;
        let bdy = nvptx::_block_dim_y() as usize;
        let bdz = nvptx::_block_dim_z() as usize;
        (bx * bdx + tx, by * bdy + ty, bz * bdz + tz)
    }
}

macro_rules! shfl_down_sync_t {
    ($ty:ty) => {
        #[inline(always)]
        pub unsafe fn shfl_down_sync(mask: u32, value: $ty, offset: u32, width: u32) -> $ty {
            let out;
            asm!(
            "shfl.sync.down.b32 {out}, {v}, {offset}, {width}, {mask};",
            out = out(reg32) out,
            v = in(reg32) value,
            offset = in(reg32) offset,
            width = in(reg32) width,
            mask = in(reg32) mask
            );
            out
        }
    }
}

shfl_down_sync_t!(f32);

fn tex2D() {
    unsafe {
        asm!(
        ""
        )
    }
}

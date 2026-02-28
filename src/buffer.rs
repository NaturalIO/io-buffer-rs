use super::utils::{safe_copy, set_zero};
use libc::{c_void, free, malloc, posix_memalign};
use nix::errno::Errno;
use std::slice;
use std::{
    fmt,
    ops::{Deref, DerefMut},
    ptr::{NonNull, null_mut},
};

/// Buffer is a static type,  size and cap (max to i32). Memory footprint is only 16B.
///
/// Can obtain from alloc (uninitialized, mutable and owned),
///
/// or wrap a raw pointer from c code (not owned,  mutable or immutable),
///
/// or convert `From<Vec<u8>>` (mutable and owned), and `To<Vec<u8>>`
///
/// When Clone, will copy the contain into a new Buffer.
#[repr(C)]
pub struct Buffer {
    buf_ptr: NonNull<c_void>,
    /// the highest bit of `size` represents `owned`
    pub(crate) size: u32,
    /// the highest bit of `cap` represents `mutable`
    pub(crate) cap: u32,
}

impl fmt::Debug for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "buffer {:p} size {}", self.get_raw(), self.len())
    }
}

unsafe impl Send for Buffer {}

unsafe impl Sync for Buffer {}

pub const MIN_ALIGN: u32 = 512;
pub const MAX_BUFFER_SIZE: usize = 1 << 31;

fn is_aligned(offset: usize, size: usize) -> bool {
    return (offset & (MIN_ALIGN as usize - 1) == 0) && (size & (MIN_ALIGN as usize - 1) == 0);
}

impl Buffer {
    /// Allocate mutable and owned aligned buffer for aio by posix_memalign(),
    /// with size set to capacity.
    ///
    /// **NOTE**: Be aware that buffer allocated is not initialized.
    ///
    /// `size`: must be larger than zero
    #[inline]
    pub fn aligned(size: i32) -> Result<Buffer, Errno> {
        let mut _buf = Self::_alloc(MIN_ALIGN, size)?;
        #[cfg(all(feature = "fail", feature = "rand"))]
        fail::fail_point!("alloc_buf", |_| {
            rand_buffer(&mut _buf);
            return Ok(_buf);
        });
        return Ok(_buf);
    }

    /// Allocate mutable and owned aligned buffer for aio by posix_memalign(),
    /// with size set to capacity.
    ///
    /// **NOTE**: Be aware that buffer allocated is not initialized.
    ///
    /// `size`: must be larger than zero
    ///
    /// `align`: normally 512 or 4096
    #[inline]
    pub fn aligned_by(size: i32, align: u32) -> Result<Buffer, Errno> {
        let mut _buf = Self::_alloc(align, size)?;
        #[cfg(all(feature = "fail", feature = "rand"))]
        fail::fail_point!("alloc_buf", |_| {
            rand_buffer(&mut _buf);
            return Ok(_buf);
        });
        return Ok(_buf);
    }

    /// Allocate mutable and owned non-aligned Buffer by malloc(),
    /// with size set to capacity.
    ///
    /// **NOTE**: Be aware that buffer allocated is not initialized.
    ///
    /// `size`: must be larger than zero
    #[inline]
    pub fn alloc(size: i32) -> Result<Buffer, Errno> {
        let mut _buf = Self::_alloc(0, size)?;
        #[cfg(all(feature = "fail", feature = "rand"))]
        fail::fail_point!("alloc_buf", |_| {
            rand_buffer(&mut _buf);
            return Ok(_buf);
        });
        return Ok(_buf);
    }

    /// Allocate a buffer.
    ///
    /// `size`: must be larger than zero
    #[inline]
    fn _alloc(align: u32, size: i32) -> Result<Self, Errno> {
        assert!(size > 0);
        let mut ptr: *mut c_void = null_mut();
        if align > 0 {
            debug_assert!((align & (MIN_ALIGN - 1)) == 0);
            debug_assert!((size as u32 & (align - 1)) == 0);
            unsafe {
                let res = posix_memalign(&mut ptr, align as libc::size_t, size as libc::size_t);
                if res != 0 {
                    return Err(Errno::ENOMEM);
                }
            }
        } else {
            ptr = unsafe { malloc(size as libc::size_t) };
            if ptr.is_null() {
                return Err(Errno::ENOMEM);
            }
        }
        // owned == true
        let _size = size as u32 | MAX_BUFFER_SIZE as u32;
        // mutable == true
        let _cap = _size;
        Ok(Self { buf_ptr: unsafe { NonNull::new_unchecked(ptr) }, size: _size, cap: _cap })
    }

    /// Wrap a mutable buffer passed from c code, without owner ship.
    ///
    /// **NOTE**: will not free on drop. You have to ensure the buffer valid throughout the lifecycle.
    ///
    /// `size`: must be larger than or equal to zero.
    #[inline]
    pub fn from_c_ref_mut(ptr: *mut c_void, size: i32) -> Self {
        assert!(size >= 0);
        assert!(!ptr.is_null());
        // owned == false
        // mutable == true
        let _cap = size as u32 | MAX_BUFFER_SIZE as u32;
        Self { buf_ptr: unsafe { NonNull::new_unchecked(ptr) }, size: size as u32, cap: _cap }
    }

    /// Wrap a const buffer passed from c code, without owner ship.
    ///
    /// **NOTE**: will not free on drop. You have to ensure the buffer valid throughout the lifecycle
    ///
    /// `size`: must be larger than or equal to zero.
    #[inline]
    pub fn from_c_ref_const(ptr: *const c_void, size: i32) -> Self {
        assert!(size >= 0);
        assert!(!ptr.is_null());
        // owned == false
        // mutable == false
        Self {
            buf_ptr: unsafe { NonNull::new_unchecked(ptr as *mut c_void) },
            size: size as u32,
            cap: size as u32,
        }
    }

    /// Tell whether the Buffer has true 'static lifetime.
    #[inline(always)]
    pub fn is_owned(&self) -> bool {
        self.size & (MAX_BUFFER_SIZE as u32) != 0
    }

    /// Tell whether the Buffer can as_mut().
    #[inline(always)]
    pub fn is_mutable(&self) -> bool {
        self.cap & (MAX_BUFFER_SIZE as u32) != 0
    }

    /// Return the buffer's size.
    #[inline(always)]
    pub fn len(&self) -> usize {
        let size = self.size & (MAX_BUFFER_SIZE as u32 - 1);
        size as usize
    }

    /// Return the memory capacity managed by buffer's ptr
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        let cap = self.cap & (MAX_BUFFER_SIZE as u32 - 1);
        cap as usize
    }

    /// Change the buffer's size, the same as `Vec::set_len()`. Panics when len > capacity
    #[inline(always)]
    pub fn set_len(&mut self, len: usize) {
        assert!(len < MAX_BUFFER_SIZE, "size {} >= {} is not supported", len, MAX_BUFFER_SIZE);
        assert!(len <= self.cap as usize, "size {} must be <= {}", len, self.cap);
        let owned: u32 = self.size & MAX_BUFFER_SIZE as u32;
        self.size = owned | len as u32;
    }

    #[inline(always)]
    pub fn as_ref(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.buf_ptr.as_ptr() as *const u8, self.len()) }
    }

    /// On debug mode, will panic if the Buffer is not owned [Buffer::from_c_ref_const()]
    ///
    /// On release will skip the check for speed.
    #[inline(always)]
    pub fn as_mut(&mut self) -> &mut [u8] {
        #[cfg(debug_assertions)]
        {
            if !self.is_mutable() {
                panic!("Cannot change a mutable buffer")
            }
        }
        unsafe { slice::from_raw_parts_mut(self.buf_ptr.as_ptr() as *mut u8, self.len()) }
    }

    /// Check this buffer usable by aio. True when get from `Buffer::aligned()`.
    #[inline(always)]
    pub fn is_aligned(&self) -> bool {
        is_aligned(self.buf_ptr.as_ptr() as usize, self.capacity())
    }

    /// Get buffer raw pointer
    #[inline]
    pub fn get_raw(&self) -> *const u8 {
        self.buf_ptr.as_ptr() as *const u8
    }

    /// Get buffer raw mut pointer
    #[inline]
    pub fn get_raw_mut(&mut self) -> *mut u8 {
        self.buf_ptr.as_ptr() as *mut u8
    }

    /// Copy from src u8 slice into self[offset..].
    ///
    /// **NOTE**: will not do memset.
    ///
    /// # Argument
    ///
    ///  * offset: Address of this buffer to start filling.
    ///
    /// # Panic
    ///
    /// If offset >= self.len(), will panic
    #[inline]
    pub fn copy_from(&mut self, offset: usize, src: &[u8]) {
        let size = self.len();
        let dst = self.as_mut();
        if offset > 0 {
            assert!(offset < size);
            safe_copy(&mut dst[offset..], src);
        } else {
            safe_copy(dst, src);
        }
    }

    /// Copy from another u8 slice into self[offset..], and memset the rest part.
    ///
    /// Argument:
    ///
    ///  * offset: Address of this buffer to start filling.
    #[inline]
    pub fn copy_and_clean(&mut self, offset: usize, other: &[u8]) {
        let end: usize;
        let size = self.len();
        let dst = self.as_mut();
        assert!(offset < size);
        if offset > 0 {
            set_zero(&mut dst[0..offset]);
            end = offset + safe_copy(&mut dst[offset..], other);
        } else {
            end = safe_copy(dst, other);
        }
        if size > end {
            set_zero(&mut dst[end..]);
        }
    }

    /// Fill this buffer with zero
    #[inline]
    pub fn zero(&mut self) {
        set_zero(self);
    }

    /// Fill specified region of buffer[offset..(offset+len)] with zero
    #[inline]
    pub fn set_zero(&mut self, offset: usize, len: usize) {
        let _len = self.len();
        let mut end = offset + len;
        if end > _len {
            end = _len;
        }
        let buf = self.as_mut();
        if offset > 0 || end < _len {
            set_zero(&mut buf[offset..end]);
        } else {
            set_zero(buf);
        }
    }
}

/// Allocates a new memory with the same size and clone the content.
/// If original buffer is a c reference, will get a owned buffer after clone().
impl Clone for Buffer {
    fn clone(&self) -> Self {
        let mut new_buf = if self.is_aligned() {
            Self::aligned(self.capacity() as i32).unwrap()
        } else {
            Self::alloc(self.capacity() as i32).unwrap()
        };
        if self.len() != self.capacity() {
            new_buf.set_len(self.len());
        }
        safe_copy(new_buf.as_mut(), self.as_ref());
        new_buf
    }
}

/// Automatically free on drop when buffer is owned
impl Drop for Buffer {
    fn drop(&mut self) {
        if self.is_owned() {
            unsafe {
                free(self.buf_ptr.as_ptr());
            }
        }
    }
}

/// Convert a owned Buffer to `Vec<u8>`. Panic when buffer is a ref.
impl Into<Vec<u8>> for Buffer {
    fn into(mut self) -> Vec<u8> {
        if !self.is_owned() {
            panic!("buffer is c ref, not owned");
        }
        // Change to not owned, to prevent drop()
        self.size &= MAX_BUFFER_SIZE as u32 - 1;
        return unsafe {
            Vec::<u8>::from_raw_parts(self.buf_ptr.as_ptr() as *mut u8, self.len(), self.capacity())
        };
    }
}

/// Convert `Vec<u8>` to Buffer, inherit the size and cap of Vec.
impl From<Vec<u8>> for Buffer {
    fn from(buf: Vec<u8>) -> Self {
        let size = buf.len();
        let cap = buf.capacity();
        assert!(size < MAX_BUFFER_SIZE, "size {} >= {} is not supported", size, MAX_BUFFER_SIZE);
        assert!(cap < MAX_BUFFER_SIZE, "cap {} >= {} is not supported", cap, MAX_BUFFER_SIZE);
        // owned == true
        let _size = size as u32 | MAX_BUFFER_SIZE as u32;
        // mutable == true
        let _cap = cap as u32 | MAX_BUFFER_SIZE as u32;
        Buffer {
            buf_ptr: unsafe { NonNull::new_unchecked(buf.leak().as_mut_ptr() as *mut c_void) },
            size: _size,
            cap: _cap,
        }
    }
}

impl Deref for Buffer {
    type Target = [u8];

    #[inline]
    fn deref(&self) -> &[u8] {
        self.as_ref()
    }
}

impl AsRef<[u8]> for Buffer {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.as_ref()
    }
}

/// On debug mode, will panic if the Buffer is not owned [Buffer::from_c_ref_const()]
///
/// On release will skip the check for speed.
impl AsMut<[u8]> for Buffer {
    #[inline]
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

impl DerefMut for Buffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

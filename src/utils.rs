/// Only copy the 0..min(dst, src) of src to dst, return the bytes copied.
#[inline]
pub fn safe_copy(dst: &mut [u8], src: &[u8]) -> usize {
    let dst_len = dst.len();
    let src_len = src.len();
    if src_len > dst_len {
        dst.copy_from_slice(&src[0..dst_len]);
        return dst_len;
    } else if src_len < dst_len {
        dst[0..src_len].copy_from_slice(src);
        return src_len;
    } else {
        dst.copy_from_slice(src);
        return dst_len;
    }
}

/// Set a buffer to zero
#[inline(always)]
pub fn set_zero(dst: &mut [u8]) {
    unsafe {
        libc::memset(dst.as_mut_ptr() as *mut libc::c_void, 0, dst.len());
    }
}

/// Produce ascii random string
#[cfg(feature = "rand")]
#[inline]
pub fn rand_buffer<T: AsMut<[u8]>>(dst: &mut T) {
    let s: &mut [u8] = dst.as_mut();
    let len = s.len();
    for i in 0..len {
        s[i] = fastrand::alphanumeric() as u8;
    }
}

/// Test whether a buffer is all set to zero
#[inline(always)]
pub fn is_all_zero(s: &[u8]) -> bool {
    for c in s {
        if *c != 0 {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {

    extern crate md5;
    use super::*;

    #[test]
    fn test_safe_copy() {
        let buf1: [u8; 10] = [0; 10];
        let mut buf2: [u8; 10] = [1; 10];
        let mut buf3: [u8; 10] = [2; 10];
        let zero: usize = 0;
        // dst zero size copy should be protected
        assert_eq!(0, safe_copy(&mut buf2[0..zero], &buf3));
        assert_eq!(&buf2, &[1; 10]);
        assert_eq!(0, safe_copy(&mut buf2[10..], &buf3));
        assert_eq!(&buf2, &[1; 10]);
        // src zero size copy should be protected

        // src zero size copy ?
        assert_eq!(10, safe_copy(&mut buf2, &buf1));
        assert_eq!(buf1, buf2);
        assert_eq!(5, safe_copy(&mut buf2[5..], &buf3));
        assert_eq!(buf1[0..5], buf2[0..5]);
        assert_eq!(buf2[5..], buf3[5..]);
        assert_eq!(5, safe_copy(&mut buf3[0..5], &buf1));
        assert_eq!(buf2, buf3);
    }

    #[cfg(feature = "rand")]
    #[test]
    fn test_rand_buffer() {
        let mut buf1: [u8; 10] = [0; 10];
        let mut buf2: [u8; 10] = [0; 10];
        rand_buffer(&mut buf1);
        rand_buffer(&mut buf2);
        assert!(md5::compute(&buf1) != md5::compute(&buf2));
    }

    #[test]
    fn test_set_zero() {
        let mut buf1: [u8; 10] = [1; 10];
        set_zero(&mut buf1);
        for i in &buf1 {
            assert_eq!(*i, 0)
        }
    }
}

/// Compute the Xbox swizzled memory index for pixel (x, y) in an image of size (w, h).
/// Uses a Morton/Z-order curve by interleaving bits of x and y.
pub fn swizzle_index(mut x: usize, mut y: usize, mut w: usize, mut h: usize) -> usize {
    let mut addr = 0usize;
    let mut bit = 0u32;
    while w > 1 || h > 1 {
        if h > 1 {
            addr |= (y & 1) << bit;
            y >>= 1;
            h >>= 1;
            bit += 1;
        }
        if w > 1 {
            addr |= (x & 1) << bit;
            x >>= 1;
            w >>= 1;
            bit += 1;
        }
    }
    addr
}

/// Unswizzle an Xbox-swizzled raw pixel buffer (in-place reorder).
/// `bpp` is bytes per pixel (2 for 16-bit formats, 4 for 32-bit).
pub fn unswizzle(src: &[u8], width: usize, height: usize, bpp: usize) -> Vec<u8> {
    let mut out = vec![0u8; width * height * bpp];
    for y in 0..height {
        for x in 0..width {
            let src_idx = swizzle_index(x, y, width, height);
            let dst_off = (y * width + x) * bpp;
            let src_off = src_idx * bpp;
            out[dst_off..dst_off + bpp].copy_from_slice(&src[src_off..src_off + bpp]);
        }
    }
    out
}

/// Rotate a pixel buffer 90° CCW: src is sw×sh, dst is sh×sw.
pub fn rotate_ccw(src: &[u8], sw: usize, sh: usize, bpp: usize) -> Vec<u8> {
    let mut out = vec![0u8; sw * sh * bpp];
    for sy in 0..sh {
        for sx in 0..sw {
            let dx = sy;
            let dy = sw - 1 - sx;
            let src_off = (sy * sw + sx) * bpp;
            let dst_off = (dy * sh + dx) * bpp;
            out[dst_off..dst_off + bpp].copy_from_slice(&src[src_off..src_off + bpp]);
        }
    }
    out
}

/// Rotate a pixel buffer 90° CW: src is w×h, dst is h×w (new_w=h, new_h=w).
/// new(r,c) = old(h-1-c, r), r in 0..w, c in 0..h.
pub fn rotate_cw(src: &[u8], w: usize, h: usize, bpp: usize) -> Vec<u8> {
    let mut out = vec![0u8; w * h * bpp];
    for r in 0..w {
        for c in 0..h {
            let src_off = ((h - 1 - c) * w + r) * bpp;
            let dst_off = (r * h + c) * bpp;
            out[dst_off..dst_off + bpp].copy_from_slice(&src[src_off..src_off + bpp]);
        }
    }
    out
}

/// Unswizzle a non-square image stored with swapped dims, then rotate CCW to restore w×h.
/// For square images, falls back to plain unswizzle.
pub fn unswizzle_rect(src: &[u8], width: usize, height: usize, bpp: usize) -> Vec<u8> {
    if width == height {
        return unswizzle(src, width, height, bpp);
    }
    // Stored as height×width (swapped), unswizzle at stored dims then rotate CCW
    let unswiz = unswizzle(src, height, width, bpp);
    rotate_ccw(&unswiz, height, width, bpp)
}

/// Rotate CW then swizzle for non-square images. For square, plain swizzle.
pub fn swizzle_rect(src: &[u8], width: usize, height: usize, bpp: usize) -> Vec<u8> {
    if width == height {
        return swizzle(src, width, height, bpp);
    }
    // Rotate CW to get stored dims (height×width), then swizzle
    let rotated = rotate_cw(src, width, height, bpp);
    swizzle(&rotated, height, width, bpp)
}

/// Swizzle a linear pixel buffer into Xbox swizzled order.
/// `bpp` is bytes per pixel (2 for 16-bit formats, 4 for 32-bit).
pub fn swizzle(src: &[u8], width: usize, height: usize, bpp: usize) -> Vec<u8> {
    let mut out = vec![0u8; width * height * bpp];
    for y in 0..height {
        for x in 0..width {
            let dst_idx = swizzle_index(x, y, width, height);
            let src_off = (y * width + x) * bpp;
            let dst_off = dst_idx * bpp;
            out[dst_off..dst_off + bpp].copy_from_slice(&src[src_off..src_off + bpp]);
        }
    }
    out
}

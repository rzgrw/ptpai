/// Low-level SIMD (ARM NEON / x86_64 AVX2) vector operations for zero-copy activation compute.
/// On Apple Silicon (aarch64), this uses hardware 128-bit vector registers directly.
/// On Linux/x86_64, this uses 256-bit AVX2 + FMA hardware vector registers.

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// High-throughput dot-product kernel.
/// Computes sum(a[i] * b[i]) using 128-bit NEON or 256-bit AVX2 registers with loop unrolling.
#[inline(always)]
pub fn vector_dot_product(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len(), "Vector lengths must match");
    let len = a.len();

    #[cfg(target_arch = "aarch64")]
    unsafe {
        neon_dot_product_f32(a.as_ptr(), b.as_ptr(), len)
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        if is_x86_feature_detected!("avx2") && is_x86_feature_detected!("fma") {
            avx2_dot_product_f32(a.as_ptr(), b.as_ptr(), len)
        } else {
            portable_dot_product_f32(a, b)
        }
    }

    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        portable_dot_product_f32(a, b)
    }
}

/// Quantize FP32 activations to INT8 with dynamic scaling factor.
/// Returns scale
#[inline(always)]
pub fn quantize_activation_fp32_to_int8(src: &[f32], dst: &mut [i8]) -> f32 {
    assert_eq!(src.len(), dst.len(), "Buffer sizes must match");

    // Find absolute maximum
    let mut max_abs = 1e-8f32;
    for &val in src {
        let abs = val.abs();
        if abs > max_abs {
            max_abs = abs;
        }
    }

    let scale = max_abs / 127.0;
    let inv_scale = 127.0 / max_abs;

    #[cfg(target_arch = "aarch64")]
    unsafe {
        let v_inv_scale = vdupq_n_f32(inv_scale);
        let mut i = 0;
        while i + 4 <= src.len() {
            let v_src = vld1q_f32(src.as_ptr().add(i));
            let v_scaled = vmulq_f32(v_src, v_inv_scale);
            let v_int32 = vcvtq_s32_f32(v_scaled);

            let ptr = dst.as_mut_ptr().add(i);
            *ptr.add(0) = vgetq_lane_s32(v_int32, 0).clamp(-128, 127) as i8;
            *ptr.add(1) = vgetq_lane_s32(v_int32, 1).clamp(-128, 127) as i8;
            *ptr.add(2) = vgetq_lane_s32(v_int32, 2).clamp(-128, 127) as i8;
            *ptr.add(3) = vgetq_lane_s32(v_int32, 3).clamp(-128, 127) as i8;
            i += 4;
        }

        while i < src.len() {
            dst[i] = (src[i] * inv_scale).clamp(-128.0, 127.0) as i8;
            i += 1;
        }
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        for (s, d) in src.iter().zip(dst.iter_mut()) {
            *d = (*s * inv_scale).clamp(-128.0, 127.0) as i8;
        }
    }

    scale
}

/// Dequantize INT8 activations back to FP32 using scale factor
#[inline(always)]
pub fn dequantize_activation_int8_to_fp32(src: &[i8], dst: &mut [f32], scale: f32) {
    assert_eq!(src.len(), dst.len(), "Buffer sizes must match");

    #[cfg(target_arch = "aarch64")]
    unsafe {
        let v_scale = vdupq_n_f32(scale);
        let mut i = 0;
        while i + 4 <= src.len() {
            let v_i32 = vsetq_lane_s32(
                src[i] as i32,
                vsetq_lane_s32(
                    src[i + 1] as i32,
                    vsetq_lane_s32(
                        src[i + 2] as i32,
                        vdupq_n_s32(src[i + 3] as i32),
                        2,
                    ),
                    1,
                ),
                0,
            );
            let v_f32 = vcvtq_f32_s32(v_i32);
            let v_res = vmulq_f32(v_f32, v_scale);
            vst1q_f32(dst.as_mut_ptr().add(i), v_res);
            i += 4;
        }
        while i < src.len() {
            dst[i] = (src[i] as f32) * scale;
            i += 1;
        }
    }

    #[cfg(not(target_arch = "aarch64"))]
    {
        for (s, d) in src.iter().zip(dst.iter_mut()) {
            *d = (*s as f32) * scale;
        }
    }
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn neon_dot_product_f32(mut a_ptr: *const f32, mut b_ptr: *const f32, mut len: usize) -> f32 {
    unsafe {
        let mut sum0 = vdupq_n_f32(0.0);
        let mut sum1 = vdupq_n_f32(0.0);
        let mut sum2 = vdupq_n_f32(0.0);
        let mut sum3 = vdupq_n_f32(0.0);

        while len >= 16 {
            let a0 = vld1q_f32(a_ptr);
            let b0 = vld1q_f32(b_ptr);
            sum0 = vfmaq_f32(sum0, a0, b0);

            let a1 = vld1q_f32(a_ptr.add(4));
            let b1 = vld1q_f32(b_ptr.add(4));
            sum1 = vfmaq_f32(sum1, a1, b1);

            let a2 = vld1q_f32(a_ptr.add(8));
            let b2 = vld1q_f32(b_ptr.add(8));
            sum2 = vfmaq_f32(sum2, a2, b2);

            let a3 = vld1q_f32(a_ptr.add(12));
            let b3 = vld1q_f32(b_ptr.add(12));
            sum3 = vfmaq_f32(sum3, a3, b3);

            a_ptr = a_ptr.add(16);
            b_ptr = b_ptr.add(16);
            len -= 16;
        }

        let sum_pair1 = vaddq_f32(sum0, sum1);
        let sum_pair2 = vaddq_f32(sum2, sum3);
        let mut total_sum = vaddq_f32(sum_pair1, sum_pair2);

        while len >= 4 {
            let a = vld1q_f32(a_ptr);
            let b = vld1q_f32(b_ptr);
            total_sum = vfmaq_f32(total_sum, a, b);
            a_ptr = a_ptr.add(4);
            b_ptr = b_ptr.add(4);
            len -= 4;
        }

        let mut acc = vaddvq_f32(total_sum);

        while len > 0 {
            acc += *a_ptr * *b_ptr;
            a_ptr = a_ptr.add(1);
            b_ptr = b_ptr.add(1);
            len -= 1;
        }

        acc
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn avx2_dot_product_f32(mut a_ptr: *const f32, mut b_ptr: *const f32, mut len: usize) -> f32 {
    unsafe {
        let mut sum0 = _mm256_setzero_ps();
        let mut sum1 = _mm256_setzero_ps();

        while len >= 16 {
            let a0 = _mm256_loadu_ps(a_ptr);
            let b0 = _mm256_loadu_ps(b_ptr);
            sum0 = _mm256_fmadd_ps(a0, b0, sum0);

            let a1 = _mm256_loadu_ps(a_ptr.add(8));
            let b1 = _mm256_loadu_ps(b_ptr.add(8));
            sum1 = _mm256_fmadd_ps(a1, b1, sum1);

            a_ptr = a_ptr.add(16);
            b_ptr = b_ptr.add(16);
            len -= 16;
        }

        let mut total_sum = _mm256_add_ps(sum0, sum1);

        while len >= 8 {
            let a = _mm256_loadu_ps(a_ptr);
            let b = _mm256_loadu_ps(b_ptr);
            total_sum = _mm256_fmadd_ps(a, b, total_sum);
            a_ptr = a_ptr.add(8);
            b_ptr = b_ptr.add(8);
            len -= 8;
        }

        let hi128 = _mm256_extractf128_ps(total_sum, 1);
        let lo128 = _mm256_castps256_ps128(total_sum);
        let sum128 = _mm_add_ps(lo128, hi128);
        let shuf = _mm_movehl_ps(sum128, sum128);
        let sum64 = _mm_add_ps(sum128, shuf);
        let shuf2 = _mm_shuffle_ps(sum64, sum64, 1);
        let res = _mm_add_ss(sum64, shuf2);
        let mut acc = _mm_cvtss_f32(res);

        while len > 0 {
            acc += *a_ptr * *b_ptr;
            a_ptr = a_ptr.add(1);
            b_ptr = b_ptr.add(1);
            len -= 1;
        }

        acc
    }
}

#[allow(dead_code)]
#[inline(always)]
fn portable_dot_product_f32(a: &[f32], b: &[f32]) -> f32 {
    let mut sum = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        sum += x * y;
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_dot_product() {
        let a = vec![1.0f32; 1024];
        let b = vec![2.0f32; 1024];
        let res = vector_dot_product(&a, &b);
        assert_eq!(res, 2048.0);
    }

    #[test]
    fn test_quantization_roundtrip() {
        let original = vec![0.5f32, -0.75f32, 1.2f32, -1.8f32, 0.0f32, 0.99f32, -0.01f32, 1.5f32];
        let mut quantized = vec![0i8; original.len()];
        let mut reconstructed = vec![0.0f32; original.len()];

        let scale = quantize_activation_fp32_to_int8(&original, &mut quantized);
        dequantize_activation_int8_to_fp32(&quantized, &mut reconstructed, scale);

        for (orig, rec) in original.iter().zip(reconstructed.iter()) {
            let error = (orig - rec).abs();
            assert!(error < 0.05, "Quantization error too large: orig={}, rec={}", orig, rec);
        }
    }
}

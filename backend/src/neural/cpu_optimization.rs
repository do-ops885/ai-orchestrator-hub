#![allow(unsafe_code)]
use std::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

/// CPU optimization module for high-performance agent processing
/// Implements SIMD vectorization and quantized operations for maximum efficiency

#[derive(Debug, Clone)]
pub struct CpuOptimizer {
    pub simd_support: SimdSupport,
    pub quantization_enabled: bool,
    pub cache_line_size: usize,
    pub preferred_vector_width: usize,
}

#[derive(Debug, Clone)]
pub struct SimdSupport {
    pub avx2: bool,
    pub avx512: bool,
    pub sse4_1: bool,
    pub neon: bool,
}

impl CpuOptimizer {
    pub fn new() -> Self {
        Self {
            simd_support: Self::detect_simd_support(),
            quantization_enabled: true,
            cache_line_size: Self::detect_cache_line_size(),
            preferred_vector_width: Self::detect_vector_width(),
        }
    }

    /// Detect available SIMD instruction sets
    fn detect_simd_support() -> SimdSupport {
        #[cfg(target_arch = "x86_64")]
        {
            SimdSupport {
                avx2: is_x86_feature_detected!("avx2"),
                avx512: is_x86_feature_detected!("avx512f"),
                sse4_1: is_x86_feature_detected!("sse4.1"),
                neon: false,
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            SimdSupport {
                avx2: false,
                avx512: false,
                sse4_1: false,
                neon: std::arch::is_aarch64_feature_detected!("neon"),
            }
        }
        
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            SimdSupport {
                avx2: false,
                avx512: false,
                sse4_1: false,
                neon: false,
            }
        }
    }

    fn detect_cache_line_size() -> usize {
        // Most modern CPUs use 64-byte cache lines
        64
    }

    fn detect_vector_width() -> usize {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx512f") {
                64 // 512 bits = 64 bytes
            } else if is_x86_feature_detected!("avx2") {
                32 // 256 bits = 32 bytes
            } else {
                16 // 128 bits = 16 bytes (SSE)
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            16 // 128 bits = 16 bytes (NEON)
        }
        
        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            8 // Fallback to scalar operations
        }
    }
}

/// High-performance vectorized operations for semantic processing
#[allow(dead_code)]
pub struct VectorizedOps;

#[allow(dead_code)]
impl VectorizedOps {
    /// Compute dot product using optimal SIMD instructions
    pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
        assert_eq!(a.len(), b.len());
        
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                return unsafe { Self::dot_product_avx2(a, b) };
            } else if is_x86_feature_detected!("sse4.1") {
                return unsafe { Self::dot_product_sse(a, b) };
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            if std::arch::is_aarch64_feature_detected!("neon") {
                return unsafe { Self::dot_product_neon(a, b) };
            }
        }
        
        // Fallback to scalar implementation
        Self::dot_product_scalar(a, b)
    }

    /// AVX2 optimized dot product (x86_64)
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn dot_product_avx2(a: &[f32], b: &[f32]) -> f32 {
        let len = a.len();
        let mut sum = _mm256_setzero_ps();
        
        // Process 8 floats at a time
        let chunks = len / 8;
        for i in 0..chunks {
            let offset = i * 8;
            let va = _mm256_loadu_ps(a.as_ptr().add(offset));
            let vb = _mm256_loadu_ps(b.as_ptr().add(offset));
            let prod = _mm256_mul_ps(va, vb);
            sum = _mm256_add_ps(sum, prod);
        }
        
        // Horizontal sum of the vector
        let sum_high = _mm256_extractf128_ps(sum, 1);
        let sum_low = _mm256_castps256_ps128(sum);
        let sum128 = _mm_add_ps(sum_high, sum_low);
        
        let sum64 = _mm_add_ps(sum128, _mm_movehl_ps(sum128, sum128));
        let sum32 = _mm_add_ss(sum64, _mm_shuffle_ps(sum64, sum64, 0x55));
        
        let mut result = _mm_cvtss_f32(sum32);
        
        // Handle remaining elements
        for i in (chunks * 8)..len {
            result += a[i] * b[i];
        }
        
        result
    }

    /// SSE optimized dot product (x86_64)
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse4.1")]
    unsafe fn dot_product_sse(a: &[f32], b: &[f32]) -> f32 {
        let len = a.len();
        let mut sum = _mm_setzero_ps();
        
        // Process 4 floats at a time
        let chunks = len / 4;
        for i in 0..chunks {
            let offset = i * 4;
            let va = _mm_loadu_ps(a.as_ptr().add(offset));
            let vb = _mm_loadu_ps(b.as_ptr().add(offset));
            let prod = _mm_mul_ps(va, vb);
            sum = _mm_add_ps(sum, prod);
        }
        
        // Horizontal sum
        let sum_high = _mm_movehl_ps(sum, sum);
        let sum_low = _mm_add_ps(sum, sum_high);
        let sum_final = _mm_add_ss(sum_low, _mm_shuffle_ps(sum_low, sum_low, 0x55));
        
        let mut result = _mm_cvtss_f32(sum_final);
        
        // Handle remaining elements
        for i in (chunks * 4)..len {
            result += a[i] * b[i];
        }
        
        result
    }

    /// NEON optimized dot product (ARM64)
    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    unsafe fn dot_product_neon(a: &[f32], b: &[f32]) -> f32 {
        let len = a.len();
        let mut sum = vdupq_n_f32(0.0);
        
        // Process 4 floats at a time
        let chunks = len / 4;
        for i in 0..chunks {
            let offset = i * 4;
            let va = vld1q_f32(a.as_ptr().add(offset));
            let vb = vld1q_f32(b.as_ptr().add(offset));
            let prod = vmulq_f32(va, vb);
            sum = vaddq_f32(sum, prod);
        }
        
        // Horizontal sum
        let sum_pair = vpadd_f32(vget_low_f32(sum), vget_high_f32(sum));
        let result_vec = vpadd_f32(sum_pair, sum_pair);
        let mut result = vget_lane_f32(result_vec, 0);
        
        // Handle remaining elements
        for i in (chunks * 4)..len {
            result += a[i] * b[i];
        }
        
        result
    }

    /// Scalar fallback implementation
    fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    /// Vectorized cosine similarity
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot = Self::dot_product(a, b);
        let norm_a = Self::vector_norm(a);
        let norm_b = Self::vector_norm(b);
        
        if norm_a == 0.0 || norm_b == 0.0 {
            0.0
        } else {
            dot / (norm_a * norm_b)
        }
    }

    /// Vectorized L2 norm calculation
    pub fn vector_norm(v: &[f32]) -> f32 {
        Self::dot_product(v, v).sqrt()
    }

    /// Vectorized element-wise addition
    pub fn vector_add(a: &[f32], b: &[f32], result: &mut [f32]) {
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), result.len());
        
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe { Self::vector_add_avx2(a, b, result) };
                return;
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            if std::arch::is_aarch64_feature_detected!("neon") {
                unsafe { Self::vector_add_neon(a, b, result) };
                return;
            }
        }
        
        // Scalar fallback
        for i in 0..a.len() {
            result[i] = a[i] + b[i];
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn vector_add_avx2(a: &[f32], b: &[f32], result: &mut [f32]) {
        let len = a.len();
        let chunks = len / 8;
        
        for i in 0..chunks {
            let offset = i * 8;
            let va = _mm256_loadu_ps(a.as_ptr().add(offset));
            let vb = _mm256_loadu_ps(b.as_ptr().add(offset));
            let sum = _mm256_add_ps(va, vb);
            _mm256_storeu_ps(result.as_mut_ptr().add(offset), sum);
        }
        
        // Handle remaining elements
        for i in (chunks * 8)..len {
            result[i] = a[i] + b[i];
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    unsafe fn vector_add_neon(a: &[f32], b: &[f32], result: &mut [f32]) {
        let len = a.len();
        let chunks = len / 4;
        
        for i in 0..chunks {
            let offset = i * 4;
            let va = vld1q_f32(a.as_ptr().add(offset));
            let vb = vld1q_f32(b.as_ptr().add(offset));
            let sum = vaddq_f32(va, vb);
            vst1q_f32(result.as_mut_ptr().add(offset), sum);
        }
        
        // Handle remaining elements
        for i in (chunks * 4)..len {
            result[i] = a[i] + b[i];
        }
    }
}

/// Quantized neural network operations for memory efficiency
#[allow(dead_code)]
pub struct QuantizedOps;

#[allow(dead_code)]
impl QuantizedOps {
    /// Convert f32 weights to 8-bit quantized representation
    pub fn quantize_weights(weights: &[f32]) -> QuantizedWeights {
        let min_val = weights.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_val = weights.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        
        let scale = (max_val - min_val) / 255.0;
        let zero_point = (-min_val / scale).round() as u8;
        
        let quantized: Vec<u8> = weights
            .iter()
            .map(|&w| ((w / scale).round() as i32 + zero_point as i32).clamp(0, 255) as u8)
            .collect();
        
        QuantizedWeights {
            data: quantized,
            scale,
            zero_point,
        }
    }

    /// Quantized matrix-vector multiplication
    pub fn quantized_matvec(
        weights: &QuantizedWeights,
        input: &[f32],
        output: &mut [f32],
        rows: usize,
        cols: usize,
    ) {
        assert_eq!(weights.data.len(), rows * cols);
        assert_eq!(input.len(), cols);
        assert_eq!(output.len(), rows);
        
        for row in 0..rows {
            let mut sum = 0i32;
            
            for col in 0..cols {
                let weight_idx = row * cols + col;
                let weight_q = weights.data[weight_idx] as i32;
                let input_q = (input[col] / weights.scale).round() as i32 + weights.zero_point as i32;
                
                sum += (weight_q - weights.zero_point as i32) * (input_q - weights.zero_point as i32);
            }
            
            output[row] = sum as f32 * weights.scale * weights.scale;
        }
    }

    /// 16-bit quantization for higher precision
    pub fn quantize_weights_16bit(weights: &[f32]) -> QuantizedWeights16 {
        let min_val = weights.iter().fold(f32::INFINITY, |a, &b| a.min(b));
        let max_val = weights.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
        
        let scale = (max_val - min_val) / 65535.0;
        let zero_point = (-min_val / scale).round() as u16;
        
        let quantized: Vec<u16> = weights
            .iter()
            .map(|&w| ((w / scale).round() as i32 + zero_point as i32).clamp(0, 65535) as u16)
            .collect();
        
        QuantizedWeights16 {
            data: quantized,
            scale,
            zero_point,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[allow(dead_code)]
pub struct QuantizedWeights {
    pub data: Vec<u8>,
    pub scale: f32,
    pub zero_point: u8,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct QuantizedWeights16 {
    pub data: Vec<u16>,
    pub scale: f32,
    pub zero_point: u16,
}

/// Cache-friendly memory operations
#[allow(dead_code)]
pub struct CacheOptimizedOps;

#[allow(dead_code)]
impl CacheOptimizedOps {
    /// Cache-friendly matrix multiplication with blocking
    pub fn blocked_matrix_multiply(
        a: &[f32], b: &[f32], c: &mut [f32],
        m: usize, n: usize, k: usize,
        block_size: usize,
    ) {
        for i_block in (0..m).step_by(block_size) {
            for j_block in (0..n).step_by(block_size) {
                for k_block in (0..k).step_by(block_size) {
                    let i_end = (i_block + block_size).min(m);
                    let j_end = (j_block + block_size).min(n);
                    let k_end = (k_block + block_size).min(k);
                    
                    for i in i_block..i_end {
                        for j in j_block..j_end {
                            let mut sum = 0.0;
                            for k_idx in k_block..k_end {
                                sum += a[i * k + k_idx] * b[k_idx * n + j];
                            }
                            c[i * n + j] += sum;
                        }
                    }
                }
            }
        }
    }

    /// Prefetch data for better cache utilization
    #[cfg(target_arch = "x86_64")]
    pub fn prefetch_data(ptr: *const u8, locality: i32) {
        unsafe {
            match locality {
                0 => std::arch::x86_64::_mm_prefetch(ptr as *const i8, std::arch::x86_64::_MM_HINT_NTA),
                1 => std::arch::x86_64::_mm_prefetch(ptr as *const i8, std::arch::x86_64::_MM_HINT_T2),
                2 => std::arch::x86_64::_mm_prefetch(ptr as *const i8, std::arch::x86_64::_MM_HINT_T1),
                3 => std::arch::x86_64::_mm_prefetch(ptr as *const i8, std::arch::x86_64::_MM_HINT_T0),
                _ => {}
            }
        }
    }
}

/// Performance benchmarking utilities
#[allow(dead_code)]
pub struct CpuBenchmark;

#[allow(dead_code)]
impl CpuBenchmark {
    pub fn benchmark_dot_product(size: usize, iterations: usize) -> f64 {
        let a: Vec<f32> = (0..size).map(|i| i as f32).collect();
        let b: Vec<f32> = (0..size).map(|i| (i * 2) as f32).collect();
        
        let start = std::time::Instant::now();
        
        for _ in 0..iterations {
            let _result = VectorizedOps::dot_product(&a, &b);
        }
        
        let elapsed = start.elapsed();
        elapsed.as_secs_f64() / iterations as f64
    }

    pub fn benchmark_quantization(size: usize, iterations: usize) -> f64 {
        let weights: Vec<f32> = (0..size).map(|i| (i as f32 - size as f32 / 2.0) / 100.0).collect();
        
        let start = std::time::Instant::now();
        
        for _ in 0..iterations {
            let _quantized = QuantizedOps::quantize_weights(&weights);
        }
        
        let elapsed = start.elapsed();
        elapsed.as_secs_f64() / iterations as f64
    }

    pub fn run_comprehensive_benchmark() -> BenchmarkResults {
        println!("🚀 Running CPU Optimization Benchmarks...");
        
        let dot_product_time = Self::benchmark_dot_product(1000, 10000);
        let quantization_time = Self::benchmark_quantization(1000, 1000);
        
        let optimizer = CpuOptimizer::new();
        
        BenchmarkResults {
            dot_product_time_us: dot_product_time * 1_000_000.0,
            quantization_time_us: quantization_time * 1_000_000.0,
            simd_support: optimizer.simd_support,
            cache_line_size: optimizer.cache_line_size,
            vector_width: optimizer.preferred_vector_width,
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct BenchmarkResults {
    pub dot_product_time_us: f64,
    pub quantization_time_us: f64,
    pub simd_support: SimdSupport,
    pub cache_line_size: usize,
    pub vector_width: usize,
}

#[allow(dead_code)]
impl BenchmarkResults {
    pub fn print_report(&self) {
        println!("\n📊 CPU Optimization Benchmark Results");
        println!("=====================================");
        println!("🔹 Dot Product Time: {:.2} μs", self.dot_product_time_us);
        println!("🔹 Quantization Time: {:.2} μs", self.quantization_time_us);
        println!("🔹 Cache Line Size: {} bytes", self.cache_line_size);
        println!("🔹 Vector Width: {} bytes", self.vector_width);
        println!("\n🧠 SIMD Support:");
        println!("  - AVX2: {}", if self.simd_support.avx2 { "✅" } else { "❌" });
        println!("  - AVX512: {}", if self.simd_support.avx512 { "✅" } else { "❌" });
        println!("  - SSE4.1: {}", if self.simd_support.sse4_1 { "✅" } else { "❌" });
        println!("  - NEON: {}", if self.simd_support.neon { "✅" } else { "❌" });
        
        let estimated_speedup = if self.simd_support.avx2 {
            "8x (AVX2)"
        } else if self.simd_support.sse4_1 {
            "4x (SSE)"
        } else if self.simd_support.neon {
            "4x (NEON)"
        } else {
            "1x (Scalar)"
        };
        
        println!("\n⚡ Estimated Speedup: {}", estimated_speedup);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dot_product_accuracy() {
        let a = vec![1.0, 2.0, 3.0, 4.0];
        let b = vec![2.0, 3.0, 4.0, 5.0];
        
        let result = VectorizedOps::dot_product(&a, &b);
        let expected = 1.0*2.0 + 2.0*3.0 + 3.0*4.0 + 4.0*5.0; // 40.0
        
        assert!((result - expected).abs() < 1e-6);
    }

    #[test]
    fn test_quantization_accuracy() {
        let weights = vec![-1.0, -0.5, 0.0, 0.5, 1.0];
        let quantized = QuantizedOps::quantize_weights(&weights);
        
        // Verify quantization preserves relative ordering
        assert!(quantized.data[0] < quantized.data[1]);
        assert!(quantized.data[1] < quantized.data[2]);
        assert!(quantized.data[2] < quantized.data[3]);
        assert!(quantized.data[3] < quantized.data[4]);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let c = vec![1.0, 0.0, 0.0];
        
        let sim_orthogonal = VectorizedOps::cosine_similarity(&a, &b);
        let sim_identical = VectorizedOps::cosine_similarity(&a, &c);
        
        assert!((sim_orthogonal - 0.0).abs() < 1e-6);
        assert!((sim_identical - 1.0).abs() < 1e-6);
    }
}
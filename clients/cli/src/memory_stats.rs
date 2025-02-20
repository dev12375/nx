use std::process;
use sysinfo::{System, SystemExt};

// We encode the memory usage to i32 type at client
pub fn bytes_to_mb_i32(bytes: u64) -> i32 {
    // Convert to MB with 3 decimal places of precision
    // Multiply by 1000 to preserve 3 decimal places
    ((bytes as f64 * 1000.0) / 1_048_576.0).round() as i32
}

// At server, we decode the memory usage from i32 to f32 to get correct memory usage
#[allow(dead_code)]
pub fn mb_i32_to_f32(mb: i32) -> f32 {
    // Convert back to f32, dividing by 1000 to get the correct value
    (mb as f32) / 1000.0
}

pub fn get_memory_info() -> (i32, i32) {
    let mut sys = System::new_all();
    sys.refresh_all();

    // 转换为 MB 并确保在 i32 范围内
    let used_mem = (sys.used_memory() / 1024) as i32;  // KiB to MiB
    let total_mem = (sys.total_memory() / 1024) as i32; // KiB to MiB

    println!("Debug - Total Memory: {}MiB, Used Memory: {}MiB", total_mem, used_mem);

    (used_mem, total_mem)
}

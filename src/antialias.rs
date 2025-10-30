pub fn antialias(buffer: &mut [u8], width: u32, height: u32) {
       let temp_buffer = buffer.to_vec();

    // Gaussian 3x3 kernel weights (normalized)
    let kernel = [
        [1.0, 2.0, 1.0],
        [2.0, 4.0, 2.0],
        [1.0, 2.0, 1.0],
    ];
    let kernel_sum = 16.0;

    for y in 0..height {
        for x in 0..width {
            let center_idx = ((y * width + x) * 4) as usize;

            let center_r = temp_buffer[center_idx] as i32;
            let center_g = temp_buffer[center_idx + 1] as i32;
            let center_b = temp_buffer[center_idx + 2] as i32;
            let center_a = temp_buffer[center_idx + 3] as i32;

            let mut max_diff = 0;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 { continue; }
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;

                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let idx = ((ny as u32 * width + nx as u32) * 4) as usize;
                        let r = temp_buffer[idx] as i32;
                        let g = temp_buffer[idx + 1] as i32;
                        let b = temp_buffer[idx + 2] as i32;
                        let a = temp_buffer[idx + 3] as i32;

                        let diff = ((r - center_r).abs() + (g - center_g).abs() + (b - center_b).abs() + (a - center_a).abs()) / 4;
                        max_diff = max_diff.max(diff);
                    }
                }
            }

            // If max difference is above threshold, this is an edge that needs smoothing
            let is_edge = max_diff > 15;

            if is_edge {
                // Apply weighted Gaussian blur
                let mut r_sum = 0.0;
                let mut g_sum = 0.0;
                let mut b_sum = 0.0;
                let mut a_sum = 0.0;

                for dy in -1..=1 {
                    for dx in -1..=1 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;

                        if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                            let idx = ((ny as u32 * width + nx as u32) * 4) as usize;
                            let weight = kernel[(dy + 1) as usize][(dx + 1) as usize];

                            r_sum += temp_buffer[idx] as f32 * weight;
                            g_sum += temp_buffer[idx + 1] as f32 * weight;
                            b_sum += temp_buffer[idx + 2] as f32 * weight;
                            a_sum += temp_buffer[idx + 3] as f32 * weight;
                        }
                    }
                }

                buffer[center_idx] = (r_sum / kernel_sum) as u8;
                buffer[center_idx + 1] = (g_sum / kernel_sum) as u8;
                buffer[center_idx + 2] = (b_sum / kernel_sum) as u8;
                buffer[center_idx + 3] = (a_sum / kernel_sum) as u8;
            }
            // else: leave pixel unchanged (no blurring in flat areas)
        }
    }
}

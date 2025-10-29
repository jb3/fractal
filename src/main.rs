use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Rng { state: seed }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }

    fn next_f32(&mut self) -> f32 {
        (self.next() >> 32) as f32 / u32::MAX as f32
    }

    fn next_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("usage: {} <seed> <output_path>", args[0]);
        eprintln!("example: {} 12345 fractal.png", args[0]);
        std::process::exit(1);
    }

    let seed: u64 = args[1].parse().unwrap_or_else(|_| {
        eprintln!("error: seed must be a number");
        std::process::exit(1);
    });

    let output_path = &args[2];

    generate_fractal(seed, output_path);
}

fn generate_fractal(seed: u64, output_path: &str) {
    let width = 800u32;
    let height = 800u32;

    let mut rng = Rng::new(seed);

    let mut buffer = vec![255u8; (width * height * 4) as usize];

    let palette = generate_palette(&mut rng);

    let bg_color = &palette[0];
    for i in 0..(width * height) as usize {
        buffer[i * 4] = bg_color.r;
        buffer[i * 4 + 1] = bg_color.g;
        buffer[i * 4 + 2] = bg_color.b;
        buffer[i * 4 + 3] = bg_color.a;
    }

    let pattern_type = (rng.next() % 3) as usize;

    match pattern_type {
        0 => draw_concentric_circles(&mut buffer, width, height, &mut rng, &palette),
        1 => draw_nested_squares(&mut buffer, width, height, &mut rng, &palette),
        _ => draw_radial_pattern(&mut buffer, width, height, &mut rng, &palette),
    }

    save_png(&buffer, width, height, output_path);
    println!(
        "Fractal generated with seed {} and saved to {}",
        seed, output_path
    );
}

fn generate_palette(rng: &mut Rng) -> Vec<Color> {
    let hue_ranges = [
        (180.0, 220.0), // Blue-cyan
        (280.0, 320.0), // Purple-magenta
        (150.0, 180.0), // Cyan-green
        (20.0, 50.0),   // Orange-yellow
        (330.0, 10.0),  // Pink-red
    ];

    let range_idx = (rng.next() % hue_ranges.len() as u64) as usize;
    let (hue_min, hue_max) = hue_ranges[range_idx];

    let base_hue = if hue_max < hue_min {
        // Wrap around case (red range)
        let h = rng.next_range(hue_min, hue_max + 360.0);
        if h >= 360.0 {
            h - 360.0
        } else {
            h
        }
    } else {
        rng.next_range(hue_min, hue_max)
    };

    let mut palette = Vec::new();

    // Light background
    palette.push(hsv_to_rgb(base_hue, 0.15, 0.95));

    // Main colors - softer
    for i in 0..4 {
        let h = (base_hue + i as f32 * 15.0) % 360.0;
        let s = 0.5 + (i as f32 * 0.1);
        let v = 0.7 + (i as f32 * 0.05);
        palette.push(hsv_to_rgb(h, s, v));
    }

    palette
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match h as u32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Color {
        r: ((r + m) * 255.0) as u8,
        g: ((g + m) * 255.0) as u8,
        b: ((b + m) * 255.0) as u8,
        a: 255,
    }
}

fn draw_concentric_circles(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    rng: &mut Rng,
    palette: &[Color],
) {
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let max_radius = width.min(height) as f32 * 0.45;

    let num_circles = 5 + (rng.next() % 4) as usize;

    for i in (0..num_circles).rev() {
        let radius = max_radius * ((i + 1) as f32 / num_circles as f32);
        let color_idx = 1 + (i % (palette.len() - 1));
        draw_circle(buffer, width, height, cx, cy, radius, &palette[color_idx]);
    }
}

fn draw_nested_squares(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    rng: &mut Rng,
    palette: &[Color],
) {
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let max_size = width.min(height) as f32 * 0.7;

    let num_squares = 4 + (rng.next() % 3) as usize;
    let base_rotation = rng.next_range(0.0, 45.0);

    for i in (0..num_squares).rev() {
        let size = max_size * ((i + 1) as f32 / num_squares as f32);
        let rotation = base_rotation + (i as f32 * 8.0);
        let color_idx = 1 + (i % (palette.len() - 1));
        draw_square(
            buffer,
            width,
            height,
            cx,
            cy,
            size,
            rotation,
            &palette[color_idx],
        );
    }
}

fn draw_radial_pattern(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    rng: &mut Rng,
    palette: &[Color],
) {
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let radius = width.min(height) as f32 * 0.35;

    let num_petals = 6 + (rng.next() % 6) as usize;

    for i in 0..num_petals {
        let angle = (i as f32 * 360.0 / num_petals as f32).to_radians();
        let x = cx + angle.cos() * radius;
        let y = cy + angle.sin() * radius;
        let petal_size = radius * 0.4;
        let color_idx = 1 + (i % (palette.len() - 1));

        draw_circle(buffer, width, height, x, y, petal_size, &palette[color_idx]);
    }

    draw_circle(buffer, width, height, cx, cy, radius * 0.3, &palette[1]);
}

fn draw_circle(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    cx: f32,
    cy: f32,
    radius: f32,
    color: &Color,
) {
    let r_sq = radius * radius;
    let min_x = ((cx - radius).max(0.0) as u32).min(width);
    let max_x = ((cx + radius).ceil() as u32).min(width);
    let min_y = ((cy - radius).max(0.0) as u32).min(height);
    let max_y = ((cy + radius).ceil() as u32).min(height);

    for y in min_y..max_y {
        for x in min_x..max_x {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            if dx * dx + dy * dy <= r_sq {
                let idx = ((y * width + x) * 4) as usize;
                buffer[idx] = color.r;
                buffer[idx + 1] = color.g;
                buffer[idx + 2] = color.b;
                buffer[idx + 3] = color.a;
            }
        }
    }
}

fn draw_square(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    cx: f32,
    cy: f32,
    size: f32,
    rotation: f32,
    color: &Color,
) {
    let mut points = Vec::new();
    for i in 0..4 {
        let angle = rotation + (i as f32 * 90.0);
        let rad = angle.to_radians();
        let r = size / 2.0 * 1.414;
        points.push((cx + rad.cos() * r, cy + rad.sin() * r));
    }
    draw_filled_polygon(buffer, width, height, &points, color);
}

fn draw_filled_polygon(
    buffer: &mut [u8],
    width: u32,
    height: u32,
    points: &[(f32, f32)],
    color: &Color,
) {
    if points.len() < 3 {
        return;
    }

    let min_y = points.iter().map(|(_, y)| *y as i32).min().unwrap().max(0) as u32;
    let max_y = points
        .iter()
        .map(|(_, y)| *y as i32)
        .max()
        .unwrap()
        .min(height as i32 - 1) as u32;

    for y in min_y..=max_y {
        let mut intersections = Vec::new();

        for i in 0..points.len() {
            let (x1, y1) = points[i];
            let (x2, y2) = points[(i + 1) % points.len()];

            if (y1 <= y as f32 && (y as f32) < y2) || (y2 <= y as f32 && (y as f32) < y1) {
                let x = x1 + (y as f32 - y1) * (x2 - x1) / (y2 - y1);
                intersections.push(x);
            }
        }

        intersections.sort_by(|a, b| a.partial_cmp(b).unwrap());

        for i in (0..intersections.len()).step_by(2) {
            if i + 1 < intersections.len() {
                let x_start = intersections[i].max(0.0) as u32;
                let x_end = intersections[i + 1].min(width as f32 - 1.0) as u32;

                for x in x_start..=x_end {
                    let idx = ((y * width + x) * 4) as usize;
                    buffer[idx] = color.r;
                    buffer[idx + 1] = color.g;
                    buffer[idx + 2] = color.b;
                    buffer[idx + 3] = color.a;
                }
            }
        }
    }
}

fn save_png(buffer: &[u8], width: u32, height: u32, path: &str) {
    let path = Path::new(path);
    let file = File::create(path).unwrap();
    let w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header().unwrap();
    writer.write_image_data(buffer).unwrap();
}

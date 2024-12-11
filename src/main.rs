use num_complex::Complex64;
use std::{collections::{HashMap, HashSet}, time::{SystemTime, UNIX_EPOCH}};
use image::{GrayImage, Luma};
use rayon::prelude::*;


const MAP_MIN: f64 = -2.0;
const MAP_MAX: f64 = 2.0;
const MAP_RESOLUTION: f64 = 720.0;
const CYCLE_DETECTION_PRECISION: f64 = 4500000000000000000.0;
const MAX_ITERATIONS: u32 = 1000;
const PIXELS: u32 = MAP_RESOLUTION as u32;
const STEP: f64 = 0.01;


fn create_grayscale_image(pixels: HashMap<(u16, u16), u64>) {
    let max_value = pixels.values().cloned().max().unwrap_or(1);

    let mut img = GrayImage::new(PIXELS,PIXELS);

    for ((x, y), value) in pixels {
        if x as u32 >= PIXELS || y as u32 >= PIXELS {
            continue;
        }

        let normalized_value = ((value as f64 / max_value as f64).sqrt().sqrt() * 255.0).round() as u8;

        img.put_pixel(x as u32, y as u32, Luma([normalized_value]));
    }

    img.save("output.png").expect("Failed to save image");
}


fn translate_range(input: f64) -> u16 {
    let output = 1.0 + (input - MAP_MIN) * (MAP_RESOLUTION - 1.0) / (MAP_MAX - MAP_MIN);
    output.round() as u16
}


fn populate_frequency_map(
    imaginary_input: Complex64,
    mut frequency_map: HashMap<(u16, u16), u64>
) -> HashMap<(u16, u16), u64> {
    let mut z = Complex64::new(0.0, 0.0);
    let mut visited_values = HashSet::new();

    for _ in 0..MAX_ITERATIONS {

        let location: (u16, u16) = (
            translate_range(z.re),
            translate_range(z.im)
        );
        let entry = frequency_map.entry(location).or_insert(0);
        *entry += 1;

        if !visited_values.insert((
            (z.re * CYCLE_DETECTION_PRECISION).round() as i64,
            (z.im * CYCLE_DETECTION_PRECISION).round() as i64,
        )) {
            break
        }

        if z.norm_sqr() > 4.0 {
            break
        }

        z = z * z + imaginary_input;

    }

    frequency_map
}

fn main() {
    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    let mut frequency_map: HashMap<(u16, u16), u64> = HashMap::new();
    const DENSITY: u64 = ((MAP_MAX-MAP_MIN)/(STEP as f64)) as u64;

    let partial_maps: Vec<HashMap<(u16, u16), u64>> = (0..=DENSITY)
        .into_par_iter()
        .map(|i| {
            let mut local_map: HashMap<(u16, u16), u64> = HashMap::new();
            let real = (i as f64 / DENSITY as f64) * (MAP_MAX - MAP_MIN) + MAP_MIN;
            for j in 0..=DENSITY {
                let imag = (j as f64 / DENSITY as f64) * (MAP_MAX - MAP_MIN) + MAP_MIN;
                local_map = populate_frequency_map(Complex64::new(real, imag), local_map);
            }
            local_map
        })
        .collect();

    for map in partial_maps {
        for (key, value) in map {
            *frequency_map.entry(key).or_insert(0) += value;
        }
    }

    let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    println!("Execution time: {:?}", end-start);

    create_grayscale_image(frequency_map);
}

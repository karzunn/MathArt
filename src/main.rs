use num_complex::Complex64;
use std::{collections::{HashMap, HashSet}, time::{SystemTime, UNIX_EPOCH}};
use image::{ImageBuffer, Luma};
use rayon::prelude::*;
use indicatif::{ProgressBar, ProgressStyle};


const MAP_MIN: f64 = -2.0;
const MAP_MAX: f64 = 2.0;
const MAP_RESOLUTION: f64 = 500 as f64;
const CYCLE_DETECTION_PRECISION: f64 = 4500000000000000000.0;
const MAX_ITERATIONS: u32 = 1000;
const PIXELS: u32 = MAP_RESOLUTION as u32;
const STEP: f64 = 0.01;
const SEGMENTS: u64 = 1;

// Step size of 0.0002, 100k interations, and 10k x 10k image is probably the limit. That took 5 hours.
// Step size has the largest impact on iteration time



fn create_grayscale_image(pixels: HashMap<(u16, u16), u64>) {

    let mut unique_values: Vec<u64> = pixels.values().cloned().collect();
    unique_values.sort_unstable();
    unique_values.dedup();

    let max_value = *unique_values.last().unwrap_or(&1);
    let new_max = if unique_values.len() > 1 {
        unique_values[unique_values.len() - 3]
    } else {
        max_value
    };

    let mut img: ImageBuffer<Luma<u16>, Vec<u16>> = ImageBuffer::new(PIXELS, PIXELS);

    let pb = ProgressBar::new(pixels.values().len() as u64);
    let style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")
        .unwrap_or_else(|_| ProgressStyle::default_bar());
    pb.set_style(style);

    for ((x, y), value) in pixels {
        if x as u32 >= PIXELS || y as u32 >= PIXELS {
            continue;
        }

        let value = if value >= new_max {
            new_max
        } else {
            value
        };

        let normalized_value = ((value as f64 / new_max as f64).sqrt().sqrt() * 65535.0).round() as u16;

        img.put_pixel(x as u32, y as u32, Luma([normalized_value]));
        pb.inc(1);
    }

    img.save("output.png").expect("Failed to save image");

    pb.finish_with_message("Image Saved!");
}


fn translate_range(input: f64) -> u16 {
    let output = 1.0 + (input - MAP_MIN) * (MAP_RESOLUTION - 1.0) / (MAP_MAX - MAP_MIN);
    output.round() as u16
}


fn populate_frequency_map(
    c: Complex64,
    mut frequency_map: HashMap<(u16, u16), u64>
) -> HashMap<(u16, u16), u64> {
    let mut z = Complex64::new(0.0, 0.0);
    let mut visited_values = HashSet::new();
    let mut local_map: HashMap<(u16, u16), u64> = HashMap::new();

    for _ in 0..MAX_ITERATIONS {

        let location: (u16, u16) = (
            translate_range(z.re),
            translate_range(z.im)
        );
        let entry = local_map.entry(location).or_insert(0);
        *entry += 1;

        if !visited_values.insert((
            (z.re * CYCLE_DETECTION_PRECISION).round() as i64,
            (z.im * CYCLE_DETECTION_PRECISION).round() as i64,
        )) {
            break
        }

        if z.norm_sqr() > 4.0 {
            for (key, value) in local_map {
                *frequency_map.entry(key).or_insert(0) += value;
            }
            break
        }

        z = z * z + c;

    }

    frequency_map
}

fn main() {
    let start = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    let mut frequency_map: HashMap<(u16, u16), u64> = HashMap::new();
    const DENSITY: u64 = ((MAP_MAX-MAP_MIN)/(STEP as f64)) as u64;
    const SEGMENT_LENGTH: u64 = DENSITY/SEGMENTS;

    let pb = ProgressBar::new(SEGMENTS);
    let style = ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")
        .unwrap_or_else(|_| ProgressStyle::default_bar());
    pb.set_style(style);

    for segment in 0..=SEGMENTS {

        let partial_maps: Vec<HashMap<(u16, u16), u64>> = ((segment*SEGMENT_LENGTH)..=((segment+1)*SEGMENT_LENGTH))
            .into_par_iter()
            .map(|i| {
                let mut local_map: HashMap<(u16, u16), u64> = HashMap::new();
                let real = (i as f64 / DENSITY as f64) * (MAP_MAX - MAP_MIN) + MAP_MIN;
                let mut imag = MAP_MIN;
                while imag <= MAP_MAX {
                    local_map = populate_frequency_map(Complex64::new(real, imag), local_map);
                    imag += STEP;
                }
                local_map
            })
            .collect();

        for map in partial_maps {
            for (key, value) in map {
                *frequency_map.entry(key).or_insert(0) += value;
            }
        }

        pb.inc(1);
    
    }

    pb.finish_with_message("Processing complete.");

    let end = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    println!("Execution time: {:?}", end-start);

    create_grayscale_image(frequency_map);
}

extern crate image;
use image::{DynamicImage, ImageBuffer, GenericImageView, Rgba};
use rand::seq::SliceRandom;
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use std::convert::TryInto;
pub fn scramble_image(image: DynamicImage, tile_positions: &mut Vec<usize>) -> DynamicImage {
    let (width, height) = image.dimensions();
    let tile_width = width / 3;
    let tile_height = height / 3;
    let mut tiles: Vec<DynamicImage> = Vec::with_capacity(9);

    // Load the font (Ensure DejaVuSans.ttf is included in the project)
    let font = Vec::from(include_bytes!("../src/DejaVuSans.ttf") as &[u8]);
    let font = Font::try_from_vec(font).unwrap();

    for y in 0..3 {
        for x in 0..3 {
            let tile_x = x * tile_width;
            let tile_y = y * tile_height;
            let tile = image.crop_imm(tile_x, tile_y, tile_width, tile_height);
            tiles.push(tile);
        }
    }

    let mut rng = rand::thread_rng();
    tiles.shuffle(&mut rng);
    *tile_positions = (0..9).collect::<Vec<usize>>();
    tile_positions.shuffle(&mut rng);

    let mut buffer = ImageBuffer::new(width, height);

    // Define border color and thickness
    let border_color = Rgba([0, 0, 0, 255]); // Black border
    let border_thickness = 5;

    for (i, tile) in tiles.iter().enumerate() {
        let x = (i % 3) as u32 * tile_width;
        let y = (i / 3) as u32 * tile_height;

        for tile_y in 0..tile_height {
            for tile_x in 0..tile_width {
                let pixel = tile.get_pixel(tile_x, tile_y);
                buffer.put_pixel(x + tile_x, y + tile_y, pixel);
            }
        }

        // Draw border around each tile manually with boundary checks
        for thickness in 0..border_thickness {
            let border_start_x = (x as i32 - thickness as i32).max(0);
            let border_end_x = ((x + tile_width) as i32 + thickness as i32).min(width as i32 - 1);
            let border_start_y = (y as i32 - thickness as i32).max(0);
            let border_end_y = ((y + tile_height) as i32 + thickness as i32).min(height as i32 - 1);

            // Top border
            for bx in border_start_x..=border_end_x {
                if border_start_y >= 0 && border_start_y < height as i32 {
                    buffer.put_pixel(bx as u32, border_start_y as u32, border_color);
                }
            }

            // Bottom border
            for bx in border_start_x..=border_end_x {
                if border_end_y >= 0 && border_end_y < height as i32 {
                    buffer.put_pixel(bx as u32, border_end_y as u32, border_color);
                }
            }

            // Left border
            for by in border_start_y..=border_end_y {
                if border_start_x >= 0 && border_start_x < width as i32 {
                    buffer.put_pixel(border_start_x as u32, by as u32, border_color);
                }
            }

            // Right border
            for by in border_start_y..=border_end_y {
                if border_end_x >= 0 && border_end_x < width as i32 {
                    buffer.put_pixel(border_end_x as u32, by as u32, border_color);
                }
            }
        }

        // Set the font size
        let text_scale = Scale { x: 50.0, y: 50.0 };  // Larger font size

        // Set the font color to white
        let text_color = Rgba([255, 255, 255, 255]); // White text color

        // Create a rectangle for the text background
        let index_text = format!("{}", i + 1);  // Static numbers from 1 to 9
        let text_width = text_scale.x * index_text.chars().count() as f32; // Estimate text width
        let text_height = text_scale.y;

        let rect_x = x as i32 + 10; // Adjust position as needed
        let rect_y = y as i32 + 10;
        let rect_width = text_width as u32 + 20; // Padding around the text
        let rect_height = text_height as u32 + 20;

        // Draw the background rectangle (for the border)
        for i in 0..rect_width {
            for j in 0..rect_height {
                let px = rect_x + i as i32;
                let py = rect_y + j as i32;
                if px >= 0 && px < width as i32 && py >= 0 && py < height as i32 {
                    buffer.put_pixel(px as u32, py as u32, border_color);
                }
            }
        }

        // Draw the text on top
        let text_x = (rect_x + 10).try_into().unwrap();
        let text_y = (rect_y + 10).try_into().unwrap();
        draw_text_mut(&mut buffer, text_color, text_x, text_y, text_scale, &font, &index_text);
    }

    DynamicImage::ImageRgba8(buffer)
}
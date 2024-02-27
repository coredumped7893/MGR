use image::{DynamicImage, GenericImageView, ImageResult, Pixel};

const IMAGE_PATH: &str = "data/radar_frame_1.png";

pub const GRID_SIZE: usize = 100;

pub use image::Rgba;

pub type PixelColor = Rgba<u8>;
type PixelLumaColor = u8;

fn load_image() -> ImageResult<DynamicImage> {
    // Use the open function to load an image from a Path.
    // `open` returns a `DynamicImage` on success.
    let img = image::open(IMAGE_PATH);

    println!("Image loaded");

    img
}

fn pixel_to_luma(pixel: PixelColor) -> PixelLumaColor {
    let luma = pixel.to_luma();
    luma[0]
}

pub fn produce_grid() -> Vec<PixelColor> {
    match load_image() {
        Ok(img) => {
            //iterate over each pixel and group it into a grid
            //each value of the grid should contain the highest value of the pixels within given cell
            let (width, height) = img.dimensions();
            let image_width = width as usize;
            let image_height = height as usize;
            println!("Image dimensions: {}x{}", image_width, image_height);
            println!("Image pixel count: {}", image_width * image_height);
            let grid_size = GRID_SIZE * GRID_SIZE;
            let mut grid: Vec<PixelColor> = vec![Rgba([0, 0, 0, 0]); grid_size];
            let grid_x_cell_divider = image_width / GRID_SIZE;//Number of pixels per grid cell

            for (x, y, color) in img.pixels() {
                let pixel_luma_value = pixel_to_luma(color);
                //calculate the grid coordinates
                let grid_x = x as usize / grid_x_cell_divider;
                let grid_y = y as usize / grid_x_cell_divider;
                let grid_index = (grid_y * GRID_SIZE) + grid_x;
                // if grid_index >= grid.len() {
                //     println!("Grid index out of bounds: {}", grid_index);
                // }

                let grid_luma_value = pixel_to_luma(grid[grid_index]);
                if grid_index < grid.len() && pixel_luma_value > grid_luma_value {
                    grid[grid_index] = color;
                }
            }

            println!("Grid created");
            println!("Grid size: {}", grid.len());
            grid
        }
        Err(e) => {
            panic!("Error loading image: {}", e);
        }
    }
}

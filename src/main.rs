use std::env;
use std::path::Path;
use std::fs::File;
use std::io::BufReader;
use image::{ImageFormat, DynamicImage, ImageError};

#[derive(Debug, Clone, Copy)]
enum SupportedFormat {
    Jpeg,
    Png,
    WebP,
    Avif,
}

impl SupportedFormat {
    fn from_extension(ext: &str) -> Result<Self, String> {
        match ext.to_lowercase().as_str() {
            "jpg" | "jpeg" => Ok(SupportedFormat::Jpeg),
            "png" => Ok(SupportedFormat::Png),
            "webp" => Ok(SupportedFormat::WebP),
            "avif" => Ok(SupportedFormat::Avif),
            _ => Err(format!("Unsupported format: {}", ext)),
        }
    } 

    fn extension(self) -> &'static str {
        match self {
            SupportedFormat::Jpeg => "jpg",
            SupportedFormat::Png => "png",
            SupportedFormat::WebP => "webp",
            SupportedFormat::Avif => "avif",
        }
    }
}

struct ImageConverter {
    quality: u8,
}

impl ImageConverter {
    fn new(quality: u8) -> Self {
        Self {
            quality: quality.min(100),
        }
    }

    fn load_image(&self, input_path: &Path) -> Result<DynamicImage, ImageError> {
        let file = File::open(input_path)?;
        let reader = BufReader::new(file);
        image::load(reader, ImageFormat::from_path(input_path)?)
    }

    fn save_image(
        &self,
        image: &DynamicImage,
        output_path: &Path,
        format: SupportedFormat,
    ) -> Result<(), ImageError> {
        match format {
            SupportedFormat::Jpeg => {
                let mut output = File::create(output_path)?;
                image.write_to(&mut output, ImageFormat::Jpeg)?;
            }
            SupportedFormat::Png => {
                image.save_with_format(output_path, ImageFormat::Png)?;
            }
            SupportedFormat::WebP => {
                image.save_with_format(output_path, ImageFormat::WebP)?;
            }
            SupportedFormat::Avif => {
                image.save_with_format(output_path, ImageFormat::Avif)?;
            }
        }
        Ok(())
    }

    fn convert(
        &self,
        input_path: &Path,
        output_path: &Path,
        target_format: SupportedFormat,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Loading image: {}", input_path.display());
        let image = self.load_image(input_path)?;
        
        println!("Image dimensions: {}x{}", image.width(), image.height());
        
        println!("Converting to {} format...", target_format.extension());
        self.save_image(&image, output_path, target_format)?;
        
        println!("Conversion completed: {}", output_path.display());
        Ok(())
    }

    fn batch_convert(
        &self,
        input_dir: &Path,
        output_dir: &Path,
        target_format: SupportedFormat,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !output_dir.exists() {
            std::fs::create_dir_all(output_dir)?;
        }

        let entries = std::fs::read_dir(input_dir)?;
        let mut converted_count = 0;

        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if let Ok(_) = SupportedFormat::from_extension(&extension.to_string_lossy()) {
                        let file_stem = path.file_stem().unwrap().to_string_lossy();
                        let output_filename = format!("{}.{}", file_stem, target_format.extension());
                        let output_path = output_dir.join(output_filename);

                        match self.convert(&path, &output_path, target_format) {
                            Ok(_) => {
                                converted_count += 1;
                                println!("✓ Converted: {}", path.file_name().unwrap().to_string_lossy());
                            }
                            Err(e) => {
                                eprintln!("✗ Failed to convert {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }

        println!("\nBatch conversion completed! {} files converted.", converted_count);
        Ok(())
    }
}

fn print_usage() {
    println!("Image Format Converter");
    println!("Supports: JPG/JPEG, PNG, WebP, AVIF");
    println!();
    println!("Usage:");
    println!("  Single file: {} <input_file> <output_file>", env::args().next().unwrap());
    println!("  Batch mode:  {} --batch <input_dir> <output_dir> <format>", env::args().next().unwrap());
    println!();
    println!("Examples:");
    println!("  {} image.png image.webp", env::args().next().unwrap());
    println!("  {} input.jpg output.avif", env::args().next().unwrap());
    println!("  {} --batch ./input ./output webp", env::args().next().unwrap());
    println!();
    println!("Supported formats: jpg, jpeg, png, webp, avif");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 3 {
        print_usage();
        std::process::exit(1);
    }

    let converter = ImageConverter::new(85); // Default quality

    if args[1] == "--batch" {
        // Batch mode
        if args.len() != 5 {
            eprintln!("Error: Batch mode requires 4 arguments");
            print_usage();
            std::process::exit(1);
        }

        let input_dir = Path::new(&args[2]);
        let output_dir = Path::new(&args[3]);
        let format_str = &args[4];

        let target_format = match SupportedFormat::from_extension(format_str) {
            Ok(format) => format,
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        };

        if !input_dir.exists() || !input_dir.is_dir() {
            eprintln!("Error: Input directory does not exist or is not a directory");
            std::process::exit(1);
        }

        if let Err(e) = converter.batch_convert(input_dir, output_dir, target_format) {
            eprintln!("Error during batch conversion: {}", e);
            std::process::exit(1);
        }
    } else {
        // Single file mode
        if args.len() != 3 {
            eprintln!("Error: Single file mode requires 2 arguments");
            print_usage();
            std::process::exit(1);
        }

        let input_path = Path::new(&args[1]);
        let output_path = Path::new(&args[2]);

        if !input_path.exists() {
            eprintln!("Error: Input file does not exist: {}", input_path.display());
            std::process::exit(1);
        }

        let target_format = match output_path.extension() {
            Some(ext) => match SupportedFormat::from_extension(&ext.to_string_lossy()) {
                Ok(format) => format,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            },
            None => {
                eprintln!("Error: Output file must have a valid extension");
                std::process::exit(1);
            }
        };

        if let Err(e) = converter.convert(input_path, output_path, target_format) {
            eprintln!("Error during conversion: {}", e);
            std::process::exit(1);
        }
    }
}

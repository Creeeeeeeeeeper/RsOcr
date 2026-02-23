mod charset;

use image::imageops::FilterType;
use image::GenericImageView;
use ndarray::Array4;
use ort::session::Session;
use base64::Engine;
use std::path::{Path, PathBuf};

fn preprocess(img_bytes: &[u8]) -> Array4<f32> {
    let img = image::load_from_memory(img_bytes).expect("Failed to load image");
    let (w, h) = img.dimensions();
    let target_height: u32 = 64;
    let target_width = (w as f64 * (target_height as f64 / h as f64)) as u32;
    let resized = img.resize_exact(target_width, target_height, FilterType::Lanczos3);
    let gray = resized.to_luma8();

    let mut arr = Array4::<f32>::zeros((1, 1, target_height as usize, target_width as usize));
    for y in 0..target_height as usize {
        for x in 0..target_width as usize {
            arr[[0, 0, y, x]] = gray.get_pixel(x as u32, y as u32).0[0] as f32 / 255.0;
        }
    }
    arr
}

fn ctc_decode(predicted: &[usize]) -> Vec<usize> {
    let mut decoded = Vec::new();
    let mut prev: Option<usize> = None;
    for &idx in predicted {
        if Some(idx) != prev && idx != 0 {
            decoded.push(idx);
        }
        prev = Some(idx);
    }
    decoded
}

fn classify(session: &mut Session, img_bytes: &[u8], charset: &[String]) -> String {
    let img_array = preprocess(img_bytes);
    let input_tensor =
        ort::value::Tensor::from_array(img_array).expect("Failed to create input tensor");

    let outputs = session
        .run(ort::inputs![input_tensor])
        .expect("Failed to run inference");

    let output = outputs[0]
        .try_extract_array::<f32>()
        .expect("Failed to extract tensor");
    let shape = output.shape();

    let predicted: Vec<usize> = if shape.len() == 3 {
        if shape[1] == 1 {
            (0..shape[0])
                .map(|t| {
                    let mut max_idx = 0usize;
                    let mut max_val = f32::NEG_INFINITY;
                    for c in 0..shape[2] {
                        let v = output[[t, 0, c]];
                        if v > max_val {
                            max_val = v;
                            max_idx = c;
                        }
                    }
                    max_idx
                })
                .collect()
        } else {
            (0..shape[1])
                .map(|t| {
                    let mut max_idx = 0usize;
                    let mut max_val = f32::NEG_INFINITY;
                    for c in 0..shape[2] {
                        let v = output[[0, t, c]];
                        if v > max_val {
                            max_val = v;
                            max_idx = c;
                        }
                    }
                    max_idx
                })
                .collect()
        }
    } else {
        let rows = shape[0];
        let cols = shape[1];
        (0..rows)
            .map(|t| {
                let mut max_idx = 0usize;
                let mut max_val = f32::NEG_INFINITY;
                for c in 0..cols {
                    let v = output[[t, c]];
                    if v > max_val {
                        max_val = v;
                        max_idx = c;
                    }
                }
                max_idx
            })
            .collect()
    };

    let indices = ctc_decode(&predicted);
    indices
        .iter()
        .filter(|&&i| i < charset.len())
        .map(|&i| charset[i].as_str())
        .collect()
}

fn is_image(p: &Path) -> bool {
    let name = p.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
    name.ends_with(".jpg")
        || name.ends_with(".jpeg")
        || name.ends_with(".png")
        || name.ends_with(".bmp")
        || name.ends_with(".gif")
}

fn print_help() {
    eprintln!("Usage: captcha-ocr [OPTIONS]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -m <path>    ONNX model path (default: ./common.onnx)");
    eprintln!("  -c <path>    charset.json path (default: built-in charset3.json)");
    eprintln!("  -i <path>    recognize a single image file");
    eprintln!("  -d <path>    recognize all images in a directory");
    eprintln!("  -b <base64>  recognize image from base64 encoded string");
    eprintln!("  -f           show filename in output (filename -> result)");
    eprintln!("  -h           show this help");
    eprintln!();
    eprintln!("If no -i, -d or -b is given, scans current directory for images.");
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cwd = std::env::current_dir().expect("Failed to get current directory");

    let mut model_path: Option<PathBuf> = None;
    let mut charset_path: Option<PathBuf> = None;
    let mut input_file: Option<PathBuf> = None;
    let mut input_dir: Option<PathBuf> = None;
    let mut input_base64: Option<String> = None;
    let mut show_filename = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-m" => {
                i += 1;
                model_path = Some(PathBuf::from(&args[i]));
            }
            "-c" => {
                i += 1;
                charset_path = Some(PathBuf::from(&args[i]));
            }
            "-i" => {
                i += 1;
                input_file = Some(PathBuf::from(&args[i]));
            }
            "-d" => {
                i += 1;
                input_dir = Some(PathBuf::from(&args[i]));
            }
            "-b" => {
                i += 1;
                input_base64 = Some(args[i].clone());
            }
            "-f" | "--filename" => {
                show_filename = true;
            }
            "-h" | "--help" => {
                print_help();
                return;
            }
            other => {
                eprintln!("Unknown option: {}", other);
                print_help();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    // Load charset
    let charset = if let Some(path) = charset_path {
        charset::load_charset_from_file(path.to_str().unwrap())
    } else {
        charset::CHARSET.clone()
    };

    // Resolve model path: -m > ./common.onnx > exe dir
    let model_path = model_path.unwrap_or_else(|| {
        let local = cwd.join("common.onnx");
        if local.exists() {
            return local;
        }
        if let Ok(exe) = std::env::current_exe() {
            let exe_dir = exe.parent().unwrap().to_path_buf();
            let beside_exe = exe_dir.join("common.onnx");
            if beside_exe.exists() {
                return beside_exe;
            }
        }
        local
    });

    if !model_path.exists() {
        eprintln!("Model not found: {}", model_path.display());
        eprintln!("Use -m <path> to specify the ONNX model path.");
        std::process::exit(1);
    }

    let mut session = Session::builder()
        .unwrap()
        .commit_from_file(&model_path)
        .expect("Failed to load ONNX model");

    // Base64 mode: decode and classify directly
    if let Some(b64) = input_base64 {
        // Strip data URI prefix (e.g. "data:image/png;base64,") if present
        let b64_data = if let Some(pos) = b64.find(",") {
            &b64[pos + 1..]
        } else {
            &b64
        };
        let img_bytes = match base64::engine::general_purpose::STANDARD.decode(b64_data) {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Failed to decode base64 string: {}", e);
                std::process::exit(1);
            }
        };
        let text = match image::load_from_memory(&img_bytes) {
            Ok(_) => classify(&mut session, &img_bytes, &charset),
            Err(e) => {
                eprintln!("Failed to load image from decoded bytes: {}", e);
                std::process::exit(1);
            }
        };
        println!("{}", text);
        return;
    }

    // Collect image files
    let files: Vec<PathBuf> = if let Some(f) = input_file {
        if !f.exists() {
            eprintln!("File not found: {}", f.display());
            std::process::exit(1);
        }
        vec![f]
    } else {
        let dir = input_dir.unwrap_or_else(|| cwd.clone());
        if !dir.is_dir() {
            eprintln!("Not a directory: {}", dir.display());
            std::process::exit(1);
        }
        let mut entries: Vec<PathBuf> = std::fs::read_dir(&dir)
            .expect("Failed to read directory")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file() && is_image(p))
            .collect();
        entries.sort();
        entries
    };

    if files.is_empty() {
        eprintln!("No image files found.");
        std::process::exit(0);
    }

    let mut count = 0;
    for path in &files {
        let img_bytes = std::fs::read(path).expect("Failed to read image");
        let text = classify(&mut session, &img_bytes, &charset);
        if show_filename {
            println!("{} -> {}", path.file_name().unwrap().to_string_lossy(), text);
        } else {
            println!("{}", text);
        }
        count += 1;
    }

    if count > 1 {
        println!("\n共识别 {} 张验证码", count);
    }
}

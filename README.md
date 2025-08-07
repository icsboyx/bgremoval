# BGRemoval - Real-time Background Removal with ML

A high-performance Rust application that captures video from cameras, removes backgrounds using ONNX machine learning models, and outputs to both a live viewer and virtual camera device for use in video calls.

## 🚀 Features

- **Real-time Camera Capture**: V4L2-based MJPEG video stream capture
- **AI-Powered Background Removal**: ONNX model inference with GPU acceleration
- **Multi-threaded Pipeline**: Optimized parallel processing architecture
- **GPU Acceleration**: CUDA and TensorRT execution providers
- **Virtual Camera Output**: Creates virtual camera device for video conferencing
- **Live Preview**: Real-time display with multiple view modes
- **Adaptive Processing**: Smart frame skipping and mask reuse for performance

## 🏗️ Architecture

The application uses a high-performance multi-threaded pipeline:

```
📷 Camera → 📦 Capture → 🔧 Decoder → 🤖 BGRemoval → 👁️ Viewer
            (MJPEG)     (RGB)       (ML Mask)     (Display)
                                        ↓
                                   🎥 Virtual Camera
```

### Pipeline Components

1. **Capture Thread**: Captures MJPEG frames using V4L2 MmapStream
2. **Decoder Thread**: Decodes MJPEG to RGB, creates dual-resolution frames
3. **BGRemoval Thread**: Runs ONNX inference to generate background masks
4. **Viewer Thread**: Displays results using Raylib with multiple views
5. **Virtual Camera Thread**: Outputs processed frames to virtual camera device

## 📋 Prerequisites

### System Requirements
- **OS**: Linux (Ubuntu 20.04+ recommended)
- **GPU**: NVIDIA GPU with CUDA support (optional but recommended)
- **Camera**: USB webcam or built-in camera
- **RAM**: 4GB+ recommended

### External Software Requirements
- **Rust**: Latest stable version
- **NVIDIA Drivers**: For GPU acceleration
- **CUDA Toolkit**: Version 11.0+ for GPU support
- **TensorRT**: Optional, for additional optimization
- **v4l2loopback**: For virtual camera functionality

### System Libraries Required
- `build-essential`
- `pkg-config`
- `libv4l-dev`
- `libjpeg-dev`
- `libturbojpeg0-dev`
- `libx11-dev`
- `libxcursor-dev`
- `libxrandr-dev`
- `libxinerama-dev`
- `libxi-dev`
- `libgl1-mesa-dev`
- `libasound2-dev`
- `v4l2loopback-dkms`
- `v4l2loopback-utils`

### ONNX Runtime Requirements
- ONNX Runtime with GPU support (version 1.16.3+)
- CUDA Runtime libraries
- TensorRT libraries (optional)

### Background Removal Model Requirements
- ONNX format background segmentation model
- Input format: RGB image tensor `[1, 3, 512, 512]`
- Output format: Segmentation mask `[1, 1, 512, 512]`
- Model file location: `models/model.onnx`

## ⚙️ Configuration

Edit the `SETUP` constant in `src/main.rs`:

```rust
pub static SETUP: Setup = Setup {
    camera_device: 0,                      // Camera device index (/dev/video0)
    capture_width: 1920,                   // Camera capture resolution
    capture_res_height: 1080,              
    full_dec_width: 1920,                  // High-resolution processing
    full_dec_height: 1080,
    ful_dec_pixel_type: PixelType::U8x4,   // RGBA format
    small_dec_width: 512,                  // ML model input size
    small_dec_height: 512,
    small_dec_pixel_type: PixelType::U8x4,
};
```

### Key Configuration Options

- **`camera_device`**: Index of camera device (check with `v4l2-ctl --list-devices`)
- **`capture_width/height`**: Camera capture resolution
- **`full_dec_*`**: High-resolution processing dimensions
- **`small_dec_*`**: ML model input dimensions (typically 512x512)

## 🚀 Usage

### 1. Start the Application
```bash
# With debug output
RUST_LOG=info cargo run --release

# Or run the binary directly
./target/release/bgremoval
```

### 2. Application Output
The application will display:
- Available camera devices and formats
- Processing pipeline status
- Live viewer window with three panels:
  - **Left**: Original high-resolution feed
  - **Center**: Low-resolution ML processing view
  - **Right**: Background removal result

### 3. Virtual Camera Usage
Use the virtual camera device in video conferencing apps:
- **Zoom**: Settings → Video → Camera → BGRemoval Virtual Camera
- **Teams**: Settings → Devices → Camera → BGRemoval Virtual Camera
- **OBS**: Add Video Capture Device → BGRemoval Virtual Camera

### 4. Controls
- **ESC**: Close viewer window
- **Space**: Toggle processing (if implemented)
- Window close button: Shutdown application

## 📁 Project Structure

```
bgremoval/
├── src/
│   ├── main.rs              # Application entry point and configuration
│   ├── capture.rs           # V4L2 camera capture with MmapStream
│   ├── decoder.rs           # MJPEG decoding and dual-resolution processing
│   ├── bgremoval.rs         # ONNX ML inference and mask generation
│   ├── viewer.rs            # Raylib multi-panel display
│   └── virtual_camera.rs    # Virtual camera device output
├── models/
│   └── model.onnx          # Background segmentation ONNX model
├── libs/                   # ONNX Runtime libraries
├── Cargo.toml              # Rust dependencies
└── README.md
```

## 🔧 Key Components Deep Dive

### Data Structures

#### `MlFrames`
```rust
pub struct MlFrames {
    pub high_res_frame: Frame,  // Full resolution for display
    pub low_res_frame: Frame,   // 512x512 for ML processing
    pub instant: Instant,       // Timestamp for performance tracking
}
```

#### `RaylibFrames` 
```rust
pub struct RaylibFrames {
    pub high_res_frame: Frame,  // Original camera feed
    pub low_res_frame: Frame,   // Downscaled feed
    pub ml_low_frame: Frame,    // ML mask (low res)
    pub ml_high_frame: Frame,   // ML mask (high res)
    pub instant: Instant,
}
```

#### `Frame` Utilities
The `Frame` struct provides essential conversion methods:
- `to_nchw_f32()`: Convert to ML model input format [N,C,H,W]
- `as_rgba()`: Convert to RGBA for display
- `resize()`: Efficient resizing with fast_image_resize

### Performance Optimizations

#### Adaptive Processing
```rust
let mask_per_frame = 0; // Process every frame (0) or skip frames (>0)
```

#### Memory Efficiency
- Reuses background masks across frames
- Smart buffer management with MmapStream
- Efficient image resizing with SIMD acceleration

#### GPU Acceleration
```rust
// CUDA execution provider
let ep = CUDAExecutionProvider::default().with_device_id(0).build();
// Fallback to TensorRT if available
// let ep = TensorRTExecutionProvider::default().with_device_id(0).build();
```

## 🎯 Performance Tuning

### Frame Rate Optimization
- **Target**: 30-60 FPS depending on hardware
- **Bottlenecks**: ML inference, MJPEG decoding
- **Solutions**: Frame skipping, GPU acceleration, smaller model input

### Memory Usage
- **Typical**: 500MB-1GB RAM usage
- **GPU**: 1-2GB VRAM for ML model
- **Optimization**: Buffer reuse, efficient pixel formats

### Latency Reduction
- **Pipeline**: Each thread adds ~1-2ms latency
- **Total**: Typically 50-100ms end-to-end
- **Optimization**: Smaller buffers, GPU processing

## 🐛 Troubleshooting

### Camera Issues
```bash
# List available cameras
v4l2-ctl --list-devices

# Check camera capabilities
v4l2-ctl --device=/dev/video0 --list-formats-ext

# Test camera
ffplay /dev/video0
```

### Virtual Camera Issues
```bash
# Reload v4l2loopback
sudo modprobe -r v4l2loopback
sudo modprobe v4l2loopback devices=1 video_nr=10 card_label="BGRemoval"

# Check virtual camera
v4l2-ctl --device=/dev/video10 --all
```

### CUDA/GPU Issues
```bash
# Check NVIDIA driver
nvidia-smi

# Check CUDA installation
nvcc --version

# Test ONNX Runtime GPU
python3 -c "import onnxruntime; print(onnxruntime.get_available_providers())"
```

### Build Issues
```bash
# Install missing Rust target
rustup target add x86_64-unknown-linux-gnu

# Clean and rebuild
cargo clean
cargo build --release

# Check library path
ldd target/release/bgremoval
```

## 📦 Dependencies

### Core Rust Crates
```toml
[dependencies]
# Video capture and processing
v4l = "0.14"
turbojpeg = "0.5"
fast_image_resize = "3.0"

# Machine learning
ort = { version = "2.0", features = ["cuda", "tensorrt"] }
ndarray = "0.15"

# Graphics and display  
raylib = "4.5"

# Async and threading
tokio = { version = "1.0", features = ["full"] }
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"
```

### System Libraries
- **libv4l2**: Video4Linux camera interface
- **libturbojpeg**: Hardware-accelerated JPEG decoding
- **CUDA Runtime**: GPU computation
- **ONNX Runtime**: ML model inference

## 🤝 Contributing

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes (`git commit -m 'Add amazing feature'`)
4. **Push** to branch (`git push origin feature/amazing-feature`)
5. **Open** a Pull Request



## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [ONNX Runtime](https://onnxruntime.ai/) for ML inference
- [Raylib](https://www.raylib.com/) for graphics rendering
- [Video4Linux](https://www.kernel.org/doc/html/latest/media/uapi/v4l/v4l2.html) for camera access
- [v4l2loopback](https://github.com/umlaeute/v4l2loopback) for virtual camera support

---

**Made with ❤️ in Rust** 🦀
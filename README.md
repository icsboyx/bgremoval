# bgremoval - Real-time Background Removal

A high-performance Rust application that captures video from a camera, removes the background using machine learning, and displays the result in real-time using Raylib.

## Features

- **Real-time Camera Capture**: Uses V4L2 to capture MJPEG video streams from cameras
- **ML-powered Background Removal**: ONNX model inference with CUDA/TensorRT acceleration
- **Multi-threaded Pipeline**: Separate threads for capture, decoding, ML processing, and rendering
- **GPU Acceleration**: CUDA and TensorRT execution providers for fast inference
- **Live Preview**: Real-time display of original, low-res, and processed frames

## Architecture

The application uses a multi-threaded pipeline architecture:

```
Camera → Capture → Decoder → bgremoval → Viewer
         (MJPEG)   (RGB)     (ML Mask)   (Display)
```

1. **Capture Thread**: Captures MJPEG frames from camera using V4L2
2. **Decoder Thread**: Decodes MJPEG to RGB and resizes for ML processing
3. **bgremoval Thread**: Runs ONNX model inference to generate background masks
4. **Viewer Thread**: Displays results using Raylib

## Prerequisites

### System Requirements
- Linux (for V4L2 camera support)
- NVIDIA GPU with CUDA support (recommended)
- Camera device (USB/built-in webcam)

### Dependencies
- **Rust** (latest stable)
- **CUDA Toolkit** (for GPU acceleration)
- **TensorRT** (optional, for additional optimization)
- **Development libraries**:
  ```bash
  sudo apt update
  sudo apt install build-essential pkg-config
  sudo apt install libv4l-dev libjpeg-dev
  sudo apt install libx11-dev libxcursor-dev libxrandr-dev libxinerama-dev libxi-dev libgl1-mesa-dev
  ```

## Installation

1. **Clone the repository**:
   ```bash
   git clone https://github.com/icsboyx/bgremoval.git
   cd bgremoval
   ```

2. **Install ONNX Runtime**:
   Download the ONNX Runtime libraries and place them in the project directory:
   ```bash
   # The project expects these files in target/debug/ and target/release/:
   # - libonnxruntime_providers_cuda.so
   # - libonnxruntime_providers_shared.so
   # - libonnxruntime_providers_tensorrt.so
   ```

3. **Add your ONNX model**:
   Place your background removal ONNX model at:
   ```
   models/model.onnx
   ```

4. **Build the project**:
   ```bash
   cargo build --release
   ```

## Configuration

Edit the [`SETUP`](src/main.rs) configuration in [`src/main.rs`](src/main.rs):

```rust
pub static SETUP: Setup = Setup {
    camera_device: 0,                      // Camera device index
    capture_width: 1920,                   // Camera capture width
    capture_res_height: 1080,              // Camera capture height
    full_dec_width: 1920,                  // High-res processing width
    full_dec_height: 1080,                 // High-res processing height
    small_dec_width: 512,                  // ML model input width
    small_dec_height: 512,                 // ML model input height
    // ... pixel type configurations
};
```

## Usage

1. **Run the application**:
   ```bash
   cargo run --release
   ```

2. **The application will**:
   - List all available camera devices
   - Display supported video formats
   - Start the real-time processing pipeline
   - Open a window showing three views:
     - Original high-resolution feed
     - Low-resolution feed
     - ML-processed background removal

3. **Controls**:
   - Close the window to stop the application
   - The application runs at 60 FPS target

## Project Structure

```
bgremoval/
├── src/
│   ├── main.rs          # Main application and configuration
│   ├── capture.rs       # Camera capture using V4L2
│   ├── decoder.rs       # MJPEG decoding and image processing
│   ├── bgremoval.rs     # ML inference and background removal
│   └── viewer.rs        # Raylib rendering and display
├── models/
│   └── model.onnx       # ONNX background removal model
├── Cargo.toml           # Rust dependencies
└── README.md
```

## Key Components

### [`Setup`](src/main.rs) Configuration
The [`Setup`](src/main.rs) struct in [`src/main.rs`](src/main.rs) contains all configuration parameters for camera resolution, processing dimensions, and pixel formats.

### [`MlFrames`](src/bgremoval.rs) & [`RaylibFrames`](src/viewer.rs)
Data structures for passing processed frames between pipeline stages:
- [`MlFrames`](src/bgremoval.rs): High and low resolution frames for ML processing
- [`RaylibFrames`](src/viewer.rs): All frame variants for display

### [`Frame`](src/viewer.rs) Utilities
The [`Frame`](src/viewer.rs) struct in [`src/viewer.rs`](src/viewer.rs) provides conversion methods:
- [`to_nchw_f32()`](src/viewer.rs): Convert to ML model input format
- [`as_rgba()`](src/viewer.rs): Convert to display format

## Performance Optimization

- **GPU Acceleration**: Uses CUDA execution provider for ML inference
- **Multi-threading**: Parallel processing pipeline
- **Memory Efficiency**: Reuses masks across frames when possible
- **Resize Optimization**: Uses fast_image_resize for efficient scaling

## Troubleshooting

### Camera Issues
```bash
# List available cameras
v4l2-ctl --list-devices

# Check camera formats
v4l2-ctl --device=/dev/video0 --list-formats-ext
```

### CUDA Issues
- Ensure NVIDIA drivers are installed
- Check CUDA toolkit installation
- Verify ONNX Runtime CUDA provider is available

### Build Issues
- Make sure all system dependencies are installed
- Check that ONNX Runtime libraries are in the correct location

## Dependencies

Key Rust crates used:
- `v4l` - Video4Linux camera capture
- `turbojpeg` - MJPEG decoding
- `ort` - ONNX Runtime integration
- `raylib` - Graphics rendering
- `fast_image_resize` - Image scaling
- `ndarray` - Multi-dimensional arrays
- `anyhow` - Error handling

## License
This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing
Contributions are welcome! Please submit a pull request or open an issue for any bugs or feature requests.
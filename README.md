# pillow-jpegxl-plugin
Pillow plugin for JPEG-XL, using Rust for bindings.

## Features
- JPEG-XL Plugin for Pillow
- Encoder/Decoder to work with JPEG-XL using safe wrapper

## Install via PIP
```
pip install pillow-jxl-plugin
```

## Build from source
Make sure `Rust` and [maturin](https://github.com/PyO3/maturin) installed, then run:
```
git clone https://github.com/Isotr0py/pillow-jpegxl-plugin
cd pillow-jpegxl-plugin

maturin build --release --features vendored
```
If you have `libjxl` installed and want to use dynamic link, run:
```
maturin build --release --features dynamic
```

## Plugin Usage
Use `import pillow_jxl` to register the plugin in your code. 

### Example:
```python
import pillow_jxl
from PIL import Image

# Lossless encode a png image
with Image.open("example.png") as img:
    img.save("example.jxl",lossless=True)

# Lossless construct from a jpeg image
with Image.open("example.jpg") as img:
    img.save("example.jxl",lossless=True)

# Decode jxl image
with Image.open("example.jxl") as img:
    display(img)
```

## Wheels status
| Wheels      | Windows 64-bit | MacOS | manylinux |
|-------------|:--------------:|:-----:|:---------:|
| CPython3.8  |        ✔       |   ✔   |     ✔     |
| CPython3.9  |        ✔       |   ✔   |     ✔     |
| CPython3.10 |        ✔       |   ✔   |     ✔     |
| CPython3.11 |        ✔       |   ✔   |     ✔     |
| CPython3.12 |        ✔       |   ✔   |     ✔     |
| CPython3.13 |        ❌       |   ❌   |     ✔     |
| PyPy3.8     |        ❌       |   ❌   |     ✔     |
| PyPy3.9     |        ❌       |   ❌   |     ✔     |
| PyPy3.10    |        ❌       |   ❌   |     ✔     |

## Credits
- [inflation/jpegxl-rs](https://github.com/inflation/jpegxl-rs)

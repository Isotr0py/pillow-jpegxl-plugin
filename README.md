# pillow-jpegxl-plugin
Pillow plugin for JPEG-XL, using Rust for bindings.

## Features
- JPEG-XL Plugin for Pillow
- Encoder/Decoder to work with JPEG-XL using safe wrapper

## Install via PIP
```
pip install pillow-jxl-plugin
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

# Decode jxl image
with Image.open("example.jxl") as img:
    display(img)
```

## Wheels status
| Wheels      | Windows 64-bit | manylinux |
|-------------|:--------------:|:---------:|
| CPython3.8  |        ✔       |   x86_64  |
| CPython3.9  |        ✔       |   x86_64  |
| CPython3.10 |        ✔       |   x86_64  |
| CPython3.11 |        ✔       |   x86_64  |
| CPython3.12 |        ❌       |   x86_64  |
| PyPy3.8     |        ❌       |   x86_64  |
| PyPy3.9     |        ❌       |   x86_64  |
| PyPy3.10    |        ❌       |   x86_64  |

## Credits
- [inflation/jpegxl-rs](https://github.com/inflation/jpegxl-rs)

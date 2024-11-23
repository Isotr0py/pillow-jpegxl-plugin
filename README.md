# pillow-jpegxl-plugin
![PyPI - Version](https://img.shields.io/pypi/v/pillow-jxl-plugin)
[![CI](https://github.com/isotr0py/pillow-jpegxl-plugin/workflows/CI/badge.svg)](https://github.com/isotr0py/pillow-jpegxl-plugin/actions?query=workflow%3ACI)

Pillow plugin for JPEG-XL, using Rust for bindings.

## Features
- JPEG-XL Plugin for Pillow
- Encoder/Decoder to work with JPEG-XL using safe wrapper
- Support EXIF metadata encoding

## Install via PIP
```
pip install pillow-jxl-plugin
```

## Build from source
Make sure [`Rust`](https://www.rust-lang.org/tools/install) installed, then run:
```
git clone https://github.com/Isotr0py/pillow-jpegxl-plugin
cd pillow-jpegxl-plugin

pip install -e .[dev] -v
```
If you have [`libjxl`](https://github.com/libjxl/libjxl) installed and want to use dynamic link, run:
```
pip install -e .[dev] -v --config-settings=build-args="--features=dynamic"
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

# encode image with JPEG-Style quality
with Image.open("example.png") as img:
    img.save("example.jxl", quality=98)

# Lossless construct from a jpeg image
with Image.open("example.jpg") as img:
    img.save("example.jxl",lossless=True)

# Decode jxl image
with Image.open("example.jxl") as img:
    display(img)
```

## Wheels status
|    Wheels   	| Windows 64-bit 	| MacOS 	| manylinux 	| musllinux 	|
|:-----------:	|:--------------:	|:-----:	|:---------:	|:---------:	|
|  CPython3.9 	|        ✔       	|   ✔   	|     ✔     	|     ✔     	|
| CPython3.10 	|        ✔       	|   ✔   	|     ✔     	|     ✔     	|
| CPython3.11 	|        ✔       	|   ✔   	|     ✔     	|     ✔     	|
| CPython3.12 	|        ✔       	|   ✔   	|     ✔     	|     ✔     	|
| CPython3.13 	|        ✔       	|   ✔   	|     ✔     	|     ✔     	|
|   PyPy3.9   	|        ✔       	|   ✔   	|     ✔     	|     ✔     	|
|   PyPy3.10  	|        ✔       	|   ✔   	|     ✔     	|     ✔     	|

## Credits
- [inflation/jpegxl-rs](https://github.com/inflation/jpegxl-rs)

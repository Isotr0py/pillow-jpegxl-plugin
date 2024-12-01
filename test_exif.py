import tempfile
import pyexiv2
from PIL import Image

import pillow_jxl

with open("test/images/metadata/sample.exif", "rb") as f:
    ref_exif = f.read()
img_ori = Image.open("test/images/sample.png")
temp = tempfile.mktemp(suffix=".jxl")

img_ori.save(temp, exif=ref_exif, use_container=True)
img_enc = pyexiv2.Image(temp)
print(img_enc.read_exif())
# img_enc = pyexiv2.Image("test/images/metadata/1x1_exif_xmp.jxl")
# print(img_enc.read_exif())
# img_enc = pyexiv2.Image("test/images/metadata/1x1_exif_xmp.jpg")
# print(img_enc.read_exif())

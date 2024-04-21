import tempfile

from PIL import Image

import pillow_jxl


def test_decode():
    # Load a JPEG image
    img = Image.open("test/images/sample.jxl")

    # Compare the two images
    assert img.size == (40, 50)
    assert img.mode == "RGBA"


def test_encode():
    temp = tempfile.mktemp(suffix=".jxl")
    img_ori = Image.open("test/images/sample.png")
    img_ori.save(temp, lossless=True)

    img_enc = Image.open(temp)
    assert img_ori.size == img_enc.size == (40, 50)
    assert img_ori.mode == img_enc.mode == "RGBA"
    assert img_enc.info["icc_profile"]


def test_icc_profile():
    # Load a JPEG image
    img_ori = Image.open("test/images/icc_profile/62AHB.jpg")
    img_jxl = Image.open("test/images/icc_profile/62AHB.jxl")

    # Compare the two images
    assert img_ori.size == img_jxl.size
    assert img_ori.mode == img_jxl.mode
    assert img_ori.info["icc_profile"] == img_jxl.info["icc_profile"]

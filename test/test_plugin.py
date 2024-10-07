import tempfile

import pytest
import numpy as np
from PIL import Image

import pillow_jxl


def test_decode():
    img_jxl = Image.open("test/images/sample.jxl")
    img_png = Image.open("test/images/sample.png")

    assert img_jxl.size == img_png.size
    assert img_jxl.mode == img_png.mode == "RGBA"
    assert not img_jxl.is_animated
    assert img_jxl.n_frames == 1
    assert list(img_jxl.getdata()) == list(img_png.getdata())


def test_decode_I16():
    img_jxl = Image.open("test/images/sample_grey.jxl")
    img_png = Image.open("test/images/sample_grey.png")

    assert img_jxl.size == img_png.size
    assert img_jxl.mode == img_png.mode == "I;16"
    assert not img_jxl.is_animated
    assert img_jxl.n_frames == 1
    # assert list(img_jxl.getdata()) == list(img_png.getdata())
    assert np.allclose(np.array(img_jxl), np.array(img_png), rtol=1e-3, atol=1)


@pytest.mark.parametrize("image", ["test/images/sample.png", "test/images/sample.jpg"])
def test_encode(image):
    temp = tempfile.mktemp(suffix=".jxl")
    with open(image, mode="rb") as f:
        img_ori = Image.open(f)
        img_ori.save(temp, lossless=True)
        img_ori.save(temp, quality=98, exif=None)

    img_enc = Image.open(temp)
    assert img_ori.size == img_enc.size == (40, 50)
    assert img_ori.mode == img_enc.mode
    assert img_enc.info["icc_profile"]


def test_jpeg_encode():
    temp = tempfile.mktemp(suffix=".jxl")
    img_ori = Image.open("test/images/sample.jpg")
    img_ori.save(temp, lossless=True)

    img_enc = Image.open(temp)
    assert img_ori.size == img_enc.size == (40, 50)
    assert img_ori.mode == img_enc.mode
    assert img_enc.info["icc_profile"]


def test_icc_profile():
    # Load a JPEG image
    img_ori = Image.open("test/images/icc_profile/62AHB.jpg")
    img_jxl = Image.open("test/images/icc_profile/62AHB.jxl")

    # Compare the two images
    assert img_ori.size == img_jxl.size
    assert img_ori.mode == img_jxl.mode
    assert img_ori.info["icc_profile"] == img_jxl.info["icc_profile"]

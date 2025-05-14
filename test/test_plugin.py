import os
import tempfile

import pyexiv2
import pytest
import numpy as np
from PIL import Image

import pillow_jxl


def test_debug_mode():
    exit_code = os.system('python -Xdev -c "from pillow_jxl import JpegXLImagePlugin"')
    assert exit_code == 0


def test_decode():
    img_jxl = Image.open("test/images/sample.jxl")
    img_png = Image.open("test/images/sample.png")

    assert img_jxl.size == img_png.size
    assert img_jxl.mode == img_png.mode == "RGBA"
    assert not img_jxl.is_animated
    assert img_jxl.n_frames == 1
    assert np.allclose(np.array(img_jxl), np.array(img_png))


def test_decode_I16():
    img_jxl = Image.open("test/images/sample_grey.jxl")
    img_png = Image.open("test/images/sample_grey.png")

    assert img_jxl.size == img_png.size
    assert img_jxl.mode == img_png.mode == "I;16"
    assert not img_jxl.is_animated
    assert img_jxl.n_frames == 1
    # we need to use atol=1 here otherwise the test will fail on MacOS
    assert np.allclose(np.array(img_jxl), np.array(img_png), atol=1)


def test_decode_F():
    img_jxl = Image.open("test/images/sample_float.jxl")
    img_ppm = Image.open("test/images/sample_float.ppm")

    assert img_jxl.size == img_ppm.size
    assert img_jxl.mode == img_ppm.mode == "F"
    assert not img_jxl.is_animated
    assert img_jxl.n_frames == 1
    assert np.allclose(np.array(img_jxl), np.array(img_ppm), atol=3e-2)


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
    img_ori.save(temp, lossless=True, lossless_jpeg=True)

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


def test_metadata_decode_from_jpeg_reconstruct():
    # Load a JPEG image
    img_ori = Image.open("test/images/metadata/1x1_exif_xmp.jpg")
    img_jxl = Image.open("test/images/metadata/1x1_exif_xmp.jpg.jxl")
    assert img_ori.getexif() == img_jxl.getexif()


def test_metadata_decode_from_direct_decoding():
    # Load a PNG image
    img_ori = Image.open("test/images/metadata/1x1_exif_xmp.png")
    img_jxl = Image.open("test/images/metadata/1x1_exif_xmp.png.jxl")
    assert img_ori.getexif() == img_jxl.getexif()


def test_metadata_encode_from_jpg():
    # Load a JPEG image
    ref_img_path = "test/images/metadata/1x1_exif_xmp.jpg"
    temp = tempfile.mktemp(suffix=".jxl")
    img_ori = Image.open(ref_img_path)
    img_ori.save(temp, use_container=True)

    img_enc = Image.open(temp)
    img_enc_exiv2 = pyexiv2.Image(temp)
    img_ori_exiv2 = pyexiv2.Image(ref_img_path)
    assert img_ori.getexif() == img_enc.getexif()
    assert img_ori_exiv2.read_exif() == img_enc_exiv2.read_exif()


def test_metadata_encode_from_raw_exif():
    with open("test/images/metadata/sample.exif", "rb") as f:
        ref_exif = f.read()
    img_ori = Image.open("test/images/sample.png")
    temp = tempfile.mktemp(suffix=".jxl")
    img_ori.save(temp, exif=ref_exif)

    ref_exif = pyexiv2.ImageData(ref_exif).read_exif()
    jxl_exif = pyexiv2.Image(temp).read_exif()
    assert ref_exif == jxl_exif


def test_metadata_encode_from_pil_exif():
    exif_img_path = "test/images/metadata/1x1_exif_xmp.jpg"
    dummy_img = Image.open("test/images/sample.png")
    exif_img = Image.open(exif_img_path)
    temp = tempfile.mktemp(suffix=".jxl")
    dummy_img.save(temp, exif=exif_img.getexif().tobytes())

    ref_exif = pyexiv2.Image(exif_img_path).read_exif()
    jxl_exif = pyexiv2.Image(temp).read_exif()
    for key in ref_exif:
        # Skip UserComment and GPSAltitude as they are broken
        if key not in ("Exif.Photo.UserComment", "Exif.GPSInfo.GPSAltitude"):
            assert ref_exif[key] == jxl_exif[key]

from PIL import Image

import pillow_jxl


def test_icc_profile():
    # Load a JPEG image
    img_ori = Image.open("test/images/icc_profile/62AHB.jpg")
    img_jxl = Image.open("test/images/icc_profile/62AHB.jxl")

    # Compare the two images
    assert img_ori.size == img_jxl.size
    assert img_ori.mode == img_jxl.mode
    assert img_ori.info["icc_profile"] == img_jxl.info["icc_profile"]

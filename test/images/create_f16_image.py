import numpy as np
import OpenEXR

height, width = (50, 40)
img_f16 = np.random.rand(height, width, 3).astype("float16")

channels = {"RGB": img_f16}
header = {"compression": OpenEXR.ZIP_COMPRESSION, "type": OpenEXR.scanlineimage}

with OpenEXR.File(header, channels) as outfile:
    outfile.write("random_image_f16.exr")

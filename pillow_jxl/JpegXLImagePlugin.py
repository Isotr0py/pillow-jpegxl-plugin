import warnings
from io import BytesIO

import PIL
from packaging.version import parse
from PIL import Image, ImageFile

from pillow_jxl import Decoder, Encoder

_VALID_JXL_MODES = {"RGB", "RGBA", "L", "LA"}
DECODE_THREADS = -1 # -1 detect available cpu cores, 0 disables parallelism


def _accept(data):
    return (
        data[:2] == b"\xff\x0a"
        or data[:12] == b"\x00\x00\x00\x0c\x4a\x58\x4c\x20\x0d\x0a\x87\x0a"
        or data[4:7] == b"JXL"
    )


# parse_jxl_box is modified from https://github.com/Fraetor/jxl_decode/blob/902cd5d479f89f93df6105a22dc92f297ab77541/src/jxl_decode/jxl.py#L88-L110
def parse_jxl_box(file, file_start: int, file_size: int) -> dict:
    LBox = int.from_bytes(file[file_start : file_start + 4], "big")
    XLBox = None
    if 1 < LBox <= 8:
        raise ValueError(f"Invalid LBox at byte {file_start}.")
    if LBox == 1:
        XLBox = int.from_bytes(file[file_start + 8 : file_start + 16], "big")
        if XLBox <= 16:
            raise ValueError(f"Invalid XLBox at byte {file_start}.")
    if XLBox:
        header_length = 16
        box_length = XLBox
    else:
        header_length = 8
        if LBox == 0:
            box_length = file_size - file_start
        else:
            box_length = LBox
    box_type = file[file_start + 4 : file_start + 8]
    return {"length": box_length, "type": box_type, "offset": header_length}


class JXLImageFile(ImageFile.ImageFile):
    format = "JXL"
    format_description = "Jpeg XL image"
    __loaded = -1
    __frame = 0

    def _open(self):
        self.fc = self.fp.read()
        self._decoder = Decoder(num_threads=DECODE_THREADS)

        self.jpeg, self._jxlinfo, self._data, icc_profile = self._decoder(self.fc)
        # FIXME (Isotr0py): Maybe slow down jpeg reconstruction
        if self.jpeg:
            with Image.open(BytesIO(self._data)) as im:
                self._data = im.tobytes()
                self._size = im.size
                self.rawmode = im.mode
                self.info = im.info
                icc_profile = im.info.get("icc_profile", icc_profile)
        else:
            self._size = (self._jxlinfo.width, self._jxlinfo.height)
            self.rawmode = self._jxlinfo.mode

            # Read the exif data from the file
            # Check if it is a JXL container first:
            if self.fc[:32] == b"\x00\x00\x00\x0C\x4A\x58\x4C\x20\x0D\x0A\x87\x0A\x00\x00\x00\x14\x66\x74\x79\x70\x6A\x78\x6C\x20\x00\x00\x00\x00\x6A\x78\x6C\x20":
                file_size = len(self.fc)
                container_pointer = 32
                data_offset_not_found = True
                while data_offset_not_found:
                    box = parse_jxl_box(self.fc, container_pointer, file_size)
                    if box["type"] == b'Exif':
                        exif_container_start = container_pointer + box["offset"]
                        self.info["exif"] = self.fc[exif_container_start : exif_container_start + box["length"]]
                        if len(self.info["exif"]) > 8:
                            if self.info["exif"][4:8] == b"II\x2A\x00" or self.info["exif"][4:8] == b"MM\x00\x2A":
                                self.info["exif"] = self.info["exif"][4:]
                        data_offset_not_found = False
                    else:
                        container_pointer += box["length"]
                        if container_pointer >= file_size:
                            data_offset_not_found = False

        if icc_profile:
            self.info["icc_profile"] = icc_profile
        # NOTE (Isotr0py): PIL 10.1.0 changed the mode to property, use _mode instead
        if parse(PIL.__version__) >= parse("10.1.0"):
            self._mode = self.rawmode
        else:
            self.mode = self.rawmode
        # FIXME (Isotr0py): animation JXL hasn't supported yet
        self.is_animated = False
        self.n_frames = 1

        self.tile = []

    # TODO(Isotr0py): Support animation seeking
    # def seek(self, frame):
    #     self.load()

    #     if self.__frame + 1 != frame:
    #         # I believe JPEG XL doesn't support seeking in animations
    #         raise NotImplementedError(
    #             "Seeking more than one frame forward is currently not supported."
    #         )
    #     self.__frame = frame

    def load(self):
        if self.__loaded != self.__frame:
            if self._data is None:
                EOFError("no more frames")

            self.__loaded = self.__frame

            if self.fp and self._exclusive_fp:
                self.fp.close()
            self.fp = BytesIO(self._data)
            self.tile = [("raw", (0, 0) + self.size, 0, self.rawmode)]

        return super().load()

    # may be defined for contained formats
    def load_seek(self, pos):
        pass

    # may be defined for blocked formats (e.g. PNG)
    # def load_read(self, bytes):
    #     pass

    def tell(self):
        return self.__frame


def _save(im, fp, filename, save_all=False):
    if im.mode not in _VALID_JXL_MODES:
        raise NotImplementedError("Only RGB, RGBA, L, LA are supported.")

    info = im.encoderinfo.copy()

    # default quality is 90
    lossless = info.get("lossless", False)
    quality = 100 if lossless else info.get("quality", 90)

    decoding_speed = info.get("decoding_speed", 0)
    effort = info.get("effort", 7)
    use_container = info.get("use_container", False)
    use_original_profile = info.get("use_original_profile", False)
    jpeg_encode = info.get("lossless_jpeg", None)
    num_threads = info.get("num_threads", -1)
    compress_metadata = info.get("compress_metadata", False)

    enc = Encoder(
        mode=im.mode,
        lossless=lossless,
        quality=quality,
        decoding_speed=decoding_speed,
        effort=effort,
        use_container=use_container,
        use_original_profile=use_original_profile,
        num_threads=num_threads,
    )
    # FIXME (Isotr0py): im.filename maybe None if parse stream
    # TODO (Isotr0py): This part should be refactored in the near future
    if im.format == "JPEG" and im.filename and (jpeg_encode or jpeg_encode is None):
        if jpeg_encode is None:
            warnings.warn(
                "Using JPEG reconstruction to create lossless JXL image from JPEG. "
                "This is the default behavior for JPEG encode, if you want to "
                "disable this, please set 'lossless_jpeg'."
            )
        with open(im.filename, "rb") as f:
            data = enc(f.read(), im.width, im.height, jpeg_encode=True)
    else:
        exif = info.get("exif")
        if exif is None:
            exif = im.getexif()
            exif = exif.tobytes() if exif else None
        if exif and exif.startswith(b"Exif\x00\x00"):
            exif = exif[6:]
        metadata = {
            "exif": exif or None,
            "jumb": info.get("jumb") or None,
            "xmp": info.get("xmp") or None,
            "compress": compress_metadata,
        }
        data = enc(im.tobytes(), im.width, im.height, jpeg_encode=False, **metadata)
    fp.write(data)


Image.register_open(JXLImageFile.format, JXLImageFile, _accept)
Image.register_save(JXLImageFile.format, _save)
Image.register_extension(JXLImageFile.format, ".jxl")
Image.register_mime(JXLImageFile.format, "image/jxl")

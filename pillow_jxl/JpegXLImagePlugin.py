from io import BytesIO
from PIL import Image, ImageFile
from pillow_jxl import Decoder, Encoder

_VALID_JXL_MODES = {"RGB", "RGBA", "L", "LA"}


def _accept(data):
    return data[:2] == b'\xff\x0a' \
        or data[:12] == b'\x00\x00\x00\x0c\x4a\x58\x4c\x20\x0d\x0a\x87\x0a' \
        or data[4:7] == b"JXL"


class JXLImageFile(ImageFile.ImageFile):

    format = "JXL"
    format_description = "Jpeg XL image"
    __loaded = -1
    __frame = 0

    def _open(self):

        self.fc = self.fp.read()
        self._decoder = Decoder()

        self._jxlinfo, self._data = self._decoder(self.fc)
        # self._size = (self._jxlinfo['width'], self._jxlinfo['height'])
        # self.mode = self.rawmode = self._jxlinfo["mode"]
        self._size = (self._jxlinfo.width, self._jxlinfo.height)
        self.mode = self.rawmode = self._jxlinfo.mode
        self.tile = []

    def seek(self, frame):

        self.load()

        if self.__frame+1 != frame:
            # I believe JPEG XL doesn't support seeking in animations
            raise NotImplementedError(
                'Seeking more than one frame forward is currently not supported.'
            )
        self.__frame = frame

    def load(self):

        if self.__loaded != self.__frame:

            if self._data is None:
                EOFError('no more frames')

            self.__loaded = self.__frame

            if self.fp and self._exclusive_fp:
                self.fp.close()
            self.fp = BytesIO(self._data)
            self.tile = [("raw", (0, 0) + self.size, 0, self.rawmode)]

        return super().load()

    def tell(self):
        return self.__frame


def _save(im, fp, filename, save_all=False):

    if im.mode not in _VALID_JXL_MODES:
        raise NotImplementedError('Only RGB, RGBA, L, LA are supported.')

    info = im.encoderinfo.copy()

    # default quality is 1
    lossless = info.get('lossless', False)
    quality = 0 if lossless else 1
    decoding_speed = info.get('decoding_speed', 0)
    use_container = info.get('use_container', True)

    enc = Encoder(
        mode=im.mode,
        lossless=lossless,
        quality=quality,
        decoding_speed=decoding_speed,
        use_container=use_container
    )
    data = enc(im.tobytes(), im.width, im.height)
    fp.write(data)


Image.register_open(JXLImageFile.format, JXLImageFile, _accept)
Image.register_save(JXLImageFile.format, _save)
Image.register_extension(JXLImageFile.format, ".jxl")
Image.register_mime(JXLImageFile.format, "image/jxl")

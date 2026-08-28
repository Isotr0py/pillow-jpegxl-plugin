class ImageInfo:
    """Metadata of a decoded image."""

    mode: str
    width: int
    height: int
    num_channels: int
    has_alpha_channel: bool

class JxlBox:
    """A single metadata/auxiliary box extracted from a JPEG XL container."""

    box_type: bytes
    data: bytes

class Encoder:
    def __init__(
        self,
        mode: str,
        lossless: bool = False,
        quality: float = 1.0,
        decoding_speed: int = 0,
        effort: int = 7,
        use_container: bool = False,
        use_original_profile: bool = False,
        num_threads: int = -1,
    ) -> None:
        """Initialize a jpeg-xl encoder.

        Args:
            mode(`str`): Pillow image mode, one of RGB, RGBA, L, LA, I;16, F.
            lossless(`bool`): Whether to use lossless compression.
            quality(`float`): JPEG-equivalent quality, ignored if `lossless` is
                True.
            decoding_speed(`int`): Decoding speed tier, from 0 (slowest) to 4
                (fastest).
            effort(`int`): Encoder effort/speed tradeoff, from 1 (fastest) to
                10 (slowest).
            use_container(`bool`): Whether to use the JPEG XL container format.
            use_original_profile(`bool`): Whether to use the original color
                profile. Always treated as True when `lossless` is True.
            num_threads(`int`): Number of threads to use, -1 to auto-detect.
        """

    def __call__(
        self,
        data: bytes,
        width: int,
        height: int,
        jpeg_encode: bool,
        exif: bytes | None = None,
        jumb: bytes | None = None,
        xmp: bytes | None = None,
        compress: bool = False,
    ) -> bytes:
        """Encode a jpeg-xl image.

        Args:
            data(`bytes`): raw image bytes, or a JPEG bitstream if
                `jpeg_encode` is True.
            width(`int`): image width in pixels.
            height(`int`): image height in pixels.
            jpeg_encode(`bool`): whether to losslessly repack a JPEG
                bitstream instead of encoding raw pixels.
            exif(`bytes | None`): optional EXIF metadata to embed, ignored if
                `jpeg_encode` is True.
            jumb(`bytes | None`): optional JUMBF metadata to embed, ignored if
                `jpeg_encode` is True.
            xmp(`bytes | None`): optional XMP metadata to embed, ignored if
                `jpeg_encode` is True.
            compress(`bool`): whether to compress the embedded metadata boxes.

        Return:
            `bytes`: The encoded jpeg-xl image.
        """

class Decoder:
    def __init__(self, num_threads: int = -1) -> None:
        """Initialize a jpeg-xl decoder.

        Args:
            num_threads(`int`): Number of threads to use, -1 to auto-detect.
        """

    def __call__(
        self, data: bytes
    ) -> tuple[bool, ImageInfo, bytes, bytes, list[JxlBox]]:
        """Decode a jpeg-xl image.

        Args:
            data(`bytes`): jpeg-xl image

        Return:
            `bool`: Whether the contained image is a reconstructed JPEG.
            `ImageInfo`: The metadata of the decoded image.
            `bytes`: The decoded image data (or the reconstructed JPEG bitstream).
            `bytes`: The embedded ICC profile, empty if none is present.
            `list[JxlBox]`: The raw metadata/auxiliary boxes found in the container.
        """

class ImageInfo:
    """
        A class contains metadata of ddecoded image.
    """
    mode: str
    width: int
    height: int
    num_channels: int
    has_alpha_channel: bool


class Encoder:

    def __init__(self,
                 parallel: bool = True,
                 has_alpha: bool = False,
                 lossless: bool = True,
                 quality: float = 0.0,
                 num_threads: int = -1): ...

    def __call__(self, data: bytes, width: int, height: int, jpeg_encode: bool) -> bytes: ...
    '''
        Encode a jpeg-xl image.

        Args:
            data(`bytes`): raw image bytes

        Return:
            `bytes`: The encoded jpeg-xl image.
    '''


class Decoder:
    '''
        Initialize a jpeg-xl decoder.

        Args:
            parallel(`bool`): enable parallel decoding
    '''

    def __init__(self, num_threads: int = -1): ...

    def __call__(self, data: bytes) -> (bool, ImageInfo, bytes): ...
    '''
        Decode a jpeg-xl image.

        Args:
            data(`bytes`): jpeg-xl image

        Return:
            `bool`: If the jpeg is reconstructed
            `ImageInfo`: The metadata of decoded image
            `bytes`: The decoded image.
    '''

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
                 quality: float = 0.0): ...

    def __call__(self, data: bytes, width: int, height: int) -> bytes: ...


class Decoder:
    '''
        Initialize a jpeg-xl decoder.

        Args:
            parallel(`bool`): enable parallel decoding
    '''

    def __init__(self, parallel: bool = True): ...

    def __call__(self, data: bytes) -> (ImageInfo, bytes): ...
    '''
        Decode a jpeg-xl image.

        Args:
            data(`bytes`): jpeg-xl image

        Return:
            `ImageInfo`: The metadata of decoded image
            `bytes`: The decoded image.
    '''

import json
import shlex
import subprocess
from argparse import ArgumentParser
from io import BytesIO
from time import time

from PIL import Image

import pillow_jxl


def encode_plugin(filename, quality, effort):
    bio = BytesIO()
    t = time()
    Image.open(filename).save(bio, format="jxl", quality=quality, effort=effort)
    d = time() - t
    return len(bio.getvalue()), d


def encode_cli(filename, quality, effort):
    cmd = f'cjxl -q {quality} -e {effort} "{filename}" -'
    t = time()
    p = subprocess.run(shlex.split(cmd), check=True, capture_output=True)
    d = time() - t
    return len(p.stdout), d


def main(args):
    filename = args.image
    quality = args.quality

    result = {
        "image": filename,
        "quality": quality,
        "effort": [],
        "plugin_size": [],
        "plugin_time": [],
        "refenc_size": [],
        "refenc_time": [],
    }
    print("Quality:", quality)
    for effort in range(1, 10):
        size_plugin, time_plugin = encode_plugin(filename, quality, effort)
        size_refenc, time_refenc = encode_cli(filename, quality, effort)
        print(f"\nEffort {effort}")
        print(f"  plugin    [size: {size_plugin}, duration: {time_plugin:.3f}]")
        print(f"  reference [size: {size_refenc}, duration: {time_refenc:.3f}]")

        result["effort"].append(effort)
        result["plugin_size"].append(size_plugin)
        result["plugin_time"].append(time_plugin)
        result["refenc_size"].append(size_refenc)
        result["refenc_time"].append(time_refenc)

    if args.output_json:
        with open(args.output_json, "w") as f:
            json.dump(result, f, indent=4)


if __name__ == "__main__":
    parser = ArgumentParser()
    parser.add_argument("-i", "--image", help="Image file for encode benchmark", required=True)
    parser.add_argument("-q", "--quality", type=int, default=98, help="Quality level")
    parser.add_argument("-o", "--output-json", help="Output JSON file")
    args = parser.parse_args()
    main(args)
#!/usr/bin/env python
import json

from PIL import Image
with open("fogkfkg.json") as f:
    d = json.loads(f.read())
d = [tuple(j) for i in d for j in i]

out = Image.new("RGBA", (862, 868))
out.putdata(d)
out.save('test_out.png')

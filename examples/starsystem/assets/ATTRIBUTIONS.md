# Star System textures

The Earth albedo, normal and roughness maps are from [Huira's Solar System texture collection](https://media.huira.space/data/textures/solar_system/).
Their original author is [Solar System Scope](https://www.solarsystemscope.com/textures/), based on NASA imagery and elevation data.
Christopher Gnam adapted the Earth specular map into a roughness map for Huira.

All three Earth textures are licensed under [Creative Commons Attribution 4.0 International (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/), including the roughness adaptation and our resized copies.
See the publisher's [attribution notice](https://media.huira.space/data/textures/solar_system/ATTRIBUTIONS.txt) and [data sources and licenses](https://docs.huira.space/getting_started/data.html#solar-system-textures).
Huira's MIT code license does not apply to these textures.

| Local file | Downloaded original | Encoding |
| --- | --- | --- |
| `2k_earth_daymap.jpg` | [8k_earth_daymap.jpg](https://media.huira.space/data/textures/solar_system/8k_earth_daymap.jpg) | sRGB albedo |
| `2k_earth_normal_map.png` | [8k_earth_normal_map.tif](https://media.huira.space/data/textures/solar_system/8k_earth_normal_map.tif) | Linear tangent-space normal, XYZ in RGB |
| `2k_earth_roughness_map.png` | [8k_earth_roughness_map.tif](https://media.huira.space/data/textures/solar_system/8k_earth_roughness_map.tif) | Linear roughness, red channel |

Downloaded on 2026-11-04. All three maps were resized from 8192×4096 to 2048×1024.
The albedo was filtered with Lanczos in linear light and encoded as sRGB JPEG (quality 92).
Normal and roughness data were filtered with a triangle kernel without gamma conversion and stored as lossless 8-bit PNG.
The shader normalizes sampled normals. No channels were inverted.

Original downloads were verified against the publisher's [SHA256SUMS](https://media.huira.space/data/textures/solar_system/SHA256SUMS):

```text
88ab060b6e7d241cfc590c69f528fab2b3247b738d40124cb590999a6fe44abc  8k_earth_daymap.jpg
79d6dbbde340c653f435f5a012c026a9bed94bdcfd9714e77885f4cba046635a  8k_earth_normal_map.tif
185a6429b72850319d89eb376c8331a3ab2c594cd9effb149b325fa3f2aaa7b4  8k_earth_roughness_map.tif
```

Credit when sharing renders made with these textures:
“Textures: Solar System Scope (solarsystemscope.com), CC BY 4.0. Roughness map adapted from its specular map by Christopher Gnam for Huira; textures resized to 2K for Gilgamesh.”

The existing `sun.jpg` is not part of this download.

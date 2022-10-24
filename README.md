# Saxony Terrain Renderer
![Screenshot 2022-10-24 at 15 44 30](https://user-images.githubusercontent.com/51823519/197553641-7e73dfce-24ca-48d9-861a-1012290f0c04.png)

A large-scale real-world terrain renderer written in Rust using the Bevy game engine.

This project is developed by [Kurt Kühnert](https://github.com/Ku95) and contains the reference implementation of my bachelors thesis.
This terrain renderer focuses on visualizing large-scale terrains in a seamless, continuous, and efficient manner. 
The source code was developed as an open source plugin for the Bevy game engine and can be found here: https://github.com/Ku95/bevy_terrain.

The terrain data is taken from the [Geoportal Sachsen](https://geoportal.sachsen.de/). (GeoSN, dl-de/by-2-0)

The full version of the thsis describing the method in great detail can be found here: `todo: link to thesis`.


## Abstract

`todo:`

## Screenshots

![Screenshot 2022-10-24 at 15 49 42](https://user-images.githubusercontent.com/51823519/197553596-313ae184-c04d-4663-a11d-b623ffadff91.png)
![Screenshot 2022-10-24 at 15 49 09](https://user-images.githubusercontent.com/51823519/197553628-dba7321c-8a1e-4b98-96ab-81716d98f765.png)
![Screenshot 2022-10-24 at 15 46 34](https://user-images.githubusercontent.com/51823519/197553638-8e3ec4ea-0114-4f84-8813-ddc46a12e3b0.png)

## Instructions

`todo:`

## Controls

- `T` - toggle camera active
- move mouse to look around
- arrow keys to move the camera horizontally
- use `PageUp` and `PageDown` to move the camera vertically 
- use `Home` and `End` to increase/decrease the movement speed

- `1` - toggle DOM/DGM
- `W` - toggle wireframe
- `M` - toggle mesh morph
- `A` - toggle albedo
- `B` - toggle base color black / white
- `C` - toggle full nodes / spherical lod
- `S` - toggle lighting
- `G` - toggle filtering bilinear / trilinear + anisotropic 
- `F` - freeze frustum culling
- `P` - show patches
- `L` - show LOD
- `U` - show UVs
- `H` - decrease patch scale
- `J` - increase patch scale
- `N` - decrease grid size
- `E` - increase grid size
- `I` - decrease view distance
- `O` - increase view distance


## License
Saxony Terrain Renderer is dual-licensed under either

* MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

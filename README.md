# Saxony Terrain Renderer
![Screenshot 2022-10-24 at 15 44 30](https://user-images.githubusercontent.com/51823519/197553641-7e73dfce-24ca-48d9-861a-1012290f0c04.png)

A large-scale real-world terrain renderer written in Rust using the Bevy game engine.

This project is developed by [Kurt Kühnert](https://github.com/kurtkuehnert) and contains the reference implementation of my bachelors thesis.
This terrain renderer focuses on visualizing large-scale terrains in a seamless, continuous, and efficient manner. 
The source code was developed as an open source plugin for the Bevy game engine and can be found here: https://github.com/kurtkuehnert/bevy_terrain.

The terrain data is taken from the [Geoportal Sachsen](https://geoportal.sachsen.de/). (GeoSN, dl-de/by-2-0)

The full version of the thsis describing the method in great detail can be found here: `todo: link to thesis`.


## Abstract

Real-time rendering of large-scale terrains is a difficult problem and remains an active field of research.
The massive scale of these landscapes, where the ratio between the size of the terrain and its resolution is spanning multiple orders of magnitude, requires an efficient level of detail strategy.
It is crucial that the geometry, as well as the terrain data, are represented seamlessly at varying distances while maintaining a constant visual quality. 
This thesis investigates common techniques and previous solutions to problems associated with the rendering of height field terrains and discusses their benefits and drawbacks.
Subsequently, two solutions to the stated problems are presented, which build and expand upon the state-of-the-art rendering methods.
A seamless and efficient mesh representation is achieved by the novel Uniform Distance-Dependent Level of Detail (UDLOD) triangulation method.
This fully GPU-based algorithm subdivides a quadtree covering the terrain into small tiles, which can be culled in parallel, and are morphed seamlessly in the vertex shader, resulting in a densely and temporal consistent triangulated mesh. 
The proposed Chunked Clipmap combines the strengths of both quadtrees and clipmaps to enable efficient out-of-core paging of terrain data.
This data structure allows for constant time view-dependent access, graceful degradation if data is unavailable, and supports trilinear and anisotropic filtering.
Together these, otherwise independent, techniques enable the rendering of large-scale real-world terrains, which is demonstrated on a dataset encompassing the entire Free State of Saxony at a resolution of one meter, in real-time.

## Screenshots

![Screenshot 2022-10-24 at 15 49 42](https://user-images.githubusercontent.com/51823519/197553596-313ae184-c04d-4663-a11d-b623ffadff91.png)
![Screenshot 2022-10-24 at 15 49 09](https://user-images.githubusercontent.com/51823519/197553628-dba7321c-8a1e-4b98-96ab-81716d98f765.png)
![Screenshot 2022-10-24 at 15 46 34](https://user-images.githubusercontent.com/51823519/197553638-8e3ec4ea-0114-4f84-8813-ddc46a12e3b0.png)
![saxony_data](https://user-images.githubusercontent.com/51823519/201691234-553c6e7c-d184-4124-b6f9-81936a2e8bc2.png)

## Instructions

### Release
I will provide a precompiled version of the project here.
Simply download and install the latest version.

### From Source

If a released version is not available for your OS, or you want to experiment with the code please compile the project from source.

Clone this project:
```
git clone https://github.com/kurtkuehnert/saxony_terrain_renderer
```

**Note:** this step is temporary, the code will be included in this repo in the future
Change into the crates directory and clone the bevy_terrain plugin:
```
cd saxony_terrain_renderer/crates
git clone https://github.com/kurtkuehnert/bevy_terrain
```

**Note:** make sure your rust version is up to data
Now compile both the download tool and the terrain renderer:
```
cargo build --release --package download_tool
cargo build --release 
```

### Configuration
Before you can run the terrain renderer you first have to setup the config file and downlad the terrain data.
Simply modify the `config.toml` file found at the root of the repository or bundled with your release.

Here you have to specify in which directory the data for the terrains should be stored.
Therefore edit the `terrain_dir` field.
The `terrain` field selects wich of the different terrain configurations to use.

By default there are three terrains available: Hartenstein, Hartenstein_large and Saxony. 
The additional parameters controll the quality and appearance of the terrain.

Before the terrain can be rendered you first have to download its terrain data.
The download tool will read the `urls` field of the selected terrain and download all required data.

If you want to add an additional terrain you can provide the tiles by copying the names from this website: https://www.geodaten.sachsen.de/batch-download-4719.html.
There select the region and the municipality and copy the links by clicking on the blue button.
Then you can past the urls of the tiles into the `urls` field and the download tool will process them accordingly.

![Screenshot 2022-11-14 at 16-17-46 Batch Download - Offene Geodaten - sachsen de](https://user-images.githubusercontent.com/51823519/201697383-18c4cf86-c075-4c6a-a3cb-3a38dd99b666.png)

With the configuration set up you can now start the download tool.
**Note:** it may take a while untill all data has finish downloading
```
./download_tool
or
cargo run --release --package download_tool
```

Finally you can visualize the data with the terrain renderer.
Make sure to set the `preprocess` flag in the config to true, the first time you view each terrain, so that the terrain data can be imported.
**Note:** it may take a while untill all data has been preprocessed
```
./saxony_terrain_renderer
or
cargo run --release
```

## Controls

These are the controlls of the terrain renderer.
Use them to look at different views and adjust the terrain quality.

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

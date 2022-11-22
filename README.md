# Terrain Renderer
![Screenshot 2022-10-24 at 15 44 30](https://user-images.githubusercontent.com/51823519/197553641-7e73dfce-24ca-48d9-861a-1012290f0c04.png)

A large-scale real-world terrain renderer written in Rust using the Bevy game engine.

This project is developed by [Kurt Kühnert](https://github.com/kurtkuehnert) and contains the reference implementation of my bachelor thesis.
This terrain renderer focuses on visualizing large-scale terrains in a seamless, continuous, and efficient manner. 
The source code was developed as the open-source plugin **[bevy_terrain](https://github.com/kurtkuehnert/bevy_terrain)** for the Bevy game engine.

Additionally, this repository contains the full version of my **[Bachelor Thesis](https://github.com/kurtkuehnert/terrain_renderer/blob/main/Thesis.pdf)** describing the novel terrain rendering method in great detail.

This [Video](https://youtu.be/ZRMt1GV50nI) showcases the capabilities and features of this terrain renderer.

The terrain data is taken from the [Geoportal Sachsen (GeoSN, dl-de/by-2-0)](https://geoportal.sachsen.de/) and the [Federal Office of Topography (©swisstopo)](https://www.swisstopo.admin.ch/en/home.html).

## Abstract

Real-time rendering of large-scale terrains is a difficult problem and remains an active field of research.
The massive scale of these landscapes, where the ratio between the size of the terrain and its resolution is spanning multiple orders of magnitude, requires an efficient level of detail strategy.
It is crucial that the geometry, as well as the terrain data, are represented seamlessly at varying distances while maintaining a constant visual quality. 
This thesis investigates common techniques and previous solutions to problems associated with the rendering of height field terrains and discusses their benefits and drawbacks.
Subsequently, two solutions to the stated problems are presented, which build and expand upon the state-of-the-art rendering methods.
A seamless and efficient mesh representation is achieved by the novel Uniform Distance-Dependent Level of Detail (UDLOD) triangulation method.
This fully GPU-based algorithm subdivides a quadtree covering the terrain into small tiles, which can be culled in parallel, and are morphed seamlessly in the vertex shader, resulting in a densely and temporally consistent triangulated mesh. 
The proposed Chunked Clipmap combines the strengths of both quadtrees and clipmaps to enable efficient out-of-core paging of terrain data.
This data structure allows for constant time view-dependent access, graceful degradation if data is unavailable, and supports trilinear and anisotropic filtering.
Together these, otherwise independent, techniques enable the rendering of large-scale real-world terrains, which is demonstrated on a dataset encompassing the entire Free State of Saxony at a resolution of one meter, in real-time.

## Screenshots

![Screenshot 2022-10-24 at 15 49 42](https://user-images.githubusercontent.com/51823519/197553596-313ae184-c04d-4663-a11d-b623ffadff91.png)
![Screenshot 2022-11-18 at 11 05 05](https://user-images.githubusercontent.com/51823519/202845032-0537e929-b13c-410b-8072-4c5b5df9830d.png)
![Screenshot 2022-11-18 at 11 03 32](https://user-images.githubusercontent.com/51823519/202845038-1b0573c5-70f8-4b69-bf39-17d6c63cead5.png)
![Screenshot 2022-10-24 at 15 49 09](https://user-images.githubusercontent.com/51823519/197553628-dba7321c-8a1e-4b98-96ab-81716d98f765.png)
![Screenshot 2022-10-24 at 15 46 34](https://user-images.githubusercontent.com/51823519/197553638-8e3ec4ea-0114-4f84-8813-ddc46a12e3b0.png)
![Screenshot 2022-11-18 at 11 02 24](https://user-images.githubusercontent.com/51823519/202845040-2ad51635-0e6a-4595-a753-f4aa80c50565.png)
![saxony_data](https://user-images.githubusercontent.com/51823519/201691234-553c6e7c-d184-4124-b6f9-81936a2e8bc2.png)


## Instructions

### Release
The precompiled binaries for Windows, Linux(Debian), and MacOS are supplied on the right.
Simply download and install the latest version.

### From Source

If a released version is not available for your OS, or you want to experiment with the code, please compile the project from source.

Clone this project:
```
git clone https://github.com/kurtkuehnert/terrain_renderer
```

**Note:** make sure your rust version is up to date (`rustup update stable`)

Now compile the terrain renderer, like so:
```
cargo build --release 
```

### Configuration
Before you can run the terrain renderer you first have to set up the config file and download the terrain data.
Simply modify the `config.toml` file found at the root of the repository or bundled with your release.

Here you have to specify in which directory the data for the terrains should be stored.
Therefore edit the `terrain_dir` field.
The `terrain` field selects which of the different terrain configurations to use.
Each of them will be stored in a subdirectory with the same name.

By default, there are four terrains available: Hartenstein, Hartenstein_large, Saxony, and Uri. 
Additional parameters control the quality and appearance of the terrain. 
For more information on the, take a look [here](https://github.com/kurtkuehnert/terrain_renderer/blob/main/crates/terrain_settings/src/lib.rs#L5-L23).

**Note:** The Saxony dataset takes up over 100 GB of diskspace and is compiled from 2 TB of source data. Start by trying the Hartenstein terrain first.

Before the terrain can be rendered you first have to download its terrain data.
The downloader supports downloading data from the Swiss and Saxon datasets.

For the downloader to work it requires a list urls of the tile data.
The lists for the default terrains can be found in the `urls` directory.
Simply copy the appropriate one into the directory of the terrain.
Additionally, the process of generating such a list for any desired terrain is described below.

#### Saxony

To download a terrain from the Saxon dataset, provide the tiles by copying their urls from this website: https://www.geodaten.sachsen.de/batch-download-4719.html.
There select the region and the municipality and copy the links by clicking on the blue button.
Then you have to paste all links into a text file (.txt or .csv) and save it inside the subdirectory of the terrain you want to download.
Finally, configure the `urls_saxony` field for your terrain in the config file.
Make sure that the `side_length` is larger or equal to the maximum amount of tiles in x or y direction.

**Note:** All DTM and DSM data of the terrain is downloaded automatically as well.

```config.toml
[[terrains]]
name = "Hartenstein"
side_length = 4
urls_saxony = "urls.txt"
```

```
terrain_dir
└── Hartenstein
    └── urls.txt
```

![Screenshot 2022-11-14 at 16-17-46 Batch Download - Offene Geodaten - sachsen de](https://user-images.githubusercontent.com/51823519/201697383-18c4cf86-c075-4c6a-a3cb-3a38dd99b666.png)

#### Switzerland
The process for terrains from the Swiss dataset is similar.
Here you have to specify the dtm and dop data separately.
Select individual tiles or entire regions on the following two web pages.
https://www.swisstopo.admin.ch/en/geodata/height/alti3d.html for the height data (DTM) and https://www.swisstopo.admin.ch/en/geodata/images/ortho/swissimage10.html for the orthophotos (DOP).
Make sure the parameters match the ones in the screenshot below.
Then press the blue search button, export all links in the menu below, and download them as a csv file.
Finally copy both urls files into the terrains directory.

**Note:** Swiss DSM data is not supported.

```config.toml
[[terrains]]
name = "Bern"
side_length = 200
urls_saxony = "urls.txt"
```

```
terrain_dir
└── Bern
    ├── urls_dtm.csv
    └── urls_dop.csv
```

![Screenshot 2022-11-16 at 18-55-41 swissALTI3D](https://user-images.githubusercontent.com/51823519/202257636-9df67a16-55d6-4e70-9060-1f9f7beb6c25.png)


With the configuration set up you can now start the download tool.

**Note:** It may take a while until all data has finished downloading.
```
./download_tool
or
cargo run --release --package download_tool
```

Finally, you can visualize the data with the terrain renderer.
Make sure to set the `preprocess` flag in the config to true, the first time you view each terrain so that the terrain data can be imported.

**Note:** It may take a while until all data has been preprocessed.
```
./saxony_terrain_renderer
or
cargo run --release
```

## Controls

These are the controls of the terrain renderer.
Use them to fly over the terrain, experiment with the quality settings and enter the different debug views.

- `T` - toggle camera movement
- move the mouse to look around
- press the arrow keys to move the camera horizontally
- use `PageUp` and `PageDown` to move the camera vertically 
- use `Home` and `End` to increase/decrease the camera's movement speed

- `V` - toggle DTM/DSM
- `W` - toggle wireframe view
- `P` - toggle tile view 
- `L` - toggle lod view
- `U` - toggle uv view
- `C` - toggle node view
- `D` - toggle mesh morph
- `A` - toggle albedo
- `B` - toggle base color black / white
- `S` - toggle lighting
- `G` - toggle filtering bilinear / trilinear + anisotropic 
- `F` - freeze frustum culling
- `H` - decrease tile scale
- `J` - increase tile scale
- `N` - decrease grid size
- `E` - increase grid size
- `I` - decrease view distance
- `O` - increase view distance

- `Z` - toggle sun rotation
- `X` - decrease the sun's period duration
- `Q` - increase the sun's period duration

## License
Saxony Terrain Renderer is dual-licensed under either

* MIT License (LICENSE-MIT or http://opensource.org/licenses/MIT)
* Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)

at your option.

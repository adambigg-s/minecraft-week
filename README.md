# minecraft-week

Minecraft inspired voxel game made in one week\*\* with Rust and wgpu

### Status

\*\*There were some things that I wasn't able to finish in the initial week
that I wanted to keep working on. This version has more than 1 week of work.
The 'one week' version is preserved in a [branch that can be found here.
](https://github.com/adambigg-s/minecraft-week/tree/f239245b432c7842381b852e00c0bf91ffa11833)

![screenshot](images/showcase.png)

### Building

You will need:
- Rust v1.9 or later
- Cargo-nightly toolchain (unlikely to build w/o)

Run the project:

`$ cargo +nightly run --release -- <seed>`

Seed optional:

`$ cargo +nightly run --release`

### Goals

- Infinite procedural generation
- Player-world interaction
- Asynchronous chunk forming & meshing
- Sun shadows
- Voxel lighting & ambient occlusion
- Physics and collision detection

#### Were they achieved?

Sun shadows weren't able to be added in the initial week

Terrain generation was brought to a satisfactory point (visually) but the module
responsible for generating terrain is a mess and not scalable for biomes, which
was a goal

### Resources

- https://sotrh.github.io/learn-wgpu/
- https://playspacefarer.com/voxel-meshing/
- https://github.com/jdah/minecraft-again.git
- https://0fps.net/2013/07/03/ambient-occlusion-for-minecraft-like-worlds/
- Reinventing Minecraft world generation by Henrik Kniberg
- https://web.archive.org/web/20201112035552/https://www.seedofandromeda.com/blogs/29-fast-flood-fill-lighting-in-a-blocky-voxel-game-pt-1 

### Examples

There are some photos in `images/` that I took while making this.

### License

Use any part for whatever you want.

![screenshot](images/flying.gif)


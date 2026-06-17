# minecraft-week
Minecraft inspired voxel game made in one week with Rust and wgpu

## Current state
![screenshot](images/trees.png)
![screenshot](images/meshing.png)

## Goals
- Infinite world generation
- World interaction
- Async chunk generation
- Sun shadows
- Voxel lighting

## Building
You will need:
- Rust v1.9 or later
- Cargo-nightly toolchain (unlikely to build w/o)

Run the project:
$ cargo run --release -- <seed>

##### Files that are in disarray
- terrain.rs
- minecraft_week.rs - the main file is so bad

##### Blocks to add
- ores (coal, iron etc.)

###### Goals for today (last day)
- add voxel AO
- fix structures generating over chunk borders
- refactor terrain so that the file is readable



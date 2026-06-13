# minecraft-week
Minecraft inspired voxel game made in one week with Rust and wgpu

### Current state
![screenshot](images/skybox_working.png)

### Goals
- Infinite world generation
- Player collision
- World interaction
- Async chunk generation
- Sun shadows
- Voxel lighting

#### Notes

##### Files that are in disarray
- mesher.rs
  - it works, but the chunks have responsibility of meshing themselves in a little tangled way
- chunk.rs
- main file

##### Logic that needs changed
- terrain gen is infinite, but it is so slow
- chunk meshing is so slow
- renderer can't unadd chunks 

##### Blocks to add
- gravel
- ores (coal, iron etc.)
- wood planks
- flower

###### Goals for today
- terrain generation (look nice)



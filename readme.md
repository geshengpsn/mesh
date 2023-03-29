# 3d mesh operation in rust 

## Mesh Types
- vertex index mesh
- half-edge mesh
- winged-edge mesh (?)

double direction convert between each type

all mesh types support bvh tree, which makes fast query possible

## Main Ability
### create mesh
- mesh primitives
- load from stl

### operate mesh
- move and rotate mesh
- boolean operation
- mesh query (fast ray cast, fast collision detection, fast distance compute)

![img](assets/bunny.png)

### mesh io
- stl

## road map
- [x] vertex indices mesh 
- [x] half-edge mesh 
- [x] stl io 
- [ ] bvh tree 
    
- [ ] obj io 

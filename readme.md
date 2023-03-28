# a 3d mesh data structure crate in rust, import, export and CSG 

## Mesh Types
- vertex index mesh
- half-edge mesh
- winged-edge mesh ?

double direction convert between each type

all mesh types support bvh tree, which makes fast query possible

## Main Ability
create mesh
- mesh primitives
- load from stl

operate mesh
- move and rotate mesh
- boolean operation
- mesh query (fast ray cast, fast collision detection, fast distance compute)


save mesh
- to stl


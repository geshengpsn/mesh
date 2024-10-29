# 3d mesh operation in rust 

## Mesh Types
- vertex index mesh
- half-edge mesh
- winged-edge mesh

double direction convert between each type

all mesh types support bvh tree, which makes fast query possible

## Main Ability
### create mesh
- mesh primitives
- load from stl

### operate mesh
- transform mesh
- boolean operation
- mesh query (fast ray cast, fast collision detection, fast distance compute)

![img](assets/bunny.png)

### mesh io
- stl
- obj

### mesh primitives
- cone
- cube
- cylinder
- plane
- sphere
- torus


## features
- [x] vertex indices mesh
    - [x] stl io
    - [x] obj io
    - [x] half-edge mesh convertion
    - [x] transform

- [x] half-edge mesh
    - [x] vertex indices mesh convertion
    - [x] transform

- [x] bvh tree
    - [x] tree data structure
    - [x] aabb bvh
    - [x] bounding sphere bvh
    - [x] bvh tree build
    - [x] fast intersect test 
    - [x] generic bounding volume support

- [ ] csg
    - [ ] csg tree
    - [ ] boolean operation
        - [ ] triangle intersect test
        - [ ] winged-egde data structure
<!-- ## develop todos
- [ ] unify all mesh types to one
    ??? user don't care what types of mesh are, dont let user read docs

- [ ] async data structure and algorothms?

## Phases 

目标基本实现

算法优化，async，多核，gpu并行优化

抽象结构优化，api设计

examples，docs

promotion: open source, publish -->

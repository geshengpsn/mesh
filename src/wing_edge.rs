// use std::rc::{Rc, Weak};

// struct Vertex<G> {
//     geometry: G,
//     edge: Weak<WingEdge<G>>,
// }

// struct Face<G> {
//     loops: Vec<Weak<Loop<G>>>,
// }

// struct Loop<G> {
//     edges: Vec<Weak<WingEdge<G>>>,
// }

// struct WingEdge<G> {
//     start_vertex: Weak<Vertex<G>>,
//     end_vertex: Weak<Vertex<G>>,
//     left_loop: Weak<Loop<G>>,
//     right_loop: Weak<Loop<G>>,
//     left_next_edge: Weak<WingEdge<G>>,
//     left_prev_edge: Weak<WingEdge<G>>,
//     right_next_edge: Weak<WingEdge<G>>,
//     right_prev_edge: Weak<WingEdge<G>>,
//     geometry: G,
// }

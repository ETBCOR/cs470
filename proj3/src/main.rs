use graph::prelude::*;

fn main() {
    let g: DirectedCsrGraph<usize> = GraphBuilder::new()
        .csr_layout(CsrLayout::Sorted)
        .edges(vec![(0, 1), (0, 2), (1, 2), (1, 3), (2, 3)])
        .build();
    println!("asdfa");
}

use crate::bytecode_reader::ByteCodeProto;
use crate::op::Op;

struct Node<N: Sized> {
    weight: N,
    next_outgoing_edge: Option<EdgeIndex>,
    next_incoming_edge: Option<EdgeIndex>,
}

struct Edge<E: Sized> {
    weight: E,
    next_outgoing_edge: Option<EdgeIndex>,
    next_incoming_edge: Option<EdgeIndex>,
}

struct Graph<N: Sized, E: Sized> {
    nodes: Vec<Node<N>>,
    edges: Vec<Edge<E>>,
}

struct NodeIndex(pub u32);

struct EdgeIndex(pub u32);

pub struct Block {
    data: Vec<Op>,
}

// impl Block {
//     pub fn split(&mut self, idx: u8) -> Block {
//         let (a, b) = self.data.split_at(idx as usize);
//
//         self.data = a.to_vec();
//
//         Block {
//             data: b.to_vec()
//         }
//     }
// }

//
// // test block analyse function
// pub fn analyse_prototype_blocks(proto: &ByteCodeProto) {
//     for (idx, &ins) in proto.disasm_bc.iter().enumerate() {
//         match ins {
//             Op::JMP(_, _) => {}
//             _ => {}
//         }
//     }
// }

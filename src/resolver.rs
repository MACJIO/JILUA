use crate::disasm::disasm;
use crate::op::Op;
use crate::{ByteCodeReadError, Graph};

#[derive(Debug)]
pub struct Block {
    data: Vec<u32>,
}

impl Block {
    #[inline(always)]
    pub fn new() -> Self {
        Block { data: vec![] }
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline(always)]
    pub fn from_ins_vec(data: Vec<u32>) -> Self {
        Block { data }
    }

    #[inline(always)]
    pub fn split(&mut self, idx: usize) -> Block {
        Block {
            data: self.data.split_off(idx),
        }
    }

    #[inline(always)]
    pub fn data(&self) -> &Vec<u32> {
        &self.data
    }
}

pub fn resolve_basic_blocks(bc_raw: &[u32]) -> Result<Graph<Block, ()>, ByteCodeReadError> {
    let mut graph: Graph<Block, ()> = Graph::new();
    let block = Block::from_ins_vec(bc_raw.to_vec());

    let proto_len = block.len();

    graph.add_node(0, block);

    let mut next_node_index_option: Option<u32> = Some(0);

    let mut skip = false;

    while let Some(next_node_index) = next_node_index_option {
        let block = graph.node_weight_mut(next_node_index).unwrap();

        // first element is index of next block after jump in current block
        // second element is absolute jump address index
        let mut jump_data_option: Option<(u32, u32)> = None;

        for (idx, &ins_raw) in block.data().iter().enumerate() {
            // to get abs index we need to do idx + next_node_index
            // so idx is not absolute!
            match disasm(ins_raw)? {
                Op::JMP(_, jump) => {
                    jump_data_option = Some((
                        (idx + 1) as u32,
                        (next_node_index as i32 + (idx + 1) as i32 + jump.0 as i32) as u32,
                    ));
                    break;
                }
                Op::UCLO(_, jump) => {
                    jump_data_option = Some((
                        (idx + 1) as u32,
                        (next_node_index as i32 + (idx + 1) as i32 + jump.0 as i32) as u32,
                    ));
                    break;
                }
                Op::ISNEXT(_, jump)
                | Op::FORI(_, jump)
                | Op::JFORI(_, jump)
                | Op::FORL(_, jump)
                | Op::IFORL(_, jump)
                | Op::ITERL(_, jump)
                | Op::IITERL(_, jump)
                | Op::LOOP(_, jump)
                | Op::ILOOP(_, jump) => {
                    jump_data_option = Some((
                        (idx + 1) as u32,
                        (next_node_index as i32 + (idx + 1) as i32 + jump.0 as i32) as u32,
                    ));
                    break;
                }
                _ => {}
            }

            // if last instruction in block and not last block in proto without any jump instructions
            // we need to analyze next block and add edge if it is not end of proto
            if (idx + 1 == block.len()) && (next_node_index as usize + idx + 1 != proto_len) {
                next_node_index_option = Some(next_node_index + idx as u32 + 1);
                skip = true;
            }
        }

        if let Some(jump_data) = jump_data_option {
            if !graph.exists(next_node_index + jump_data.0) {
                graph.split_node(next_node_index, next_node_index + jump_data.0, |block| {
                    block.split(jump_data.0 as usize)
                });
            }

            graph.add_edge((), next_node_index, next_node_index + jump_data.0);

            // get node index to split for jump address index
            if let Some(node_index) = graph.try_node_with_max_index_less_then_or_equal(jump_data.1)
            {
                if node_index == jump_data.1 {
                    // already split, so we need only to add edge
                    // infinity loop?
                } else if node_index == next_node_index {
                    // split on current resolving block
                    graph.split_node(node_index, jump_data.1, |block| {
                        block.split((jump_data.1 - node_index) as usize)
                    });
                } else {
                    graph.split_node(node_index, jump_data.1, |block| {
                        block.split((jump_data.1 - node_index) as usize)
                    });
                }
                graph.add_edge((), node_index, jump_data.1);
            }

            next_node_index_option = Some(next_node_index + jump_data.0);
        } else if !skip {
            next_node_index_option = None;
        } else {
            skip = false;
            graph.add_edge((), next_node_index, next_node_index_option.unwrap());
        }
    }

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use crate::resolver::{resolve_basic_blocks};


    #[test]
    fn it_works() {
        let bc: Vec<u32> = vec![
            196660, 327, 319, 297, 566, 786, 1067, 2148336968, 1833, 67638, 526357, 67881,
            2148009805, 68406, 185207608, 396037, 2147683160, 133929, 185207589, 17498400,
            2146961231, 50529606, 2146501970, 131404,
        ];

        resolve_basic_blocks(bc);
    }
}

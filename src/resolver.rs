use crate::disasm::disasm;
use crate::op::Op;
use crate::types::Jump;
use crate::{DecompileError, Graph};

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

#[derive(Debug)]
pub enum BranchKind {
    True,
    False,
    Unconditional,
    LoopInit,
    Loop
}

fn recurse_block(
    graph: &mut Graph<Block, BranchKind>,
    bc_raw: &[u32],
    idx: u32,
) -> Result<(), DecompileError> {
    // checks if passed index is contained in existing node
    let block_exists = if let Some(prev_idx) = graph.try_prev_node(idx) {
        // block already exists
        if idx == prev_idx {
            return Ok(());
        }

        let dist = (idx - prev_idx) as usize;
        let block = graph.node_weight(prev_idx).unwrap();

        // checks if current block contains passed index
        if block.len() > dist {
            graph.split_node(prev_idx, idx, |block| block.split(dist));
            true
        } else {
            // we need to create new block
            false
        }
    } else {
        // we also need to create new block
        false
    };

    if !block_exists {
        let block_start_idx = idx;
        let next_block = graph.try_next_node(block_start_idx);
        let mut prev_cond_idx = None;

        for (idx, &ins_raw) in bc_raw.iter().enumerate().skip(block_start_idx as usize) {
            // add block it has no jumps in the end
            if Some(idx as u32) == next_block {
                graph.add_node(
                    block_start_idx,
                    Block::from_ins_vec(bc_raw[block_start_idx as usize..idx as usize].to_vec()),
                );
                graph.add_edge(BranchKind::Unconditional, block_start_idx, idx as u32);

                return Ok(());
            }

            match disasm(ins_raw)? {
                // save conditional instruction index for determine unconditional jumps
                Op::ISLT(_, _)
                | Op::ISGE(_, _)
                | Op::ISLE(_, _)
                | Op::ISGT(_, _)
                | Op::ISEQV(_, _)
                | Op::ISNEV(_, _)
                | Op::ISEQS(_, _)
                | Op::ISNES(_, _)
                | Op::ISEQN(_, _)
                | Op::ISNEN(_, _)
                | Op::ISEQP(_, _)
                | Op::ISNEP(_, _)
                | Op::IST(_)
                | Op::ISF(_) => prev_cond_idx = Some(idx),
                Op::JMP(_, jump) => {
                    graph.add_node(
                        block_start_idx,
                        Block::from_ins_vec(
                            bc_raw[block_start_idx as usize..idx as usize].to_vec(),
                        ),
                    );

                    let dest_block_idx = ((idx + 1) as i32 + jump.0 as i32) as u32;
                    recurse_block(graph, bc_raw, dest_block_idx)?;

                    if prev_cond_idx.map(|v| v + 1) == Some(idx) {
                        graph.add_edge(BranchKind::True, block_start_idx, dest_block_idx);

                        let next_block_idx = (idx + 1) as u32;
                        recurse_block(graph, bc_raw, next_block_idx)?;
                        graph.add_edge(BranchKind::False, block_start_idx, next_block_idx);
                    } else {
                        graph.add_edge(BranchKind::Unconditional, block_start_idx, dest_block_idx);
                    }

                    return Ok(());
                }
                Op::UCLO(_, jump) => {
                    if jump.0 == 0 {
                        continue;
                    }

                    graph.add_node(
                        block_start_idx,
                        Block::from_ins_vec(
                            bc_raw[block_start_idx as usize..idx as usize].to_vec(),
                        ),
                    );

                    let dest_block_idx = ((idx + 1) as i32 + jump.0 as i32) as u32;
                    recurse_block(graph, bc_raw, dest_block_idx)?;
                    graph.add_edge(BranchKind::Unconditional, block_start_idx, dest_block_idx);

                    return Ok(());
                }
                Op::ISNEXT(_, jump) // todo: find out id ISNEXT is conditional
                | Op::FORL(_, jump)
                | Op::IFORL(_, jump) => {
                    graph.add_node(
                        block_start_idx,
                        Block::from_ins_vec(
                            bc_raw[block_start_idx as usize..idx as usize].to_vec(),
                        ),
                    );

                    let dest_block_idx = ((idx + 1) as i32 + jump.0 as i32) as u32;
                    recurse_block(graph, bc_raw, dest_block_idx)?;
                    graph.add_edge(BranchKind::Loop, block_start_idx, dest_block_idx);

                    return Ok(());
                }
                Op::FORI(_, jump)
                | Op::JFORI(_, jump)
                | Op::ITERL(_, jump)
                | Op::IITERL(_, jump)
                | Op::LOOP(_, jump)
                | Op::ILOOP(_, jump) => {
                    graph.add_node(
                        block_start_idx,
                        Block::from_ins_vec(
                            bc_raw[block_start_idx as usize..idx as usize].to_vec(),
                        ),
                    );

                    let dest_block_idx = ((idx + 1) as i32 + jump.0 as i32) as u32;
                    recurse_block(graph, bc_raw, dest_block_idx)?;
                    graph.add_edge(BranchKind::False, block_start_idx, dest_block_idx);

                    let next_block_idx = (idx + 1) as u32;
                    recurse_block(graph, bc_raw, next_block_idx)?;
                    graph.add_edge(BranchKind::LoopInit, block_start_idx, next_block_idx);

                    return Ok(());
                }
                Op::RET(_, _) | Op::RET0(_, _) | Op::RET1(_, _) | Op::RETM(_, _) => {
                    graph.add_node(
                        block_start_idx,
                        Block::from_ins_vec(bc_raw[block_start_idx as usize..].to_vec()),
                    );

                    return Ok(());
                }
                _ => {}
            }
        }

        graph.add_node(
            block_start_idx,
            Block::from_ins_vec(bc_raw[block_start_idx as usize..].to_vec()),
        );
    }

    Ok(())
}

pub fn resolve_basic_blocks(bc_raw: &[u32]) -> Result<Graph<Block, BranchKind>, DecompileError> {
    let mut graph: Graph<Block, BranchKind> = Graph::new();

    recurse_block(&mut graph, bc_raw, 0)?;

    // println!("GRAPH \n{:?}\n", graph);
    Ok(graph)
}

#[cfg(test)]
mod tests {
    use crate::resolver::resolve_basic_blocks;

    #[test]
    fn it_works() {
        let bc: Vec<u32> = vec![
            196660, 327, 319, 297, 566, 786, 1067, 2148336968, 1833, 67638, 526357, 67881,
            2148009805, 68406, 185207608, 396037, 2147683160, 133929, 185207589, 17498400,
            2146961231, 50529606, 2146501970, 131404,
        ];

        resolve_basic_blocks(&bc[..]);
    }
}

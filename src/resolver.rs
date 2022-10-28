use crate::disasm::disasm;
use crate::op::Op;
use crate::{DecompileError, graph};
use crate::graph::Graph;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub enum BranchKind {
    True,
    False,
    Unconditional,
    Loop,
    LoopOut,
    LoopBody,
    LoopIter,
}

fn recurse_block(
    graph: &mut Graph<Block, BranchKind>,
    bc_raw: &[u32],
    idx: u32,
) -> Result<(), DecompileError> {
    // checks if passed index is contained in existing node
    let block_exists = if let Some(prev_idx) = graph.try_prev_node(idx) {
        // block already exists (block already analyzed)
        if idx == prev_idx {
            return Ok(());
        }

        let dist = (idx - prev_idx) as usize;
        let block = graph.node_weight(prev_idx).unwrap();

        // checks if current block contains passed index
        if block.len() > dist {
            graph.split_node(prev_idx, idx, |block| block.split(dist));
            // add unconditional branch
            graph.add_edge(BranchKind::Unconditional, prev_idx, idx);
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

        let block_size = bc_raw.len();

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
                | Op::ISTC(_, _)
                | Op::ISNEV(_, _)
                | Op::ISEQS(_, _)
                | Op::ISNES(_, _)
                | Op::ISEQN(_, _)
                | Op::ISNEN(_, _)
                | Op::ISEQP(_, _)
                | Op::ISNEP(_, _)
                | Op::IST(_)
                | Op::ISF(_)
                | Op::ISFC(_, _) => prev_cond_idx = Some(idx),
                // pairs() or next() iterator "for" loop
                // it is also unconditional branch
                Op::ISNEXT(_, jump) => {
                    // add block including current instruction
                    graph.add_node(
                        block_start_idx,
                        Block::from_ins_vec(
                            bc_raw[block_start_idx as usize..=idx as usize].to_vec(),
                        ),
                    );

                    // calculate dest jump address
                    let dest_block_idx = ((idx + 1) as i32 + jump.0 as i32) as u32;
                    recurse_block(graph, bc_raw, dest_block_idx)?;

                    // add edge for loop (unconditional)
                    let current_block_start_idx = graph.try_prev_node(idx as u32).unwrap();
                    graph.add_edge(BranchKind::LoopIter, current_block_start_idx, dest_block_idx);

                    return Ok(());
                }
                // branch to loop body in iterator "for" loop
                // it is conditional branch
                Op::ITERL(_, jump)
                | Op::IITERL(_, jump) => {
                    graph.add_node(
                        block_start_idx,
                        Block::from_ins_vec(
                            bc_raw[block_start_idx as usize..=idx as usize].to_vec(),
                        ),
                    );

                    // calculate dest jump address
                    let dest_block_idx = ((idx + 1) as i32 + jump.0 as i32) as u32;
                    recurse_block(graph, bc_raw, dest_block_idx)?;

                    // add edge to loop body (like True condition)
                    let current_block_start_idx = graph.try_prev_node(idx as u32).unwrap();
                    graph.add_edge(BranchKind::LoopBody, current_block_start_idx, dest_block_idx);

                    // analyze loop out block
                    let next_block_idx = (idx + 1) as u32;
                    recurse_block(graph, bc_raw, next_block_idx)?;

                    let current_block_start_idx = graph.try_prev_node(idx as u32).unwrap();
                    graph.add_edge(BranchKind::LoopOut, current_block_start_idx, next_block_idx);

                    return Ok(());
                }
                // don't know what to do with this instructions, please create an issue if occurs
                Op::JITERL(_, _) => { unimplemented!("JITERL instruction") }
                Op::JFORL(_, _) => { unimplemented!("JFORL instruction") }
                // numeric "for" loop initialization
                // it is conditional branch
                Op::FORI(_, jump)
                | Op::JFORI(_, jump) => {
                    graph.add_node(
                        block_start_idx,
                        Block::from_ins_vec(
                            bc_raw[block_start_idx as usize..=idx as usize].to_vec(),
                        ),
                    );

                    // calculate dest jump address
                    let dest_block_idx = ((idx + 1) as i32 + jump.0 as i32) as u32;
                    recurse_block(graph, bc_raw, dest_block_idx)?;

                    // add edge to loop out (like False condition)
                    let current_block_start_idx = graph.try_prev_node(idx as u32).unwrap();
                    graph.add_edge(BranchKind::LoopOut, current_block_start_idx, dest_block_idx);

                    // analyze loop body block
                    let next_block_idx = (idx + 1) as u32;
                    recurse_block(graph, bc_raw, next_block_idx)?;

                    let current_block_start_idx = graph.try_prev_node(idx as u32).unwrap();
                    graph.add_edge(BranchKind::LoopBody, current_block_start_idx, next_block_idx);

                    return Ok(());
                }
                // branch to loop body in numeric "for" loop
                // it is conditional
                Op::FORL(_, jump)
                | Op::IFORL(_, jump) => {
                    graph.add_node(
                        block_start_idx,
                        Block::from_ins_vec(
                            bc_raw[block_start_idx as usize..=idx as usize].to_vec(),
                        ),
                    );

                    // calculate dest jump address
                    let dest_block_idx = ((idx + 1) as i32 + jump.0 as i32) as u32;
                    recurse_block(graph, bc_raw, dest_block_idx)?;

                    // add edge to loop body (like True condition)
                    let current_block_start_idx = graph.try_prev_node(idx as u32).unwrap();
                    graph.add_edge(BranchKind::LoopBody, current_block_start_idx, dest_block_idx);

                    // analyze loop out block
                    let next_block_idx = (idx + 1) as u32;
                    recurse_block(graph, bc_raw, next_block_idx)?;

                    let current_block_start_idx = graph.try_prev_node(idx as u32).unwrap();
                    graph.add_edge(BranchKind::LoopOut, current_block_start_idx, next_block_idx);

                    return Ok(());
                }
                Op::UCLO(_, jump) => {
                    // if jump.0 == 0 => non branching

                    // add node end with current instruction
                    graph.add_node(
                        block_start_idx,
                        Block::from_ins_vec(
                            bc_raw[block_start_idx as usize..=idx as usize].to_vec(),
                        ),
                    );

                    let dest_block_idx = ((idx + 1) as i32 + jump.0 as i32) as u32;
                    recurse_block(graph, bc_raw, dest_block_idx)?;

                    let current_block_start_idx = graph.try_prev_node(idx as u32).unwrap();
                    graph.add_edge(BranchKind::Unconditional, current_block_start_idx, dest_block_idx);

                    return Ok(());
                }
                Op::JMP(_, jump) => {
                    graph.add_node(
                        block_start_idx,
                        Block::from_ins_vec(
                            bc_raw[block_start_idx as usize..=idx as usize].to_vec(),
                        ),
                    );

                    let dest_block_idx = ((idx + 1) as i32 + jump.0 as i32) as u32;
                    recurse_block(graph, bc_raw, dest_block_idx)?;

                    // conditional JMP is also could be "while" or "until" loop part, but we can't
                    // determine this actually
                    if prev_cond_idx.map(|v| v + 1) == Some(idx) {
                        // block start idx can be changed in recurse_block() function
                        let current_block_start_idx = graph.try_prev_node(idx as u32).unwrap();
                        graph.add_edge(BranchKind::True, current_block_start_idx, dest_block_idx);

                        let next_block_idx = (idx + 1) as u32;
                        recurse_block(graph, bc_raw, next_block_idx)?;

                        let current_block_start_idx = graph.try_prev_node(idx as u32).unwrap();
                        graph.add_edge(BranchKind::False, current_block_start_idx, next_block_idx);
                    } else {
                        let current_block_start_idx = graph.try_prev_node(idx as u32).unwrap();
                        graph.add_edge(BranchKind::Unconditional, current_block_start_idx, dest_block_idx);
                    }

                    return Ok(());
                }
                Op::RET(_, _) | Op::RET0(_, _) | Op::RET1(_, _) | Op::RETM(_, _) => {
                    // analyze jump after RET1 case always next JMP in RET0
                    if idx + 2 <= block_size {
                        if let Op::JMP(..) = disasm(bc_raw[idx + 1])? {
                            continue;
                        }
                    }

                    graph.add_node(
                        block_start_idx,
                        Block::from_ins_vec(bc_raw[block_start_idx as usize..=idx].to_vec()),
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

    Ok(graph)
}

#[cfg(test)]
mod tests {
    use crate::{DecompileError, read_bytecode_dump, read_uleb128};
    use crate::utils::parse_luajit_bytecode_file;

    use std::fs::File;


    #[test]
    fn resolve_basic_blocks_autotest() -> Result<(), DecompileError> {
        let mut file = File::open("./blocker_test/index.lua")?;
        let luajit_txt_bytecode_file = File::open("./blocker_test/index_dissasm.txt")?;

        let parser_prototypes = parse_luajit_bytecode_file(luajit_txt_bytecode_file);
        let bc_dump = read_bytecode_dump(&mut file)?;

        for (proto_idx, proto) in bc_dump.prototypes().iter().enumerate() {
            let basic_blocks = proto.basic_block_graph_ref();
            let parser_basic_blocks = parser_prototypes.get(proto_idx).unwrap();

            println!("Testing prototype {}", proto_idx);
            // check blocks quantity
            assert_eq!(basic_blocks.node_count(), parser_basic_blocks.len());

            // test
            for (block_idx, _) in basic_blocks.iter_node_weights() {
                let parser_block_option = parser_basic_blocks.get(&(block_idx as u16));
                assert!(parser_block_option.is_some());
            }

            // reverse test
            for &parser_block_idx in parser_basic_blocks.keys() {
                let block_option = basic_blocks.exists(parser_block_idx as u32);
                assert!(block_option);
            }
        }

        Ok(())
    }
}
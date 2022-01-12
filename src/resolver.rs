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

#[derive(Debug, Clone)]
pub enum BranchKind {
    True,
    False,
    Unconditional,
    LoopInit,
    Loop,
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
                | Op::ISF(_) => prev_cond_idx = Some(idx),
                Op::JMP(_, jump) => {
                    graph.add_node(
                        block_start_idx,
                        Block::from_ins_vec(
                            bc_raw[block_start_idx as usize..=idx as usize].to_vec(),
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
                            bc_raw[block_start_idx as usize..=idx as usize].to_vec(),
                        ),
                    );

                    let dest_block_idx = ((idx + 1) as i32 + jump.0 as i32) as u32;
                    recurse_block(graph, bc_raw, dest_block_idx)?;
                    graph.add_edge(BranchKind::Unconditional, block_start_idx, dest_block_idx);

                    return Ok(());
                }
                Op::ISNEXT(_, jump) // todo: find out if ISNEXT is conditional
                | Op::FORL(_, jump)
                | Op::IFORL(_, jump) => {
                    graph.add_node(
                        block_start_idx,
                        Block::from_ins_vec(
                            bc_raw[block_start_idx as usize..=idx as usize].to_vec(),
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
                            bc_raw[block_start_idx as usize..=idx as usize].to_vec(),
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
                    // analyze jump after RET1 case
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
    use crate::DecompileError;
    use crate::disasm::disasm;
    use crate::resolver::resolve_basic_blocks;

    #[test]
    fn it_works() -> Result<(), DecompileError> {
        let bc: Vec<u32> = vec![822, 50398009, 50463545, 197671, 33686338, 196622, 2147812440, 822,
                                50398009, 50463545, 263207, 33686338, 1078, 67175481, 67437625,
                                394535, 33686594, 262158, 2147550552, 459815, 1334, 83952953,
                                84215097, 525863, 33686850, 327694, 2147550808, 460071, 1590,
                                100730425, 100992569, 591655, 33687106, 393230, 2147551064, 460327,
                                1846, 117507897, 118097721, 722983, 198930, 789031, 2870, 184617785,
                                184683321, 855079, 33688386, 134940710, 33687362, 458766, 2147551320,
                                460583, 2091, 459782, 2147617112, 918535, 2148534616, 985398,
                                152045881, 1116711, 264972, 2147552088, 1182503, 168495654, 2857,
                                3113, 17041730, 1247542, 264722, 16910658, 67883, 133452, 2159937880,
                                1311751, 2149386584, 1378614, 264722, 461586, 396306, 33818946, 589838,
                                2148272472, 985398, 152439097, 1509941, 985910, 186125113, 720910,
                                2147552344, 2868, 169413437, 16910658, 67883, 133452, 985398,
                                152045881, 264722, 1706791, 168495654, 2857, 3113, 17041730, 132172,
                                2157906264, 1771830, 33622338, 589839, 2156923480, 1837366, 50399554,
                                589838, 2148928344, 1903414, 1969191, 33688386, 720910, 2148600664,
                                1903414, 2034742, 396562, 202181670, 33688386, 720910, 2148141912,
                                985910, 185994041, 2100277, 658700, 2147552600, 2166055, 203558205,
                                16911170, 68395, 133964, 2859, 3113, 3371, 67112489, 920577,
                                2150174552, 2150109013, 2297639, 790546, 2363687, 252776230, 4150,
                                270864441, 270929977, 987410, 2560551, 286396710, 33689666, 1052946,
                                271061049, 2691623, 463655, 33820738, 1051922, 265477, 2148405336,
                                4150, 270864441, 270929977, 987410, 2757159, 286396710, 33689666,
                                1052946, 271061049, 2691623, 463655, 33820738, 1050642, 2147946328,
                                3338, 2147815512, 462087, 2147553368, 2147618648, 201329686,
                                2144669528, 2058, 2148339544, 2821942, 528402, 33689410, 985106,
                                67593, 2147553112, 67625, 2887478, 252186424, 984076, 2147553368,
                                2950183, 2149257048, 69419, 3018806, 987446, 288297273, 67244098,
                                2147816280, 3085366, 336794680, 267269, 2147554392, 134955,
                                33755973, 2147029842, 983055, 2147553368, 2148208728, 987190,
                                269488185, 3150119, 4649, 4905, 17043522, 1249334, 266514,
                                16912450, 69675, 135244, 1380150, 266258, 463122, 397842,
                                33820482, 983054, 2148274008, 986934, 253103929, 3215413,
                                987446, 286789945, 1114126, 2147553880, 4404, 270078269,
                                16912194, 69419, 134988, 986934, 252710713, 266258, 1708327,
                                269553702, 4393, 4649, 17043266, 132172, 2148206936, 985398,
                                152045881, 3279399, 2857, 3113, 17041730, 1247542, 264722, 16910658,
                                67883, 133452, 65611];

        let graph = resolve_basic_blocks(&bc[..])?;
        for (block_idx, block) in graph.iter_node_weights() {
            println!("block({})", block_idx);
            for &ins in &block.data {
                println!("{:?}", disasm(ins)?);
            }
            println!();
        }

        Ok(())
    }
}

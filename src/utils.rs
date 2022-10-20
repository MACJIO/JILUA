use std::fs::File;
use std::io::{self, BufRead};
use std::collections::HashMap;
use regex::Regex;

pub fn parse_luajit_bytecode_file(file: File) -> Vec<HashMap<u16, Vec<String>>> {
    // TODO: Somehow try to collect edges
    // instruction index expression
    let ins_idx_re = Regex::new(r"([0-9]{4}).{4}[A-Z]{2,}").unwrap();
    // jump to expression
    let jump_to_re = Regex::new(r"=>\s([0-9]{4})").unwrap();
    // from jump expression
    let jump_from_re = Regex::new(r"[0-9]{4}\s=>\s[A-Z]{2,}").unwrap();
    // jump from and jump to simultaneously
    let jump_from_to_re = Regex::new(r"[0-9]{4}\s=>\s[A-Z]{2,}.*=>\s([0-9]{4})").unwrap();

    let mut prototypes: Vec<HashMap<u16, Vec<String>>> = Vec::new();

    // parser for luajit official bytecode
    for line_res in io::BufReader::new(file).lines() {
        let line = line_res.unwrap();

        // prototype start
        if line.contains("-- BYTECODE --") {
            let mut proto: HashMap<u16, Vec<String>> = HashMap::new();
            proto.insert(0, Vec::new());

            prototypes.push(proto);
        } else if let Some(cap) = ins_idx_re.captures(&line) {
            // get current instruction index from line, decrement cause starts from 1
            let curr_ins_idx = &cap[1].parse::<u16>().unwrap() - 1;

            if let Some(last_proto) = prototypes.last_mut() {
                if jump_from_to_re.is_match(&line) {
                    // create block and add current instruction
                    last_proto.insert(curr_ins_idx, vec![line]);
                    // create empty block for next instructions
                    last_proto.insert(curr_ins_idx + 1, Vec::new());
                } else if jump_to_re.is_match(&line) {
                    // add current instruction to last block
                    last_proto.iter_mut().max().unwrap().1.push(line);
                    // last_proto.iter_mut().last().unwrap().1.push(line);
                    // create empty block for next instructions
                    last_proto.insert(curr_ins_idx + 1, Vec::new());
                } else if jump_from_re.is_match(&line) {
                    // create block and add current instruction
                    last_proto.insert(curr_ins_idx, vec![line]);
                } else {
                    // just add to last block
                    last_proto.iter_mut().max().unwrap().1.push(line);
                    // last_proto.iter_mut().last().unwrap().1.push(line);
                }
            } else {
                panic!("Prototypes vector cannot be empty");
            }
        }
    }

    prototypes
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs::File;
    use std::io;
    use crate::utils::parse_luajit_bytecode_file;

    #[test]
    fn test_luajit_bytecode_parser() {
        let file = File::open("./blocker_test/test_bytecode_for_parser.txt").unwrap();

        let prototypes = parse_luajit_bytecode_file(file);

        assert_eq!(prototypes.len(), 2);

        // real blocks for first proto (block_id, block_len)
        let real_blocks: HashMap<u16, u16> = HashMap::from([
            (0, 3), (3, 1), (4, 2), (6, 5), (11, 2), (13, 5), (18, 2), (20, 7), (27, 2), (29, 2),
            (31, 1), (32, 10), (42, 5), (47, 6)
        ]);

        let first_proto = prototypes.get(0).unwrap();

        // first check quantity of blocks
        assert_eq!(real_blocks.len(), first_proto.len());

        // check block ids and lengths
        for (real_block_id, &real_block_len) in real_blocks.iter() {
            let block_option = first_proto.get(real_block_id);
            assert!(block_option.is_some());
            assert_eq!(block_option.unwrap().len(), real_block_len as usize);
        }

        // test second proto
        let real_blocks: HashMap<u16, u16> = HashMap::from([
            (0, 8), (8, 5), (13, 4), (17, 3), (20, 1), (21, 2), (23, 1)
        ]);

        let second_proto = prototypes.get(1).unwrap();

        assert_eq!(real_blocks.len(), second_proto.len());

        for (real_block_id, &real_block_len) in real_blocks.iter() {
            let block_option = second_proto.get(real_block_id);
            assert!(block_option.is_some());
            assert_eq!(block_option.unwrap().len(), real_block_len as usize);
        }
    }
}
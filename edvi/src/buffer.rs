//
// Copyright (c) 2024 Jeff Garzik
//
// This file is part of the posixutils-rs project covered under
// the MIT License.  For the full license text, please see the LICENSE
// file in the root directory of this project.
// SPDX-License-Identifier: MIT
//

pub const MAX_CHUNK: usize = 1000000;

#[derive(Clone, Debug)]
pub struct Chunk {
    data: String,

    lines: usize,
    first_line: usize,
    last_line: usize,
}

impl Chunk {
    pub fn new() -> Chunk {
        Chunk {
            data: String::new(),
            lines: 0,
            first_line: 0,
            last_line: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_edge(&self, line_no: usize) -> bool {
        (line_no == self.first_line) || (line_no == self.last_line)
    }

    pub fn push_line(&mut self, line: &str) {
        self.data.push_str(line);
        self.lines += 1;
    }
}

pub fn as_chunks(vs: &[String]) -> Vec<Chunk> {
    let mut chunks: Vec<Chunk> = Vec::new();

    let mut chunk = Chunk::new();
    for line in vs {
        if chunk.len() + line.len() > MAX_CHUNK {
            chunks.push(chunk);
            chunk = Chunk::new();
        }
        chunk.push_line(line);
    }

    if chunk.len() > 0 {
        chunks.push(chunk);
    }

    chunks
}

#[derive(Debug)]
pub struct Buffer {
    pub pathname: String,

    pub cur_line: usize,
    pub last_line: usize,

    chunks: Vec<Chunk>,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            pathname: String::new(),
            cur_line: 0,
            last_line: 0,
            chunks: Vec::new(),
        }
    }

    pub fn set_cur_line(&mut self, line: usize) {
        assert!(line <= self.last_line);
        self.cur_line = line;
    }

    pub fn append(&mut self, mut chunk: Chunk) {
        let cur_tail = self.last_line;
        let new_lines = chunk.lines;

        chunk.first_line = cur_tail + 1;
        chunk.last_line = chunk.first_line + new_lines - 1;

        if self.cur_line == 0 {
            self.cur_line = 1;
        }
        self.last_line += new_lines;

        self.chunks.push(chunk);
    }

    fn renumber(&mut self, adj: usize) {
        for chunk in &mut self.chunks {
            chunk.first_line += adj;
            chunk.last_line += adj;
        }
    }

    fn insert_head(&mut self, chunks: &[Chunk]) {
        assert!(chunks.len() > 0);

        let mut chunks = chunks.to_vec();

        // total lines in insertion; assign line numbers.
        let mut total_lines = 0;
        let mut cur_line = 0;
        for chunk in &mut chunks {
            total_lines += chunk.lines;
            chunk.first_line = cur_line + 1;
            chunk.last_line = chunk.first_line + chunk.lines - 1;
            cur_line = chunk.last_line;
        }

        // adjust line numbers of existing chunks
        self.renumber(total_lines);

        self.last_line += total_lines;

        self.chunks.splice(0..0, chunks);
    }

    fn insert_tail(&mut self, chunks: &[Chunk]) {
        for chunk in chunks {
            self.append(chunk.clone());
        }
    }

    fn insert_middle(&mut self, _line_no: usize, _insert_before: bool, _chunks: &[Chunk]) {
        // assert!(chunks.len() > 0);
        todo!();
    }

    pub fn insert(&mut self, line_no: usize, insert_before: bool, chunks: &[Chunk]) {
        if chunks.len() == 0 {
            assert!((insert_before && line_no == 1) || (!insert_before && line_no == 0));
            self.insert_tail(chunks);
        } else if insert_before && (line_no == 1 || line_no == 0) {
            self.insert_head(chunks);
        } else if !insert_before && line_no == self.last_line {
            self.insert_tail(chunks);
        } else {
            self.insert_middle(line_no, insert_before, chunks);
        }
    }
}

//
// Copyright (c) 2024 Jeff Garzik
//
// This file is part of the posixutils-rs project covered under
// the MIT License.  For the full license text, please see the LICENSE
// file in the root directory of this project.
// SPDX-License-Identifier: MIT
//

pub struct Chunk {
    data: String,

    lines: u64,
    first_line: u64,
    last_line: u64,
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

    pub fn push_line(&mut self, line: &str) {
        self.data.push_str(line);
        self.lines += 1;
    }
}

pub struct Buffer {
    pub pathname: String,

    cur_line: u64,
    last_line: u64,

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
}

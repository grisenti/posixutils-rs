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

    first_line: u64,
    last_line: u64,
}

impl Chunk {
    pub fn new(line_no: u64) -> Chunk {
        Chunk {
            data: String::new(),
            first_line: line_no,
            last_line: line_no,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn push_line(&mut self, line: &str) {
        self.data.push_str(line);
        self.last_line = self.last_line + 1;
    }
}

pub struct Buffer {
    pub pathname: String,

    last_line: u64,

    chunks: Vec<Chunk>,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            pathname: String::new(),
            last_line: 0,
            chunks: Vec::new(),
        }
    }

    pub fn append(&mut self, chunk: Chunk) {
        self.last_line = chunk.last_line;
        self.chunks.push(chunk);
    }
}

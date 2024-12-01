//
// Copyright (c) 2024 Jeff Garzik
//
// This file is part of the posixutils-rs project covered under
// the MIT License.  For the full license text, please see the LICENSE
// file in the root directory of this project.
// SPDX-License-Identifier: MIT
//

use std::fs;
use std::io::{self, Read, Write};

use gettextrs::{bind_textdomain_codeset, gettext, setlocale, textdomain, LocaleCategory};

const DEF_BLOCK_SIZE: usize = 512;

const CONV_ASCII_IBM: [u8; 256] = [
    0x0, 0x1, 0x2, 0x3, 0x37, 0x2d, 0x2e, 0x2f, 0x16, 0x5, 0x25, 0xb, 0xc, 0xd, 0xe, 0xf, 0x10,
    0x11, 0x12, 0x13, 0x3c, 0x3d, 0x32, 0x26, 0x18, 0x19, 0x3f, 0x27, 0x1c, 0x1d, 0x1e, 0x1f, 0x40,
    0x5a, 0x7f, 0x7b, 0x5b, 0x6c, 0x50, 0x7d, 0x4d, 0x5d, 0x5c, 0x4e, 0x6b, 0x60, 0x4b, 0x61, 0xf0,
    0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0x7a, 0x5e, 0x4c, 0x7e, 0x6e, 0x6f, 0x7c,
    0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7,
    0xd8, 0xd9, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xad, 0xe0, 0xbd, 0x5f, 0x6d, 0x79,
    0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97,
    0x98, 0x99, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xc0, 0x4f, 0xd0, 0xa1, 0x7, 0x20,
    0x21, 0x22, 0x23, 0x24, 0x15, 0x6, 0x17, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x9, 0xa, 0x1b, 0x30,
    0x31, 0x1a, 0x33, 0x34, 0x35, 0x36, 0x8, 0x38, 0x39, 0x3a, 0x3b, 0x4, 0x14, 0x3e, 0xe1, 0x41,
    0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58,
    0x59, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76,
    0x77, 0x78, 0x80, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f, 0x90, 0x9a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f,
    0xa0, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf, 0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8,
    0xb9, 0xba, 0xbb, 0xbc, 0xbd, 0xbe, 0xbf, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf, 0xda, 0xdb, 0xdc,
    0xdd, 0xde, 0xdf, 0xea, 0xeb, 0xec, 0xed, 0xee, 0xef, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
];

const CONV_EBCDIC_ASCII: [u8; 256] = [
    0x0, 0x1, 0x2, 0x3, 0x9c, 0x9, 0x86, 0x7f, 0x97, 0x8d, 0x8e, 0xb, 0xc, 0xd, 0xe, 0xf, 0x10,
    0x11, 0x12, 0x13, 0x9d, 0x85, 0x8, 0x87, 0x18, 0x19, 0x92, 0x8f, 0x1c, 0x1d, 0x1e, 0x1f, 0x80,
    0x81, 0x82, 0x83, 0x84, 0xa, 0x17, 0x1b, 0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x5, 0x6, 0x7, 0x90,
    0x91, 0x16, 0x93, 0x94, 0x95, 0x96, 0x4, 0x98, 0x99, 0x9a, 0x9b, 0x14, 0x15, 0x9e, 0x1a, 0x20,
    0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xd5, 0x2e, 0x3c, 0x28, 0x2b, 0x7c, 0x26,
    0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf, 0xb0, 0xb1, 0x21, 0x24, 0x2a, 0x29, 0x3b, 0x7e, 0x2d,
    0x2f, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8, 0xb9, 0xcb, 0x2c, 0x25, 0x5f, 0x3e, 0x3f, 0xba,
    0xbb, 0xbc, 0xbd, 0xbe, 0xbf, 0xc0, 0xc1, 0xc2, 0x60, 0x3a, 0x23, 0x40, 0x27, 0x3d, 0x22, 0xc3,
    0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xca,
    0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f, 0x70, 0x71, 0x72, 0x5e, 0xcc, 0xcd, 0xce, 0xcf, 0xd0, 0xd1,
    0xe5, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0xd2, 0xd3, 0xd4, 0x5b, 0xd6, 0xd7, 0xd8,
    0xd9, 0xda, 0xdb, 0xdc, 0xdd, 0xde, 0xdf, 0xe0, 0xe1, 0xe2, 0xe3, 0xe4, 0x5d, 0xe6, 0xe7, 0x7b,
    0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0xe8, 0xe9, 0xea, 0xeb, 0xec, 0xed, 0x7d,
    0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f, 0x50, 0x51, 0x52, 0xee, 0xef, 0xf0, 0xf1, 0xf2, 0xf3, 0x5c,
    0x9f, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0x30,
    0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
];

const CONV_ASCII_EBCDIC: [u8; 256] = [
    0x0, 0x1, 0x2, 0x3, 0x37, 0x2d, 0x2e, 0x2f, 0x16, 0x5, 0x25, 0xb, 0xc, 0xd, 0xe, 0xf, 0x10,
    0x11, 0x12, 0x13, 0x3c, 0x3d, 0x32, 0x26, 0x18, 0x19, 0x3f, 0x27, 0x1c, 0x1d, 0x1e, 0x1f, 0x40,
    0x5a, 0x7f, 0x7b, 0x5b, 0x6c, 0x50, 0x7d, 0x4d, 0x5d, 0x5c, 0x4e, 0x6b, 0x60, 0x4b, 0x61, 0xf0,
    0xf1, 0xf2, 0xf3, 0xf4, 0xf5, 0xf6, 0xf7, 0xf8, 0xf9, 0x7a, 0x5e, 0x4c, 0x7e, 0x6e, 0x6f, 0x7c,
    0xc1, 0xc2, 0xc3, 0xc4, 0xc5, 0xc6, 0xc7, 0xc8, 0xc9, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5, 0xd6, 0xd7,
    0xd8, 0xd9, 0xe2, 0xe3, 0xe4, 0xe5, 0xe6, 0xe7, 0xe8, 0xe9, 0xad, 0xe0, 0xbd, 0x9a, 0x6d, 0x79,
    0x81, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89, 0x91, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97,
    0x98, 0x99, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xc0, 0x4f, 0xd0, 0x5f, 0x7, 0x20,
    0x21, 0x22, 0x23, 0x24, 0x15, 0x6, 0x17, 0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x9, 0xa, 0x1b, 0x30,
    0x31, 0x1a, 0x33, 0x34, 0x35, 0x36, 0x8, 0x38, 0x39, 0x3a, 0x3b, 0x4, 0x14, 0x3e, 0xe1, 0x41,
    0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58,
    0x59, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76,
    0x77, 0x78, 0x80, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f, 0x90, 0x6a, 0x9b, 0x9c, 0x9d, 0x9e, 0x9f,
    0xa0, 0xaa, 0xab, 0xac, 0x4a, 0xae, 0xaf, 0xb0, 0xb1, 0xb2, 0xb3, 0xb4, 0xb5, 0xb6, 0xb7, 0xb8,
    0xb9, 0xba, 0xbb, 0xbc, 0xa1, 0xbe, 0xbf, 0xca, 0xcb, 0xcc, 0xcd, 0xce, 0xcf, 0xda, 0xdb, 0xdc,
    0xdd, 0xde, 0xdf, 0xea, 0xeb, 0xec, 0xed, 0xee, 0xef, 0xfa, 0xfb, 0xfc, 0xfd, 0xfe, 0xff,
];

#[allow(clippy::upper_case_acronyms)]
enum AsciiConv {
    Ascii,
    EBCDIC,
    IBM,
}

enum Conversion {
    Ascii(AsciiConv),
    Lcase,
    Ucase,
    Swab,
    Block,
    Unblock,
    Sync,
}

struct Config {
    ifile: String,
    ofile: String,
    ibs: usize,
    obs: usize,
    cbs: usize,
    seek: usize,
    skip: usize,
    count: usize,
    conversions: Vec<Conversion>,
    noerror: bool,
    notrunc: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ifile: Default::default(),
            ofile: Default::default(),
            ibs: DEF_BLOCK_SIZE,
            obs: DEF_BLOCK_SIZE,
            cbs: Default::default(),
            seek: Default::default(),
            skip: Default::default(),
            count: Default::default(),
            conversions: Default::default(),
            noerror: Default::default(),
            notrunc: Default::default(),
        }
    }
}

fn convert_ascii(data: &mut [u8], ascii_conv: &AsciiConv) {
    match ascii_conv {
        AsciiConv::Ascii => {
            for byte in data.iter_mut() {
                *byte = CONV_EBCDIC_ASCII[*byte as usize];
            }
        }
        AsciiConv::EBCDIC => {
            for byte in data.iter_mut() {
                *byte = CONV_ASCII_EBCDIC[*byte as usize];
            }
        }
        AsciiConv::IBM => {
            for byte in data.iter_mut() {
                *byte = CONV_ASCII_IBM[*byte as usize];
            }
        }
    }
}

fn convert_swab(data: &mut [u8]) {
    for chunk in data.chunks_exact_mut(2) {
        chunk.swap(0, 1);
    }
}

fn convert_lcase(data: &mut [u8]) {
    for byte in data.iter_mut() {
        if *byte >= b'A' && *byte <= b'Z' {
            *byte += 32;
        }
    }
}

fn convert_ucase(data: &mut [u8]) {
    for byte in data.iter_mut() {
        if *byte >= b'a' && *byte <= b'z' {
            *byte -= 32;
        }
    }
}

fn convert_sync(data: &mut Vec<u8>, block_size: usize) {
    let current_len = data.len();
    if current_len < block_size {
        data.resize(block_size, 0); // Pad with null bytes (0x00)
    }
}

fn convert_block(data: &mut Vec<u8>, cbs: usize) {
    let mut result = Vec::new();
    let mut line = Vec::new();

    for &byte in data.iter() {
        if byte == b'\n' {
            while line.len() < cbs {
                line.push(b' ');
            }
            result.extend_from_slice(&line[..cbs]);
            line.clear();
        } else {
            line.push(byte);
        }
    }

    if !line.is_empty() {
        while line.len() < cbs {
            line.push(b' ');
        }
        result.extend_from_slice(&line[..cbs]);
    }

    *data = result;
}

fn convert_unblock(data: &mut Vec<u8>, cbs: usize) {
    let mut result = Vec::new();
    for chunk in data.chunks(cbs) {
        let trimmed_chunk = chunk
            .iter()
            .rposition(|&b| b != b' ')
            .map_or(chunk, |pos| &chunk[..=pos]);
        result.extend_from_slice(trimmed_chunk);
        result.push(b'\n');
    }
    *data = result;
}

fn apply_conversions(data: &mut Vec<u8>, config: &Config) {
    for conversion in &config.conversions {
        match conversion {
            Conversion::Ascii(ascii_conv) => convert_ascii(data, ascii_conv),
            Conversion::Lcase => convert_lcase(data),
            Conversion::Ucase => convert_ucase(data),
            Conversion::Swab => convert_swab(data),
            Conversion::Sync => convert_sync(data, config.ibs),
            Conversion::Block => convert_block(data, config.cbs),
            Conversion::Unblock => convert_unblock(data, config.cbs),
        }
    }
}

fn copy_convert_file(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut ifile: Box<dyn Read> = if config.ifile.is_empty() {
        Box::new(io::stdin().lock())
    } else {
        Box::new(fs::File::open(&config.ifile)?)
    };
    let mut ofile: Box<dyn Write> = if config.ofile.is_empty() {
        Box::new(io::stdout().lock())
    } else {
        Box::new(fs::File::create(&config.ofile)?)
    };

    let mut ibuf = vec![0u8; config.ibs];
    let obuf = vec![0u8; config.obs];

    let mut count = 0;
    let mut skip = config.skip;
    let mut seek = config.seek;

    loop {
        if skip > 0 {
            let n = ifile.read(&mut ibuf)?;
            if n == 0 {
                break;
            }
            skip -= n;
            continue;
        }

        if seek > 0 {
            let n = ifile.read(&mut ibuf)?;
            if n == 0 {
                break;
            }
            seek -= n;
            continue;
        }

        let n = ifile.read(&mut ibuf)?;
        if n == 0 {
            break;
        }

        if config.count > 0 {
            if count >= config.count {
                break;
            }
            count += 1;
        }

        let mut ibuf = ibuf[..n].to_vec();

        apply_conversions(&mut ibuf, config);

        if config.obs != 0 {
            ofile.write_all(&ibuf)?;
        } else {
            ofile.write_all(&obuf[..n])?;
        }
    }

    Ok(())
}

fn parse_conv_list(config: &mut Config, s: &str) -> Result<(), Box<dyn std::error::Error>> {
    for convstr in s.split(",") {
        let conversion = match convstr {
            "ascii" => Conversion::Ascii(AsciiConv::Ascii),
            "ebcdic" => Conversion::Ascii(AsciiConv::EBCDIC),
            "ibm" => Conversion::Ascii(AsciiConv::IBM),
            "block" => Conversion::Block,
            "unblock" => Conversion::Unblock,
            "lcase" => Conversion::Lcase,
            "ucase" => Conversion::Ucase,
            "swab" => Conversion::Swab,
            "sync" => Conversion::Sync,
            "noerror" => {
                config.noerror = true;
                continue;
            }
            "notrunc" => {
                config.notrunc = true;
                continue;
            }
            _ => {
                eprintln!("{}: {}", gettext("invalid conv option"), convstr);
                return Err("invalid conv option".into());
            }
        };
        config.conversions.push(conversion);
    }
    Ok(())
}

fn parse_block_size(s: &str) -> Result<usize, Box<dyn std::error::Error>> {
    let mut s = s.to_string();
    let mut scale = 1;
    let suffix = s.pop().unwrap();
    if suffix.is_alphabetic() {
        match suffix {
            'c' => scale = 1,
            'w' => scale = 2,
            'b' => scale = 512,
            'k' | 'K' => scale = 1024,
            'm' | 'M' => scale = 1024 * 1024,
            'g' | 'G' => scale = 1024 * 1024 * 1024,
            _ => {
                eprintln!("{}: {}", gettext("invalid block size suffix"), suffix);
                return Err("invalid block size suffix".into());
            }
        }
    } else {
        s.push(suffix);
    }
    let size = s.parse::<usize>()?;
    Ok(size * scale)
}

fn parse_cmdline(args: &[String]) -> Result<Config, Box<dyn std::error::Error>> {
    let mut config = Config::default();

    for arg in args {
        // Split arg into option and argument
        let (op, oparg) = {
            match arg.split_once("=") {
                None => {
                    let msg = format!("{}: {}", gettext("invalid option"), arg);
                    eprintln!("{}", msg);
                    return Err(msg.into());
                }
                Some((opt, optarg)) => (opt, optarg.to_string()),
            }
        };

        // per-option processing
        match op {
            "if" => config.ifile = oparg,
            "of" => config.ofile = oparg,
            "ibs" => config.ibs = parse_block_size(&oparg)?,
            "obs" => config.obs = parse_block_size(&oparg)?,
            "bs" => {
                let block_sz = parse_block_size(&oparg)?;
                config.ibs = block_sz;
                config.obs = block_sz;
            }
            "cbs" => config.cbs = parse_block_size(&oparg)?,
            "skip" => config.skip = oparg.parse::<usize>()?,
            "seek" => config.seek = oparg.parse::<usize>()?,
            "count" => config.count = oparg.parse::<usize>()?,
            "conv" => parse_conv_list(&mut config, &oparg)?,

            _ => {
                eprintln!("{}: {}", gettext("invalid option"), op);
            }
        }
    }
    Ok(config)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setlocale(LocaleCategory::LcAll, "");
    textdomain("posixutils-rs")?;
    bind_textdomain_codeset("posixutils-rs", "UTF-8")?;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let config = parse_cmdline(&args)?;

    copy_convert_file(&config)?;

    Ok(())
}

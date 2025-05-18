use std::{
    fs,
    io::{Read, Seek, SeekFrom},
};

use crate::{BYTE, FIXED, WORD};

#[derive(Debug)]
pub struct ColorProfile {
    pub kind: WORD,
    pub flags: WORD,
    pub gamma: FIXED,
    pub reserved: [BYTE; 8],
    pub icc_profile_data: Option<Vec<u8>>,
}

impl ColorProfile {
    pub fn new(file: &mut fs::File) -> Self {
        let mut word = [0u8; 2];
        let mut fixed = [0u8; 4];
        file.read_exact(&mut word).unwrap();
        let kind = u16::from_le_bytes(word);
        file.read_exact(&mut word).unwrap();
        let flags = u16::from_le_bytes(word);
        file.read_exact(&mut fixed).unwrap();
        let gamma = i32::from_le_bytes(fixed);

        // 8 reserved bytes
        file.seek(SeekFrom::Current(8)).unwrap();

        let mut icc_profile_data = None;
        if kind == 2 {
            // Kind is ICC
            let mut dword = [0u8; 4];
            file.read_exact(&mut dword).unwrap();
            let data_length = u32::from_le_bytes(dword);
            let mut data = vec![0u8; data_length as usize];
            file.read_exact(&mut data).unwrap();
            icc_profile_data = Some(data);
        }

        Self {
            kind,
            flags,
            gamma,
            reserved: [0; 8],
            icc_profile_data,
        }
    }
}

use std::{fmt::Debug, fs, io::Read};

use crate::{read, DWORD};

#[derive(Debug)]
pub struct User {
    flags : DWORD,
    
    // bit 1 
    text : Option<String>,
    
    // bit 2
    color : Option<[u8; 4]>,
}
impl User {
}
mod gds_utils;

use std::env;
use std::fs::File;
use std::io::{self, Read};
use gds_utils::{RecordType, DataType};
use std::collections::HashSet;

fn main() -> io::Result<()> {
    //Collect command line args
    let args: Vec<String> = env::args().collect();
    if args.len() <2 {
        eprintln!("Usage: gdsii_parser <file_path>");
        return Ok(());
    }

    //will need error handling verify .gds extension


    let file_path = &args[1];
    //open gds file
    let mut file = File::open(file_path)?;
    // Read the file into a buffer
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    //Print library name
    if let Some(lib_name) = find_library_name(&buffer){
        println!("Library name: {}", lib_name);
    } else {
        println!("No library name found");
    }
    
    //Print top cells
    if let Some(top_cells) = get_top_cells(&buffer) {
        println!("Top cells:");
        for cell in top_cells {
            println!(" {}", cell);
        }
    } else {
        println!("No top cells found")
    }

    Ok(())
}

fn  find_library_name(buffer: &[u8]) -> Option<String>{
    let mut index = 0;
    while index < buffer.len(){
        if index + 4 > buffer.len(){
            return None;
        }
        
        let record_length = u16::from_be_bytes([buffer[index], buffer[index +1]]) as usize;
        let record_type = buffer[index +2];
        let data_type = buffer[index +3];

        if record_type == RecordType::LIBNAME as u8 {
            if data_type == DataType::AsciiString as u8 {
                let name_bytes = &buffer[index + 4..index + record_length];
                return Some(String::from_utf8_lossy(name_bytes).trim().to_string());
            }
        }
        index += record_length;
    }
    None
}

fn get_top_cells(buffer: &[u8]) -> Option<Vec<String>> {
    let mut all_cells = HashSet::new();
    let mut referenced_cells = HashSet::new();
    let mut index = 0;

    while index < buffer.len() {
        if index + 4 > buffer.len() {
            return None;
        }
        let record_length = u16::from_be_bytes([buffer[index], buffer[index +1]]) as usize;
        let record_type = buffer[index +2];
        let data_type = buffer[index +3];
        
        match record_type {
            // Store cell name when we find STRNAME
            rt if rt == RecordType::STRNAME as u8 => {
                if data_type == DataType::AsciiString as u8 {
                    let name = String::from_utf8_lossy(&buffer[index +4..index + record_length])
                        .trim()
                        .to_string();
                    all_cells.insert(name);
                }
            } 
            //Stoire referenced cell name when we find sname which is part of SREF
            rt if rt == RecordType::SNAME as u8=> {
                if data_type == DataType::AsciiString as u8 {
                    let name = String::from_utf8_lossy(&buffer[index + 4..index + record_length])
                    .trim()
                    .to_string();
                referenced_cells.insert(name);
                }
            }
            _ => {}
        }
        index += record_length;
    }

    let top_cells: Vec<String> = all_cells
    .into_iter()
    .filter(|cell| !referenced_cells.contains(cell))
    .collect();

    if top_cells.is_empty() {
        None
    } else {
        Some(top_cells)
    }
}
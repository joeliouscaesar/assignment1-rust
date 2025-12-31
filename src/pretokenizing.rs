use std::collections::HashMap;
use std::fs::File;
use std::io::Error;
use std::io::Read;
use std::io::Seek;
use std::time::Instant;
use memchr::memmem::Finder;
use memmap2::Mmap;
use std::sync::mpsc;
use std::thread;
use regex::bytes::Regex;

/// Determines chunk boundaries given a file 
pub fn find_chunk_boundaries(
    mut file:File,
    desired_num_chunks:u64,
    split_special_token: Option<&[u8]>
) -> Result<Vec<u64>, Error> {

    let file_size = file.metadata()?.len();
    let chunk_size = file_size / desired_num_chunks;

    let mut chunk_boundaries:Vec<u64> = Vec::new();
    for i in 0..(desired_num_chunks+1) {
        chunk_boundaries.push((i * chunk_size) as u64);
    }
    chunk_boundaries.push(file_size as u64);

    // if special token not specified, return naive boundaries
    let split_special_token = match split_special_token {
        None => return Ok(chunk_boundaries),
        Some(st) => st
    };

    // initialize our memchr finder
    let finder:Finder = Finder::new(&split_special_token);

    for bi in 1..(&chunk_boundaries.len()-1){
        let mut initial_position = chunk_boundaries[bi];
        let seek_res = file.seek(std::io::SeekFrom::Start(initial_position as u64));

        // actually not sure about this one, so we're just going to assume we hit the file end?
        match seek_res {
            Err(_) => {
                chunk_boundaries[bi] = file_size;
                break;
            },
            _ => {}
        };
        loop {
            const MINI_CHUNK_SIZE:usize = 4096;
            let mut buffer= [0u8; MINI_CHUNK_SIZE];
            let bytes_read = file.read(&mut buffer)?;
            // look for special token
            let found_at = finder.find(&buffer[..bytes_read]);
            match found_at {
                None => {
                    initial_position += MINI_CHUNK_SIZE as u64
                },
                Some(loc) => {
                    chunk_boundaries[bi] = initial_position + (loc as u64);
                    break
                }
            };
            if bytes_read < MINI_CHUNK_SIZE {
                chunk_boundaries[bi] = file_size;
                break;
            } 
       }
    }
    // remove duplicates
    let mut i:usize = 1;
    while i < chunk_boundaries.len() {
        if chunk_boundaries[i-1] == chunk_boundaries[i] {
            chunk_boundaries.remove(i);
        }else {
            i += 1;
        }
    }
    return Ok(chunk_boundaries);
}


fn add_to_hash(
    pretoken_counts:&mut HashMap<Vec<u8>, usize>, 
    mmap:&Mmap, 
    doc_start:usize, 
    doc_end:usize,
    re:&Regex){
    
    // goes through document, adds counts of pretokens to the hash 
    for pretoken_match in re.find_iter(&mmap[doc_start..doc_end]) {
        let pretoken_vec = pretoken_match.as_bytes().to_vec();
        *pretoken_counts.entry(pretoken_vec).or_insert(0) += 1;
    }
}


pub fn get_pretoken_counts(
    file_path:&str,
    threads:u64,
    split_special_token: Option<&[u8]>
) -> Option<HashMap<Vec<u8>,usize>> {

    // get's boundaries
    let file = match File::open(&file_path){
        Err(e) =>{
            println!("IO Issue: {e}");
            return None
        },
        Ok(f) => f
    };

    let boundaries = match find_chunk_boundaries(file, threads, split_special_token){
        Err(e) =>{
            println!("Boundaries Issue: {e}");
            return None
        },
        Ok(bs) => bs
    };

    // this gets consumed in boundaries so make again
    let file = match File::open(&file_path){
        Err(e) =>{
            println!("IO Issue: {e}");
            return None
        },
        Ok(f) => f
    };

    // start threads 
    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::new();
    // convert special token to a vector
    let split_special_token_vec:Option<Vec<u8>> = match split_special_token {
        Some(st) => {
            let mut st_vec = Vec::new();
            for b in st {
                st_vec.push(*b)
            };
            Some(st_vec)
        },
        None => None
    };

    for i in 1..boundaries.len() {
        // clone the sender
        let tx_clone = tx.clone();
        let section_start = boundaries[i-1] as usize;
        let section_end = boundaries[i] as usize;
        let re = match Regex::new(r"'(?:[sdmt]|ll|ve|re)| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|\s+$|\s+"){
            Err(e) =>{
                println!("Regex Issue: {e}");
                return None
            },
            Ok(re) => re
        };
        let special_token_copy = split_special_token_vec.clone();
        let mmap = unsafe{ Mmap::map(&file) };
        let mmap = match mmap {
            Err(e) =>{
                println!("Mmap Issue: {e}");
                return None                
            },
            Ok(m) => m
        };
        handles.push(thread::spawn(move ||{
            // hash to send back
            let mut pretoken_counts:HashMap<Vec<u8>, usize> = HashMap::new();

            // if not special token just do based on splits
            let special_token_copy = match special_token_copy {
                None => {
                    add_to_hash(&mut pretoken_counts, &mmap, section_start, section_end, &re);
                    let _ = tx_clone.send(pretoken_counts);
                    return
                },
                Some(st) => st
            } ;

            // get document cutoffs using memmem
            let finder:Finder = Finder::new(&special_token_copy);
            let st_locs = finder.find_iter(&mmap[section_start..section_end]);
            // iterate over cutoffs, add pretokens to hash
            let mut start:usize = 0;
            let mut any_sts = false;
            for end in st_locs {
                any_sts = true;
                if end == start {
                    continue
                }
                add_to_hash(&mut pretoken_counts, &mmap, section_start + start, section_start + end, &re);
                // go beyond next special token 
                start = end + special_token_copy.len();
            }
            // condition for no special tokens found
            if !any_sts {
                add_to_hash(&mut pretoken_counts, &mmap, section_start, section_end, &re);
            }

            let _ = tx_clone.send(pretoken_counts);
            return
        }));


    }

    drop(tx);

    // Iterate over incoming messages until all senders hang up
    let mut final_map:HashMap<Vec<u8>,usize> = HashMap::new();
    for received_map in rx {
        // Merge logic
        for (k,v) in received_map {
            *final_map.entry(k).or_insert(0) += v;
        }
    }

    return Some(final_map);

}

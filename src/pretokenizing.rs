// file for pretokenizing stuff
use std::fs::File;
use std::io::Error;
use std::io::Read;
use std::io::Seek;
use memchr::arch::all;
use memchr::memmem::Finder;
use memmap2::Mmap;
use std::collections::HashSet;
use std::sync::mpsc;
use std::thread;

pub fn find_chunk_boundaries(
    mut file:File,
    desired_num_chunks:u64,
    split_special_token: &[u8]
) -> Result<Vec<u64>, Error> {

    let file_size = file.metadata()?.len();
    let chunk_size = file_size / desired_num_chunks;

    let mut chunk_boundaries:Vec<u64> = Vec::new();
    for i in 0..(desired_num_chunks+1) {
        chunk_boundaries.push((i * chunk_size) as u64);
    }
    chunk_boundaries.push(file_size as u64);

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
            if bytes_read == 0 {
                chunk_boundaries[bi] = file_size;
                break;
            }
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


pub fn find_st_locs(
    mut file:File,
    split_special_token: &[u8]
) -> Result<Vec<usize>, Error> {
    // pretty significant speedup using mmap 47 -> 27 secs    

    // // initialize our memchr finder
    let finder:Finder = Finder::new(&split_special_token);
    let mut buffer:Vec<u8> = Vec::new();
    file.read_to_end(&mut buffer)?;
    let mmap = unsafe { Mmap::map(&file)}?;


    // println!("read {} bytes",mmap.len());
    let locs = finder.find_iter(&buffer).collect();

    return Ok(locs);
}

pub fn find_st_locs_par(
    mut file:File,
    threads:u64,
    split_special_token: &[u8]
) -> Result<Vec<usize>, Error> {
    // pretty significant speedup using mmap 47 -> 27,
    // with threads got to 15 sec

    // chunk lazy
    let file_size = file.metadata()?.len();
    let chunk_width = file_size / threads;
    let mut chunks:Vec<(u64,u64)> = Vec::new();
    for i in 0..threads {
        chunks.push((i*chunk_width, (i+1)*chunk_width));
    }
    
    // let mut buffer:Vec<u8> = Vec::new();
    // file.read_to_end(&mut buffer)?;

    let (tx, rx) = mpsc::channel();

    let mut handles = Vec::new();
    let mut sst2:Vec<u8> = Vec::new();
    for b in split_special_token {
        sst2.push(*b)
    };

    // Spawn 4 threads
    for i in 0..threads {
        let tx_clone = tx.clone(); // Each thread gets its own sender handle
        let (lower,upper) = chunks[i as usize];
        let lower = lower as usize;
        let upper = upper as usize;
        let sst3 = sst2.clone();
        let mmap = unsafe { Mmap::map(&file)}?;
        // let split_special_token2 = split_special_token.clone();
        // initialize memchar finder
        handles.push(thread::spawn(move || {
            let finder:Finder = Finder::new(&sst3);
            let locs:Vec<usize> = finder.find_iter(&mmap[lower..upper]).collect();
            // Send the partial result back
            tx_clone.send(locs).unwrap();
        }));
    }
    // IMPORTANT: Drop the original sender. 
    // The receiver will keep waiting as long as *any* sender is alive.
    // If we don't drop 'tx', the loop below will block forever.
    drop(tx);

    // Iterate over incoming messages until all senders hang up
    let mut final_map:Vec<usize> = Vec::new();
    for received_map in rx {
        // Merge logic
        final_map.append(&mut received_map.clone());
    }

    // println!("read {} bytes",mmap.len());
    return Ok(final_map);
}



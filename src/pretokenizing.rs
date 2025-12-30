use std::collections::HashMap;
// file for pretokenizing stuff
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


// pub fn find_st_locs(
//     mut file:File,
//     split_special_token: &[u8]
// ) -> Result<Vec<usize>, Error> {
//     // pretty significant speedup using mmap 47 -> 27 secs    

//     // // initialize our memchr finder
//     let finder:Finder = Finder::new(&split_special_token);
//     let mut buffer:Vec<u8> = Vec::new();
//     file.read_to_end(&mut buffer)?;
//     let mmap = unsafe { Mmap::map(&file)}?;


//     // println!("read {} bytes",mmap.len());
//     let locs = finder.find_iter(&buffer).collect();

//     return Ok(locs);
// }

// pub fn find_st_locs_par(
//     file:File,
//     threads:u64,
//     split_special_token: &[u8]
// ) -> Result<Vec<usize>, Error> {
//     // pretty significant speedup using mmap 47 -> 27,
//     // with threads got to 15 sec

//     // chunk lazy
//     let file_size = file.metadata()?.len();
//     let chunk_width = file_size / threads;
//     let mut chunks:Vec<(u64,u64)> = Vec::new();
//     for i in 0..threads {
//         chunks.push((i*chunk_width, (i+1)*chunk_width));
//     }
    
//     // let mut buffer:Vec<u8> = Vec::new();
//     // file.read_to_end(&mut buffer)?;

//     let (tx, rx) = mpsc::channel();

//     let mut handles = Vec::new();
//     let mut sst2:Vec<u8> = Vec::new();
//     for b in split_special_token {
//         sst2.push(*b)
//     };

//     // Spawn 4 threads
//     for i in 0..threads {
//         let tx_clone = tx.clone(); // Each thread gets its own sender handle
//         let (lower,upper) = chunks[i as usize];
//         let lower = lower as usize;
//         let upper = upper as usize;
//         let sst3 = sst2.clone();
//         let mmap = unsafe { Mmap::map(&file)}?;
//         // let split_special_token2 = split_special_token.clone();
//         // initialize memchar finder
//         handles.push(thread::spawn(move || {
//             let finder:Finder = Finder::new(&sst3);
//             let locs:Vec<usize> = finder.find_iter(&mmap[lower..upper]).collect();
//             // Send the partial result back
//             tx_clone.send(locs).unwrap();
//         }));
//     }
//     // IMPORTANT: Drop the original sender. 
//     // The receiver will keep waiting as long as *any* sender is alive.
//     // If we don't drop 'tx', the loop below will block forever.
//     drop(tx);

//     // Iterate over incoming messages until all senders hang up
//     let mut final_map:Vec<usize> = Vec::new();
//     for received_map in rx {
//         // Merge logic
//         final_map.append(&mut received_map.clone());
//     }

//     // println!("read {} bytes",mmap.len());
//     return Ok(final_map);
// }


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
    split_special_token: &[u8]
) -> Option<HashMap<Vec<u8>,usize>> {

    let start = Instant::now();
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
    let end = Instant::now();
    println!("found boundaries, elapsed {:?}", end - start);

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
    let mut split_special_token_vec:Vec<u8> = Vec::new();
    for b in split_special_token {
        split_special_token_vec.push(*b)
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
            // get document cutoffs using memmem
            let finder:Finder = Finder::new(&special_token_copy);
            let st_locs = finder.find_iter(&mmap[section_start..section_end]);
            // instantiate hash
            let mut pretoken_counts:HashMap<Vec<u8>, usize> = HashMap::new();
            // iterate over cutoffs, add pretokens to hash
            let mut start:usize = 0;
            for end in st_locs {
                if end == start {
                    continue
                }
                add_to_hash(&mut pretoken_counts, &mmap, section_start + start, section_start + end, &re);
                // go beyond next special token 
                start = end + special_token_copy.len();
            }

            let _ = tx_clone.send(pretoken_counts);
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


// pub fn scratch(){
//     let mybytes = b"I'm just a byte list nobody loves me";

//     let myre:Regex = match Regex::new(r"\s+just"){
//         Err(_) => return (),
//         Ok(r) => r
//     };
//     let mymatch = match myre.find(mybytes){
//         None => return (),
//         Some(m) => m
//     };
//     // cool so we can call to vec on these
//     let mymatchedbytes = mymatch.as_bytes();

//     println!("mymatch {:?}", mymatch.as_bytes());

//     // let finder:Finder = Finder::new(&b"just");
//     // let natural_loc = finder.find(mybytes);
//     // let next_loc = finder.find(&mybytes[3..]);
//     // match (natural_loc, next_loc) {
//     //     (Some(n),Some(m)) => {
//     //         println!("first {n} second {m}")
//     //     },
//     //     (_, _) => {
//     //         println!("match issue")
//     //     }
//     // };
//     // return ();
// }



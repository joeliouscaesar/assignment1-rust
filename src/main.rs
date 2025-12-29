// use fancy_regex::Regex;
use regex::bytes::Regex;
use std::collections::{HashMap, BTreeSet};
use std::hash::Hash;
use std::vec;
use memmap2::Mmap;
use memchr::memmem;
use rayon::prelude::*;
use std::time::Instant;

struct Pretoken {
    count:usize,
    alphabet_list:Vec<Vec<u8>>,
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct AlphabetPair {
    pair:(Vec<u8>,Vec<u8>),
}

#[derive(Clone)]
struct AlphabetPairInfo {
    count:usize,
    pretoken_ids:Vec<usize>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
struct AlphabetPairKey {
    count:usize,
    pair:AlphabetPair,
}

#[test]
fn alphabet_pair_key_order_check(){
let x:Vec<u8> = vec![1,12,13];
    let y:Vec<u8> = vec![1,12];
    let z:Vec<u8> = vec![3,1];


    let ap1 = AlphabetPairKey { count:12, pair:AlphabetPair{pair:(x.clone(),y.clone())}};
    let ap2 = AlphabetPairKey {count:8, pair:AlphabetPair{pair:(x.clone(),y.clone())}};
    let ap3 = AlphabetPairKey {count:8, pair:AlphabetPair{pair:(z.clone(),y.clone())}};
    let ap4 = AlphabetPairKey {count:12, pair:AlphabetPair{pair:(z.clone(),y.clone())}};
    
    let comp11 = ap1 == ap1;
    let comp12 = ap1 > ap2;
    let comp13 = ap1 > ap3;
    let comp14 = ap1 < ap4;
    let comp23 = ap2 < ap3;
    let comp24 = ap2 < ap4;
    let comp34 = ap4 > ap3;

    assert!(comp11);
    assert!(comp12);
    assert!(comp13);
    assert!(comp14);
    assert!(comp23);
    assert!(comp24);
    assert!(comp34);

}

fn regex_test<'a>(content:&'a [u8], re:&Regex, mut counts:HashMap<&'a [u8], usize>) -> HashMap<&'a [u8], usize>{
    // let content = b"I'm the content, just a string. well bytes. whatever. hopefully this works. it's a long shot, no doubt.";
    let byte_vecs = re.find_iter(content).map(|x| x.as_bytes());
    for bv in byte_vecs{
        *counts.entry(bv).or_insert(0) += 1;
    }
    return counts;
}

fn get_pretoken_list2() -> Option<Vec<Pretoken>> {
   // PRETOKENS
   
    let mut pretoken_list:Vec<Pretoken> = Vec::new();

    // compile regex
    // let re = Regex::new(r"'(?:[sdmt]|ll|ve|re)| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|\s+(?!\S)|\s+");
    let re = Regex::new(r"'(?:[sdmt]|ll|ve|re)| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|\s+$|\s+");
    // pattern match on if error
    let re = match re {
        Ok(x) => x,
        Err(e) => {
            println!("regex issue {e}");
            return None;
        }
    };

    //read in with mmap
    let fi = std::fs::File::open("data/TinyStoriesV2-GPT4-train.txt");
    // let fi = std::fs::File::open("data/fake.txt");
    let fi = match fi {
        Err(_) => return None,
        Ok(fi) => fi
    };
    let mmap = unsafe { Mmap::map(&fi)};
    let mmap = match mmap {
        Err(_) => {
            println!("mmap issue");
            return None
        },
        Ok(mmm) => mmm
    };
    // get the delimiters
    let delimiter = b"<|endoftext|>";
    let delim_len = delimiter.len();
    let finder = memmem::Finder::new(delimiter);
    // return pairs of indices that are blocks to read in (b/n delimiters)
    let mut chunks:Vec<(usize,usize)> = Vec::new();
    let mut start:usize = 0;
    for end in finder.find_iter(&mmap) {
        if end != start {
            chunks.push((start, end));
        }
        start = end + delim_len;
    }

    let pretoken_counts = chunks.par_iter().fold(|| HashMap::new(),
        |current_counts, c:&(usize,usize)| {
            regex_test(&mmap[c.0..c.1],&re,current_counts)
    }).reduce(|| HashMap::new(), 
        |mut mapa, mapb| {
            for (k,v) in mapb {
                *mapa.entry(k).or_insert(0) += v;
            }
            return mapa
    });

    let pretoken_count_len = pretoken_counts.len();
    println!("pretoken counts {pretoken_count_len}");

    for(pretoken_str, n) in pretoken_counts {
        // for pretoken objects
        let mut al = Vec::new();
        for b in pretoken_str {
            al.push(vec![b.clone()]);
        }
        let new_pretoken = Pretoken{
            count: n,
            alphabet_list: al,
        };
        pretoken_list.push(new_pretoken);
    };
    return Some(pretoken_list);
}


// fn get_pretoken_list() -> Option<Vec<Pretoken>>{
//    // PRETOKENS
//     let mut pretoken_counts:HashMap<String, usize> = HashMap::new();
//     let mut pretoken_list:Vec<Pretoken> = Vec::new();

//     // compile regex
//     let re = Regex::new(r"'(?:[sdmt]|ll|ve|re)| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|\s+(?!\S)|\s+");
//     // pattern match on if error
//     let re = match re {
//         Ok(x) => x,
//         Err(e) => {
//             println!("regex issue {e}");
//             return None;
//         }
//     };

//     // read in file
//     // let contents = fs::read_to_string("data/corpus.en");
//     let contents = fs::read_to_string("data/TinyStoriesV2-GPT4-valid.txt");
//     let contents = match contents {
//         Ok(s) => s,
//         Err(e) => {
//             println!("issue reading file: {e}");
//             return None;
//         }
//     };
//     // add values to a hash map
//     let re_iter = re.find_iter(&contents);
//     for re_match in re_iter {

//         // string pretoken
//         let group = match re_match {
//             Ok(g) => g,
//             Err(e) => {
//                 println!("issue with group {e}");
//                 continue;
//             }
//         };
//         let group = group.as_str().to_string();

//         let count = match pretoken_counts.get(&group){
//             None => 0,
//             Some(n) => *n,
//         };
//         let count = count + 1;
//         pretoken_counts.insert(group, count);

//     } 

//     for(pretoken_str, n) in pretoken_counts {
//         // for pretoken objects
//         let mut al = Vec::new();
//         for b in pretoken_str.bytes() {
//             al.push(vec![b]);
//         }
//         let new_pretoken = Pretoken{
//             count: n,
//             alphabet_list: al,
//         };
//         pretoken_list.push(new_pretoken);
//     };
//     return Some(pretoken_list);
// }

fn count_initial_pairs(pretoken_list:&Vec<Pretoken>) -> HashMap<AlphabetPair, AlphabetPairInfo>{
    let mut alphabet_pair_hash:HashMap<AlphabetPair,AlphabetPairInfo> = HashMap::new();
    for (pid, pretoken) in pretoken_list.iter().enumerate() {
        // add pairs to the alphabet pair hash 
        for i in 1..pretoken.alphabet_list.len() {
            let a1 = pretoken.alphabet_list[i-1].clone(); 
            let a2 = pretoken.alphabet_list[i].clone();
            let pair = AlphabetPair{pair:(a1,a2)};
            // gets a mutable borrow to hash entry
            let api = alphabet_pair_hash.entry(pair).or_insert(
        AlphabetPairInfo{count:0, pretoken_ids:Vec::new()}
            );
            let pretoken_already_there = match api.pretoken_ids.last() {
                None => false,
                Some(pid_value) => pid == *pid_value
            };
            if !pretoken_already_there {
                api.pretoken_ids.push(pid);
            }
            // count increment either way
            api.count += pretoken.count;
        }        
    }
    return alphabet_pair_hash;
}

fn make_alphabet_pair_tree(initial_counts:&HashMap<AlphabetPair, AlphabetPairInfo>) -> BTreeSet<AlphabetPairKey>{
    let mut ap_tree:BTreeSet<AlphabetPairKey> = BTreeSet::new();
    for(k, v) in initial_counts {
        let new_key = AlphabetPairKey{count:v.count, pair:k.clone()};
        ap_tree.insert(new_key);
    }
    return ap_tree;
}
fn new_alphabet_list(pretoken:&Pretoken, pair:&AlphabetPair) -> Vec<Vec<u8>>{
    let pair_tuple = &pair.pair;
    let mut i:usize = 1;
    let mut new_alphabet_list:Vec<Vec<u8>> = pretoken.alphabet_list.clone();
    while i < new_alphabet_list.len() {
        if new_alphabet_list[i-1] == pair_tuple.0 && new_alphabet_list[i] == pair_tuple.1 {
            let mut to_add = new_alphabet_list[i].clone();
            new_alphabet_list[i-1].append(&mut to_add);
            new_alphabet_list.remove(i);
        }
        i += 1;
    }
    return new_alphabet_list;
}

fn update_ap_structures(ap_hash:&mut HashMap<AlphabetPair,AlphabetPairInfo> ,ap_tree:&mut BTreeSet<AlphabetPairKey>, pairs_to_change:HashMap<AlphabetPair, AlphabetPairInfo>) {
    // updates both ap hash and ap tree
    for (ap, api) in pairs_to_change {
        let new_count = api.count;
        let ap_clone = ap.clone();
        // insert into the hash, get prior hash entry back
        let hash_entry = ap_hash.insert(ap, api);
        // get old tree key from old count
        let old_ap_key:Option<AlphabetPairKey> = match hash_entry {
            None => None,
            Some(api) => Some(AlphabetPairKey{count:api.count, pair:ap_clone.clone()}),
        };
        // if it's not none, remove
        let _ = match old_ap_key {
            None => false,
            Some(apk) => ap_tree.remove(&apk),
        };
        // insert new key if its count is > 0
        let new_ap_key:Option<AlphabetPairKey> = if new_count == 0 {
            None
        }else {
            Some(AlphabetPairKey{count:new_count, pair:ap_clone})
        };
        let _ = match new_ap_key {
            None => true,
            Some(apk) => ap_tree.insert(apk)
        };
    }
}

fn update_pairs_to_change(
    pairs_to_change:&mut HashMap<AlphabetPair, AlphabetPairInfo>,
    ap_hash:&HashMap<AlphabetPair, AlphabetPairInfo>,
    new_alphabet_list:&Vec<Vec<u8>>,
    pretoken:&Pretoken,
    pid:&usize){

    // count new pairs
    let mut count_change_hash:HashMap<(&Vec<u8>,&Vec<u8>), i64> = HashMap::new();
    for i in 1..new_alphabet_list.len() {
        let pair = (&new_alphabet_list[i-1], &new_alphabet_list[i]);
        let current_count = count_change_hash.entry(pair).or_insert(0);
        *current_count += pretoken.count as i64;
    }
    // remove old pairs
    for i in 1..pretoken.alphabet_list.len() {
        let pair = (&pretoken.alphabet_list[i-1], &pretoken.alphabet_list[i]);
        let current_count = count_change_hash.entry(pair).or_insert(0);
        *current_count -= pretoken.count as i64;
    }
    // update pairs to change
    for (pair, val) in count_change_hash {
        if val == 0 {
            continue
        }
        let ap = AlphabetPair{pair:(pair.0.clone(), pair.1.clone())};
        let old_entry = match ap_hash.get(&ap) {
            None => AlphabetPairInfo { count: 0, pretoken_ids: Vec::new() },
            Some(api) => api.clone()
        };
        let api = pairs_to_change.entry(ap).or_insert(old_entry);
        api.pretoken_ids.push(*pid);
        let new_count:i64 = (api.count as i64)+ val;
        api.count = new_count as usize; 
    }
}

fn train(num_merges:usize) -> Option<Vec<AlphabetPair>> {
    let t_start = Instant::now();
    println!("Starting training {t_start:?}");

    // for now just returns merges
    let mut merges:Vec<AlphabetPair> = Vec::new();

    // get list of pretokens
    let mut pretoken_list = get_pretoken_list2()?;
    // print some summary stats
    let num_pretokens = pretoken_list.len();
    println!("total number of pretokens: {num_pretokens}");

    let t_pretoke = Instant::now();
    println!("Done pretokenizing {t_pretoke:?}");
    let elapsed = t_pretoke - t_start;
    println!("elapsed: {elapsed:?}");

    // get initial counts as a hash, then sort using a BTreeMap
    let mut ap_hash = count_initial_pairs(&pretoken_list);
    // insert into BTreeMap
    let mut ap_tree = make_alphabet_pair_tree(&ap_hash);
    
    while merges.len() < num_merges {
        // get max ap from tree
        let max_ap = ap_tree.last()?;
        // get pid list from hash
        let pretoken_id_list = &ap_hash.get(&max_ap.pair)?.pretoken_ids;
        // hash of pairs to change, key AlphabetPair, values AlphabetPairInfo
        let mut pairs_to_change:HashMap<AlphabetPair, AlphabetPairInfo> = HashMap::new();
        for pid in pretoken_id_list {
            let pretoken = match pretoken_list.get_mut(*pid){
                None => continue,
                Some(pt) => pt,
            };
            // new alphabet list 
            let new_alphabet_list : Vec<Vec<u8>> = new_alphabet_list(&pretoken, &max_ap.pair);    
            // decrement counts of old pairs in our pairs to change hash
            update_pairs_to_change(&mut pairs_to_change, &ap_hash, &new_alphabet_list, &pretoken, pid);
            // replace alphabet list
            pretoken.alphabet_list = new_alphabet_list;
        }
        // have to copy here in case it gets modified in the update call
        let max_ap_clone = max_ap.clone();
        // no need to edit this, think it might cause issues as well
        merges.push(max_ap_clone.pair.clone());
        update_ap_structures(&mut ap_hash, &mut ap_tree, pairs_to_change);
        ap_tree.remove(&max_ap_clone);
    }

    let t_merges = Instant::now();
    println!("Done merging {t_merges:?}");
    let elapsed = t_merges - t_pretoke;
    println!("elapsed: {elapsed:?}");
    let elapsed = t_merges - t_start;
    println!("total train elapsed: {elapsed:?}");

    return Some(merges);
}



fn main() {
    match train(10000){
        None => println!("no merges returned"),
        Some(merges) => {

            println!("MERGES (first 10)");
            let mut i = 0;
            for merge in merges{
                let pair = merge.pair;
                println!("{pair:?}");
                i += 1;
                if i > 10 {
                    break;
                }
            }
        }
    };
    return;
}




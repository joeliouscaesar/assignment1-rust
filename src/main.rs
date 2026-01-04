use std::collections::{HashMap, BTreeSet};
use std::hash::Hash;
use std::vec;
use std::time::Instant;
use std::thread;
use std::fs;
use std::io::Write;
mod pretokenizing;
mod encode_decode;

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

fn get_pretoken_list(
    file_path:&str,
    threads:u64,
    split_special_token: Option<&[u8]>
) -> Vec<Pretoken> {

    let pretoken_counts = pretokenizing::get_pretoken_counts(file_path, threads, split_special_token)
        .expect("issue with pretoken counts");

    let mut pretoken_list:Vec<Pretoken> = Vec::new();
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
    return pretoken_list;
}


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
        let hash_entry = if new_count > 0 {
            ap_hash.insert(ap, api)
        }else{
            ap_hash.remove(&ap)
        };
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

fn flatten_pair(pair:&(Vec<u8>,Vec<u8>)) -> Vec<u8>{
    let mut flattened = pair.0.clone();
    flattened.append(&mut pair.1.clone());
    flattened
}

#[test]
fn flatten_pair_test(){
    let p1:(Vec<u8>, Vec<u8>) = (vec![0, 12, 23], vec![1,2,3]);
    let ans1:Vec<u8> = vec![0, 12, 23, 1, 2, 3];
    let p2:(Vec<u8>, Vec<u8>) = (vec![0, 12, 23], vec![]);
    let ans2:Vec<u8> = vec![0, 12, 23];
    let p3:(Vec<u8>, Vec<u8>) = (vec![], vec![]);
    let ans3:Vec<u8> = vec![];
    let p4:(Vec<u8>, Vec<u8>) = (vec![],vec![0, 12, 23]);
    let ans4:Vec<u8> = vec![0, 12, 23];
    assert_eq!(flatten_pair(&p1), ans1);
    assert_eq!(flatten_pair(&p2), ans2);
    assert_eq!(flatten_pair(&p3), ans3);
    assert_eq!(flatten_pair(&p4), ans4);
}

fn train(
    input_path:&str,
    vocab_size:usize,
    special_token:Option<&[u8]>, // takes 1 or 0 special tokens
    threads:Option<u64>
) -> Option<(HashMap<usize, Vec<u8>>, Vec<AlphabetPair>)> {

    // structs to return
    let mut vocab:HashMap<usize, Vec<u8>> = HashMap::new();
    let mut merges:Vec<AlphabetPair> = Vec::new();

    // initialize vocab
    for i in 0..256 {
        let i_u8 = i as u8;
        let i_usize = i as usize;
        vocab.insert(i_usize, vec![i_u8]);
    }
    match special_token {
        None => {},
        Some(st) => {
            _ = vocab.insert(vocab.len(), st.to_vec())
        }
    };

    // determine threads to use for pretokenization
    let available_threads = match thread::available_parallelism() {
        Ok(t) => t.get(),
        Err(e) => {
            println!("{e}");
            return None
        }
    };
    let threads = match threads {
        None => available_threads as u64,
        Some(t) => t
    };

    // get list of pretokens
    let mut pretoken_list = get_pretoken_list(input_path, threads, special_token);

    // get initial counts as a hash, then sort using a BTreeSet
    let mut ap_hash = count_initial_pairs(&pretoken_list);
    // insert into BTreeSet
    let mut ap_tree = make_alphabet_pair_tree(&ap_hash);
    
    while vocab.len() < vocab_size {
        // get max ap from tree, add to vocab/merges
        let max_ap = ap_tree.last();
        let max_ap = match max_ap {
            None => {
                break
            }, // in this case I think no more possible merges!
            Some(ap) => ap
        };

        let flat_ap = flatten_pair(&max_ap.pair.pair);
        vocab.insert(vocab.len(), flat_ap);
        merges.push(max_ap.pair.clone());

        // get pid list for this alphabet pair
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
            // update pairs to change given our new alphabet list
            update_pairs_to_change(&mut pairs_to_change, &ap_hash, &new_alphabet_list, &pretoken, pid);
            // replace alphabet list for this pretoken
            pretoken.alphabet_list = new_alphabet_list;
        }
        // update ap structures with pairs to change and remove max pair
        let max_ap_clone = max_ap.clone();
        update_ap_structures(&mut ap_hash, &mut ap_tree, pairs_to_change);
        assert!(!ap_tree.contains(&max_ap_clone));
        assert!(!ap_hash.contains_key(&max_ap_clone.pair));
    }

    return Some((vocab,merges));
}



fn main() {
    // // For Testing 
    // let datafile = "data/corpus.en";
    // let st = b"<|endoftext|>";
    // let (vocab, merges) = train(&datafile, 500, Some(st), Some(1)).expect("training failed");
    // let fi = fs::OpenOptions::new().append(false).create(true).write(true).open("myvocab.txt");
    // let mut fi = match fi {
    //     Err(e) => {println!("output file issue"); return}
    //     Ok(f) => f
    // };
    // for i in 0..vocab.len(){
    //     if let Some(s) = vocab.get(&i) {
    //         let line = format!("{:?}: {}\n", s, i);
    //         _ = fi.write(&line.as_bytes());
    //     }
    // }

    
    // // TinyStoriesTrain
    // // elapsed: 30.254248618s
    // let datafile = "data/TinyStoriesV2-GPT4-train.txt";
    // let st = b"<|endoftext|>";
    // let start = Instant::now();
    // let (vocab, _) = train(&datafile, 10000, Some(st), Some(4)).expect("training failed");
    // let end = Instant::now();
    // assert_eq!(vocab.len(), 10000);
    // println!("elapsed: {:?}", end-start);

    // OWT Train
    // elapsed: 15853.827499257s
    let datafile = "data/owt_train.txt";
    let st = b"<|endoftext|>";
    let start = Instant::now();
    let vocab_size:usize=30000;
    let (vocab, _) = train(&datafile, vocab_size, Some(st), Some(4)).expect("training failed");
    let end = Instant::now();
    assert_eq!(vocab.len(), vocab_size);
    println!("elapsed: {:?}", end-start);

    // write vocab
    let fi = fs::OpenOptions::new().append(false).create(true).write(true).open("owt_vocab.txt");
    let mut fi = match fi {
        Err(_) => {println!("output file issue"); return}
        Ok(f) => f
    };
    for i in 0..vocab.len(){
        if let Some(s) = vocab.get(&i) {
            let line = format!("{:?}: {}\n", s, i);
            _ = fi.write(&line.as_bytes());
        }
    }
    return;
}




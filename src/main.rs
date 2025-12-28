use fancy_regex::Regex;
use std::cmp::max;
// for regex
use std::fs;
use std::collections::{HashMap, BTreeSet};
use std::hash::Hash;
use std::vec;

//  Pretoken contains count and a vector list 
//  okay BTree for the alphabet pairs, keys are tuple of (count, a1, a2), values are vector of pretokens

// struct for pretokens
// lifetime specifiers are a little weird, the 'a things
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

fn get_pair_indices(pretoken:&Pretoken, pair:&AlphabetPair) -> Vec<usize> {
    let mut inds:Vec<usize> = Vec::new();
    let pair_tuple = &pair.pair;
    for i in 1..pretoken.alphabet_list.len() {
        let first_same = &pair_tuple.0 == &pretoken.alphabet_list[i-1];
        let second_same = &pair_tuple.1 == &pretoken.alphabet_list[i];
        if first_same && second_same {
            inds.push(i-1)
        }
    }
    return inds;
}

fn get_old_pairs(pretoken:&Pretoken, inds:&Vec<usize>) -> Vec<AlphabetPair> {
    // returns list of pairs that are being impacted by this merge
    let mut old_pairs:Vec<AlphabetPair> = Vec::new();
    for ind in inds {
        let ind = *ind;
        if ind > 0 {
            let old_low = AlphabetPair{pair:(pretoken.alphabet_list[ind-1].clone(), pretoken.alphabet_list[ind].clone())};
            old_pairs.push(old_low);
        }
        else if (ind+1) < (pretoken.alphabet_list.len() - 1) {
            let old_high = AlphabetPair{pair:(pretoken.alphabet_list[ind+1].clone(), pretoken.alphabet_list[ind+2].clone())};
            old_pairs.push(old_high);
        }
    }
    return old_pairs;
}

fn get_pretoken_list() -> Vec<Pretoken>{
   // PRETOKENS
    let mut pretoken_counts:HashMap<String, usize> = HashMap::new();
    let mut pretoken_list:Vec<Pretoken> = Vec::new();

    // compile regex
    let re = Regex::new(r"'(?:[sdmt]|ll|ve|re)| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|\s+(?!\S)|\s+");
    // pattern match on if error
    let re = match re {
        Ok(x) => x,
        Err(e) => {
            println!("regex issue {e}");
            return pretoken_list;
        }
    };

    // read in file
    let contents = fs::read_to_string("data/corpus.en");
    let contents = match contents {
        Ok(s) => s,
        Err(e) => {
            println!("issue reading file: {e}");
            return pretoken_list;
        }
    };
    // add values to a hash map
    let re_iter = re.find_iter(&contents);
    for re_match in re_iter {

        // string pretoken
        let group = match re_match {
            Ok(g) => g,
            Err(e) => {
                println!("issue with group {e}");
                continue;
            }
        };
        let group = group.as_str().to_string();

        let count = match pretoken_counts.get(&group){
            None => 0,
            Some(n) => *n,
        };
        let count = count + 1;
        pretoken_counts.insert(group, count);

    } 

    for(pretoken_str, n) in pretoken_counts {
        // for pretoken objects
        let mut al = Vec::new();
        for b in pretoken_str.bytes() {
            al.push(vec![b]);
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

fn update_old_pairs_to_change(
    pairs_to_change:&mut HashMap<AlphabetPair, AlphabetPairInfo>, 
    ap_hash:&HashMap<AlphabetPair, AlphabetPairInfo>,
    old_pairs:Vec<AlphabetPair>,
    pretoken:&Pretoken){
    // adds old pair information for this pretoken to the pairs to change hash
    for old_pair in old_pairs {
        let (old_count, old_pids) = match ap_hash.get(&old_pair) {
            None => (0, &Vec::new()),
            Some(api) => (api.count, &api.pretoken_ids)
        };
        // mutable reference to pairs_to_change entry
        let api = pairs_to_change.entry(old_pair).or_insert(
            AlphabetPairInfo{count:old_count, pretoken_ids:(*old_pids).clone()}
        );
        // decrement new count
        api.count -= pretoken.count;
    }
}

fn update_alphabet_list(pretoken:&mut Pretoken, pair:&AlphabetPair) {
    let pair_tuple = &pair.pair;
    let mut i:usize = 1;
    while i < pretoken.alphabet_list.len() {
        if pretoken.alphabet_list[i-1] == pair_tuple.0 && pretoken.alphabet_list[i] == pair_tuple.1 {
            let mut to_add = pretoken.alphabet_list[i].clone();
            pretoken.alphabet_list[i-1].append(&mut to_add);
            pretoken.alphabet_list.remove(i);
        }
        i += 1;
    }
}

fn get_new_pairs(pretoken:&Pretoken, pair:&AlphabetPair) -> Vec<AlphabetPair>{
    let mut new_al = pair.pair.0.clone();
    new_al.append(&mut pair.pair.1.clone());
    let mut new_pairs:Vec<AlphabetPair> = Vec::new();
    for i in 1..pretoken.alphabet_list.len() {
        if pretoken.alphabet_list[i] != new_al {
            continue
        }
        if i > 0 {
            // to avoid double counting
            if pretoken.alphabet_list[i-1] != pretoken.alphabet_list[i]{
                let lower_pair = AlphabetPair{pair:(pretoken.alphabet_list[i-1].clone(), pretoken.alphabet_list[i].clone())};
                new_pairs.push(lower_pair);
            }
        }
        if (i+1) < pretoken.alphabet_list.len() {
            let upper_pair = AlphabetPair{pair:(pretoken.alphabet_list[i].clone(), pretoken.alphabet_list[i+1].clone())};
            new_pairs.push(upper_pair);
        }
    }
    return new_pairs;
}

fn update_new_pairs_to_change(
    pairs_to_change:&mut HashMap<AlphabetPair, AlphabetPairInfo>, 
    newpairs:Vec<AlphabetPair>, 
    ap_hash:&HashMap<AlphabetPair, AlphabetPairInfo>,
    pretoken:&Pretoken,
    pid:usize){

    // adds old pair information for this pretoken to the pairs to change hash
    for new_pair in newpairs {
        let (old_count, old_pids) = match ap_hash.get(&new_pair) {
            None => (0, &Vec::new()),
            Some(api) => (api.count, &api.pretoken_ids)
        };
        // mutable reference to pairs_to_change entry
        let api = pairs_to_change.entry(new_pair).or_insert(
            AlphabetPairInfo{count:old_count, pretoken_ids:(*old_pids).clone()}
        );
        // increment new count
        api.count += pretoken.count;
        let already_there = match api.pretoken_ids.last() {
            None => false,
            Some(last_pid) => *last_pid == pid
        };
        if !already_there {
            api.pretoken_ids.push(pid);
        }
    }
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


fn train(num_merges:usize) -> Option<Vec<AlphabetPair>> {
    // for now just returns merges
    println!("Hello, world!");
    let mut merges:Vec<AlphabetPair> = Vec::new();

    // get list of pretokens
    let mut pretoken_list = get_pretoken_list();
    // print some summary stats
    let num_pretokens = pretoken_list.len();
    println!("total number of pretokens: {num_pretokens}");

    // get initial counts as a hash, then sort using a BTreeMap
    let mut ap_hash = count_initial_pairs(&pretoken_list);
    // insert into BTreeMap
    let mut ap_tree = make_alphabet_pair_tree(&ap_hash);

    // // borrow last one read only 
    // let max_ap = ap_tree.last()?;
    // let pair = &max_ap.pair.pair;
    // let count = &max_ap.count;
    // println!("max_ap {pair:?} max_count {count}");
    
    while merges.len() < num_merges {
        // get max ap from tree
        let max_ap = ap_tree.last()?;
        let print_pair = &max_ap.pair.pair;
        println!("{print_pair:?}");
        // get pid list from hash
        let pretoken_id_list = &ap_hash.get(&max_ap.pair)?.pretoken_ids;
        // hash of pairs to change, key AlphabetPair, values AlphabetPairInfo
        let mut pairs_to_change:HashMap<AlphabetPair, AlphabetPairInfo> = HashMap::new();
        for pid in pretoken_id_list {
            let mut pretoken = match pretoken_list.get_mut(*pid){
                None => continue,
                Some(pt) => pt,
            };
            let inds = get_pair_indices(&pretoken, &max_ap.pair);
            // get old pairs as AlphabetPairs
            let old_pairs = get_old_pairs(&pretoken,&inds);            
            // decrement counts of old pairs in our pairs to change hash
            update_old_pairs_to_change(&mut pairs_to_change, &ap_hash, old_pairs, &pretoken);
            // replace the maxpair in this pretoken alphabet list
            update_alphabet_list(&mut pretoken, &max_ap.pair);
            // get new pairs to add to hash
            let new_pairs = get_new_pairs(&pretoken, &max_ap.pair);
            update_new_pairs_to_change(&mut pairs_to_change, new_pairs, &ap_hash, &pretoken, *pid);
        }
        // have to copy here in case it gets modified in the update call
        let max_ap_clone = max_ap.clone();
        merges.push(max_ap_clone.pair.clone());
        update_ap_structures(&mut ap_hash, &mut ap_tree, pairs_to_change);
        ap_tree.remove(&max_ap_clone);
    }

    return Some(merges);
}



fn main() {
    match train(10){
        None => println!("no merges returned"),
        Some(merges) => {
            println!("MERGES");
            for merge in merges{
                let pair = merge.pair;
                println!("{pair:?}")
            }
        }
    };
    return;
}




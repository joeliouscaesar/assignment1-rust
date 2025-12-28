use fancy_regex::Regex;
use core::num;
use std::cmp::max;
// for regex
use std::fs;
use std::collections::{HashMap, HashSet, BTreeMap};
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
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
struct AlphabetPair {
    pair:(Vec<u8>,Vec<u8>),
}

struct AlphabetPairInfo {
    count:usize,
    pretoken_ids:Vec<usize>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
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

// def get_pair_indices(pretoken:Pretoken, pair:tuple[bytes]) -> list[int]:
//     inds = []
//     pair_bytes = flatten(pair)
//     for i in range(1, len(pretoken.alphabet_list)):
//         current_pair = pretoken.alphabet_list[i-1] + pretoken.alphabet_list[i]
//         if current_pair == pair_bytes:
//             inds.append(i-1)    
//     return inds

// fn get_pair_indices(pretoken:&Pretoken, pair:&(Vec<u8>,Vec<u8>)) -> Vec<usize> {
//     let mut inds:Vec<usize> = Vec::new();
//     for i in 1..pretoken.alphabet_list.len() {
//         let first_same = bigger_byte_vec(&pair.0, &pretoken.alphabet_list[i-1]) == 0;
//         let second_same = bigger_byte_vec(&pair.1, &pretoken.alphabet_list[i]) == 0;
//         if first_same && second_same {
//             inds.push(i-1)
//         }
//     }
//     return inds;
// }

// fn get_old_pairs(pretoken:&Pretoken, inds:&Vec<usize>) -> Vec<(Vec<u8>,Vec<u8>)> {
//     // returns list of pairs that are being impacted by this merge
//     let mut old_pairs:Vec<(Vec<u8>,Vec<u8>)> = Vec::new();
//     for ind in inds {
//         let ind = *ind;
//         if ind > 0 {
//             let old_low = (pretoken.alphabet_list[ind-1].clone(), pretoken.alphabet_list[ind].clone());
//             old_pairs.push(old_low);
//         }
//         else if (ind+1) < (pretoken.alphabet_list.len() - 1) {
//             let old_high = (pretoken.alphabet_list[ind+1].clone(), pretoken.alphabet_list[ind+2].clone());
//             old_pairs.push(old_high);
//         }
//     }
//     return old_pairs;
// }



// fn decrement_old_pairs(&mut alphabet_pair_hash:HashMap<(Vec<u8>,Vec<u8>), AlphabetPair>, &mut pairs_to_change:HashMap<(Vec<u8>,Vec<u8>), AlphabetPair>, pretoken:Pretoken, inds:Vec<usize>){
//     // going to move changed pairs from alphabet_pair
//     for ind in inds {
//         if ind > 0 {
//             let old_low = (pretoken.alphabet_list[ind-1], pretoken.alphabet_list[ind]);
//             let mut ap = match pairs_to_change.get(old_low) {
//                 None => AlphabetPair { pair: old_low.clone(), count: 0, pretoken_list: Vec::new() },
//                 Some(ap) => ap,
//             };
//             ap.count -= pretoken.count;
//             pairs_to_change.insert(old_low, ap);
//         }
//     }
//     return pairs_to_change;
// }

// fn get_new_pairs(pretoken:&mut Pretoken, pair:&(Vec<u8>,Vec<u8>)) -> Vec<(Vec<u8>,Vec<u8>)> {
//     // updates pretoken alphabet list and returns a vector of pairs to add 
//     let i:usize = 0;
//     let mut new_pairs:Vec<(Vec<u8>,Vec<u8>)> = Vec::new();
//     while i < (pretoken.alphabet_list.len()-1) {
//         if bigger_byte_vec(&pair.0, &pretoken.alphabet_list[i]) == 0 && bigger_byte_vec(&pair.1, &pretoken.alphabet_list[i+1]) == 0 {
//             // replace 
//             let mut nextchar= pretoken.alphabet_list.remove(i+1);
//             pretoken.alphabet_list[i].append(&mut nextchar);
//         }
//     } 
//     // okay now get new pairs
//     let mut pair_flat:Vec<u8> = pair.0.clone();
//     pair_flat.append(&mut pair.1.clone());

//     let i:usize = 0;
//     while i < pretoken.alphabet_list.len() {
//         let current_a = bigger_byte_vec(&pair_flat, &pretoken.alphabet_list[i]);
//         if current_a != 0 {
//             continue;
//         }
//         if i > 0 {
//             let new_pair = (pretoken.alphabet_list[i-1].clone(), pretoken.alphabet_list[i].clone());
//             new_pairs.push(new_pair);
//         }
//         if i < (pretoken.alphabet_list.len()-1) {
//             let new_pair = (pretoken.alphabet_list[i].clone(), pretoken.alphabet_list[i+1].clone());
//             new_pairs.push(new_pair);
//         }    
//     } 
//     return new_pairs;
// }


// def update_alphabet_list(pretoken:Pretoken, pair:tuple[bytes]) -> list[int]:
//     # returns locations of new pretoken pairs
//     # updates the alphabet list for a prektoken
//     locs = []
//     ind = 1
//     while ind < len(pretoken.alphabet_list):
//         current_pair = (pretoken.alphabet_list[ind-1], pretoken.alphabet_list[ind])
//         if pair == current_pair:
//             pretoken.alphabet_list.pop(ind-1)
//             pretoken.alphabet_list.pop(ind-1)
//             pretoken.alphabet_list.insert(ind-1, flatten(pair))
//             locs.append(ind-1)
//         ind += 1
//     return locs


// def get_new_pairs(pretoken:Pretoken, inds:list[int]):
//     if len(pretoken.alphabet_list) <= 1:
//         return []
//     new_pairs = []
//     for ind in inds:
//         if ind > 0:
//             if pretoken.alphabet_list[ind-1] != pretoken.alphabet_list[ind]:
//                 # backwards pair
//                 new_pairs.append((pretoken.alphabet_list[ind-1], pretoken.alphabet_list[ind]))
//         if ind < (len(pretoken.alphabet_list) - 1):
//             # forwards pair
//             new_pairs.append((pretoken.alphabet_list[ind], pretoken.alphabet_list[ind+1]))
//     return new_pairs

// fn changed_alphabet_pairs<'a>(alphabet_pair_hash:&mut HashMap<(Vec<u8>,Vec<u8>), AlphabetPair>,old_pairs:&Vec<(Vec<u8>,Vec<u8>)>) -> Vec<AlphabetPair>{
//     // get (unique) old pairs from alphabet pair hash, from the tuples
//     let mut changed_aps = Vec::new();
//     for pair in old_pairs {
//         let ap = alphabet_pair_hash.remove(&pair);
//         match ap {
//             None => continue,
//             Some(ap) => changed_aps.push(ap),
//         };
//     }
//     return changed_aps;
// }


// fn make_sorted_alphabet_pair_list<'a>(alphabet_pair_hash:&'a HashMap<(Vec<u8>,Vec<u8>), AlphabetPair>) -> Vec<&AlphabetPair>{
//     let mut alphabet_pair_sort = Vec::new();
//     for ap in (&alphabet_pair_hash).values() {
//         let (_, ind) = get_alphabet_pair_loc(ap, &alphabet_pair_sort);
//         alphabet_pair_sort.insert(ind, ap);
//     }
//     return alphabet_pair_sort;
// }

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

fn make_alphabet_pair_tree(initial_counts:HashMap<AlphabetPair, AlphabetPairInfo>) -> BTreeMap<AlphabetPairKey, Vec<usize>>{
    let mut ap_tree:BTreeMap<AlphabetPairKey, Vec<usize>> = BTreeMap::new();
    for(k, v) in initial_counts {
        let new_key = AlphabetPairKey{count:v.count, pair:k};
        ap_tree.insert(new_key, v.pretoken_ids);
    }
    return ap_tree;
}


fn train(num_merges:usize) -> Option<Vec<(Vec<u8>,Vec<u8>)>> {
    // for now just returns merges
    println!("Hello, world!");
    let merges:Vec<(Vec<u8>,Vec<u8>)> = Vec::new();

    // get list of pretokens
    let pretoken_list = get_pretoken_list();
    // print some summary stats
    let num_pretokens = pretoken_list.len();
    println!("total number of pretokens: {num_pretokens}");

    // get initial counts as a hash, then sort using a BTreeMap
    let initial_pair_counts = count_initial_pairs(&pretoken_list);
    // insert into BTreeMap
    let ap_tree = make_alphabet_pair_tree(initial_pair_counts);

    // borrow last one read only 
    let (max_ap, max_ap_pretoken_list) = ap_tree.last_key_value()?;
    let pair = &max_ap.pair.pair;
    let count = &max_ap.count;
    println!("max_ap {pair:?} max_count {count}");
    

    // while merges.len() < num_merges {
    //     // retrieve max pair, in the worst way ever
    //     let mut maxpair:Option<&AlphabetPair> = None;
    //     for value in alphabet_pair_hash.values(){
    //         maxpair = match maxpair {
    //             None => Some(value),
    //             Some(pair) => {
    //                 if greater_pair(&pair, &value) == 1 {
    //                     Some(value)
    //                 }else{
    //                     Some(pair)
    //                 }
    //             }
    //         };
    //     }
    //     let max_ap = match maxpair {
    //         Some(pair) => pair,
    //         None => return merges
    //     };

    //     let mut pairs_to_change:HashMap<(Vec<u8>,Vec<u8>), AlphabetPair> = HashMap::new();
    //     // loop over pretokens, iterable bc we want to update the alphabet lists
    //     for pid in max_ap.pretoken_list.iter() {
    //         let mut pretoken = match pretoken_list.get_mut(*pid){
    //             None => continue,
    //             Some(pt) => pt,
    //         };
    //         let inds = get_pair_indices(&pretoken, &max_ap.pair);
    //         // get old pairs as tuples
    //         let old_pairs = get_old_pairs(&pretoken,&inds);

    //         //changed alphabet pairs here
    //         let changed_aps = changed_alphabet_pairs(&mut alphabet_pair_hash, &old_pairs);

    //         // remove old pairs from the sorted list
    //         for ap in changed_aps {
    //             // add to pair to change hash
    //             let key = ap.pair.clone();
    //             pairs_to_change.insert(key, ap);
    //         }
    //         // update pairs to change counts with the old_pair tuples
    //         for pair in old_pairs {
    //             let ap = match pairs_to_change.get_mut(&pair){
    //                 None => continue,
    //                 Some(ap) => ap,
    //             };
    //             ap.count -= pretoken.count;
    //         }
    //         let new_pairs:Vec<(Vec<u8>,Vec<u8>)> = get_new_pairs(&mut pretoken, &max_ap.pair);
    //         // add new pairs to pairs to change hash
    //         for new_pair in new_pairs {
    //             if !pairs_to_change.contains_key(&new_pair) {
    //                 pairs_to_change.insert(new_pair.clone(), AlphabetPair { pair: new_pair.clone(), count: 0, pretoken_list: Vec::new() });
    //             }
    //             let ap = match pairs_to_change.get_mut(&new_pair) {
    //                 None => continue,
    //                 Some(ap) => ap,
    //             };
    //             ap.count += pretoken.count;
    //             ap.pretoken_list.push(*pid);
    //         }

    //     }
    //     // (re)insert each of pairs with count > 0 change into the alphabet_pair_has
    //     // (re)insert each of the pairs with count > 0 into the sorted alphabet list


    // }

    return Some(merges);
}



fn main() {
    train(100);
    return;
}




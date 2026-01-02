
// two algorithms to implement
// naive
// for merge 1, iterate through pretoken, check if this merge happens, if it does, merge, if it doesn't, don't
// ...
// ~O(w*v*p) where w is the size of the pretoken and v is the size of the vocab and p is the number of pretokens

// second
// make hash of all merges, store order in which they occurred
// iterate through possible merges (I think O(w^3)) and then maintain a sorted 
// whatever O(w^3 log w) so we know the order in which to merge them
// linear scan over these, performing merges, and then we're all done!
// ~O(v) + O(w^3plogw) or something like that idk
// 
// I lied, I think we can improve this if we use the vocab words
// (c, at) and (ca, t) are not both valid merges? 

// consider two strings that have the same subsequence [a1..ak]
// first thought is any internal merges (aj, aj+1) or something, must take place in both
// base case singleton merges, assuming a1/ak haven't been merged outside the substring
// if we (aj,aj+1) is a valid merge, it has to be valid in both
// now say we have a bigger merge of (an..am)(am+1..ai) all internally
// by inductive hypothesis, each of (an..am) and (am+1..ai) must have occured in both, so new merge is valid in both
// either way this makes it O(w^2*p*log w) + O(v) so much better

use std::collections::HashMap;
use std::collections::HashSet;

// we'll use this in the non-naive one
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct VocabDetail{
    vocab_num:u64,
    word:Vec<u8>,
    in_pretoken:Vec<usize>,
}

fn read_in_vocab(file_path:&str) -> Option<Vec<Vec<u8>>>{
    //return vec of words, merge order is assumed to be same as file order
    return None;
}

fn make_vocab_hash(vocab:Vec<Vec<u8>>) -> Option<HashMap<Vec<u8>, u64>>{
    // makes vocab into a hash where the merge order is stored
    return None;
}

fn lookup_and_insert_sorted(subword:&[u8], vocab_hash:&HashMap<Vec<u8>, u64>, vocab_in_subword:&mut Vec<VocabDetail>) -> Option<Vec<VocabDetail>>{
    return None;
}

fn merge_if_valid(pretoken:&mut Vec<u8>, vocab_in_subword:&Vec<VocabDetail>) {
    return ();

}




// don't worry about slicing these..
// fn cats(k:usize) {
//     let x = vec![1,2,3,4];
//     let whelp = dogs(x);    
//     let z = vec![1,2,3,4];
//     match whelp.contains(&z[..z.len()]) {
//         true => println!("yup z is in there!"),
//         false => println!("nope, z is not there")
//     };
// }

// fn dogs(x:Vec<i32>) -> HashSet<Vec<i32>>{
//     let mut myhash:HashSet<Vec<i32>> = HashSet::new();
//     myhash.insert(x);
//     return myhash;
// }



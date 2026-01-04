
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
    vocab_num:u32,
    word:Vec<u8>,
}

struct EncodedPair{
    vocab_id:u32,
    length:usize,
}



const INITAL_ALPHABET_SIZE:i32 = 257;

fn read_in_vocab(file_path:&str) -> Option<Vec<Vec<u8>>>{
    //return vec of words, merge order is assumed to be same as file order
    return None;
}

fn make_vocab_hash(vocab:Vec<Vec<u8>>) -> Option<HashMap<Vec<u8>, u32>>{
    // makes vocab into a hash where the merge order is stored
    return None;
}

fn lookup_and_insert_sorted(subword:&[u8], vocab_hash:&HashMap<Vec<u8>, u32>, vocab_in_subword:&mut Vec<VocabDetail>){
    let vocab_num = match vocab_hash.get(subword) {
        None => return,
        Some(num) => num
    };
    let word:Vec<u8> = subword.iter().map(|b| *b).collect();
    let detail:VocabDetail = VocabDetail { vocab_num: *vocab_num, word: word};
    match vocab_in_subword.binary_search(&detail) {
        Ok(_) => return, //already in the vocab vec
        Err(idx) => vocab_in_subword.insert(idx, detail)
    };
    return;
}

fn valid_merge(byte_loc:usize, vocab_len:usize, encoded:&Vec<EncodedPair>) -> bool{
    let mut token_start:usize = 0;
    for en in encoded {
        let token_end = token_start + en.length;
        // can't start between start/end
        if token_start < byte_loc && byte_loc < token_end {
            return false;
        }
        // can't end between start/end
        if token_start < byte_loc+vocab_len && byte_loc+vocab_len < token_end {
            return false;
        }
        // short circuit
        if byte_loc+vocab_len < token_start {
            return true;
        }
        token_start = token_end;
    }
    return true;
}
#[test]
fn test_valid_merge(){
    // going to test on c at s 
    let mut encoded:Vec<EncodedPair> = Vec::new();
    encoded.push(EncodedPair { vocab_id:0, length: 1 });
    encoded.push(EncodedPair { vocab_id:1, length: 2 });
    encoded.push(EncodedPair { vocab_id:2, length: 1 });
    
    assert_eq!(valid_merge(0, 1, &encoded), true);
    assert_eq!(valid_merge(0, 3, &encoded), true);
    assert_eq!(valid_merge(0, 4, &encoded), true);
    assert_eq!(valid_merge(0, 2, &encoded), false);
}





// fn merge_if_valid(pretoken:Vec<u32>, vocab_detail:VocabDetail) {
//     // loop through, check 
//     return ();
// }


// fn encode(text:String, vocab_hash:&HashMap<Vec<u8>, u32>) -> Option<Vec<u32>> {
//     let text_bytes = text.as_bytes();
//     let mut vocab_in_subword:Vec<VocabDetail> = Vec::new(); 
//     for subword_start in 0..text_bytes.len() {
//         for subword_end in subword_start+1..text_bytes.len()+1 {
//             let subword = &text_bytes[subword_start..subword_end];
//             lookup_and_insert_sorted(&subword, &vocab_hash, &mut vocab_in_subword);
//         }
//     }
//     // initialize our encoded vector
//     let mut encoded:Vec<u32> = Vec::new();
//     for tb in text_bytes {
//         encoded.push(*tb as u32);
//     }
//     // merge if valid
//     merge_if_valid(pretoken, &vocab_in_subword);


//     return None;
// }

// merge step kind of complicated
// think we want to keep the pretoken as a u8 vec
// loop through bytes for vocab match
    // when we have a match, check it isn't fully contained in another merge
    // 


// look for a match 


// another two vectors






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



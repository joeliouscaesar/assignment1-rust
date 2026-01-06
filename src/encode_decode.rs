
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
use std::hash::Hash;

// we'll use this in the non-naive one
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct VocabDetail{
    vocab_num:u32,
    word:Vec<u8>,
    loc:usize,
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

fn lookup_and_insert_sorted(subword:&[u8], loc:usize, vocab_hash:&HashMap<Vec<u8>, u32>, vocab_in_subword:&mut Vec<VocabDetail>){
    let vocab_num = match vocab_hash.get(subword) {
        None => return,
        Some(num) => num
    };
    let word:Vec<u8> = subword.iter().map(|b| *b).collect();
    let detail:VocabDetail = VocabDetail { vocab_num: *vocab_num, word: word, loc:loc};
    match vocab_in_subword.binary_search(&detail) {
        Ok(_) => return, //already in the vocab vec
        Err(idx) => vocab_in_subword.insert(idx, detail)
    };
    return;
}

fn valid_merge(byte_loc:usize, vocab_len:usize, encoded:&Vec<EncodedPair>) -> Option<(usize,usize)>{
    let mut token_start:usize = 0;
    let mut token_num:usize = 0;
    let mut count:usize = 0;
    for (i, en) in encoded.iter().enumerate() {
        let token_end = token_start + en.length;
        // can't start between start/end
        if token_start < byte_loc && byte_loc < token_end {
            return None;
        }
        // can't end between start/end
        else if token_start < byte_loc+vocab_len && byte_loc+vocab_len < token_end {
            return None;
        }        
        // check if we're at a match start
        if token_start == byte_loc {
            token_num = i;
            count += 1;
        }else if count > 0 {
            count += 1;
        }
        if byte_loc+vocab_len == token_end {
            return Some((token_num, count));
        }
        token_start = token_end;
    }
    return None;
}
#[test]
fn test_valid_merge(){
    // going to test on c at s 
    let mut encoded:Vec<EncodedPair> = Vec::new();
    encoded.push(EncodedPair { vocab_id:0, length: 1 });
    encoded.push(EncodedPair { vocab_id:1, length: 2 });
    encoded.push(EncodedPair { vocab_id:2, length: 1 });
    
    assert_eq!(valid_merge(0, 1, &encoded), Some((0,1)));
    assert_eq!(valid_merge(0, 3, &encoded), Some((0,2)));
    assert_eq!(valid_merge(0, 4, &encoded), Some((0,3)));
    assert_eq!(valid_merge(0, 2, &encoded), None);
    assert_eq!(valid_merge(1, 1, &encoded), None);
    assert_eq!(valid_merge(2, 2, &encoded), None);
}



// fn merge_if_valid(pretoken:Vec<u32>, vocab_detail:VocabDetail) {

//     // loop through, check 
//     return ();
// }


fn encode(text:String, vocab_hash:&HashMap<Vec<u8>, u32>) -> Option<Vec<u32>> {
    let text_bytes = text.as_bytes();
    let mut vocab_in_subword:Vec<VocabDetail> = Vec::new(); 
    for subword_start in 0..text_bytes.len() {
        for subword_end in subword_start+1..text_bytes.len()+1 {
            let subword = &text_bytes[subword_start..subword_end];
            lookup_and_insert_sorted(&subword, subword_start,&vocab_hash, &mut vocab_in_subword);
        }
    }
    // initialize our encoded vector
    let mut encoded:Vec<EncodedPair> = Vec::new();
    for tb in text_bytes {
        let ep = EncodedPair{vocab_id:(*tb as u32), length:1};
        encoded.push(ep);
    }
    // do merges
    let final_encoded = vocab_in_subword.iter().fold(
        encoded, do_merges
    );

    let token_ids:Vec<u32> = final_encoded.iter().map(|x| x.vocab_id).collect();

    return Some(token_ids);
}

fn do_merges(mut encoded_pairs:Vec<EncodedPair>, vd:&VocabDetail) -> Vec<EncodedPair>{
    let (pos, mut num) = match valid_merge(vd.loc, vd.word.len(), &encoded_pairs){
        None => return encoded_pairs,
        Some((p,n)) => (p,n)
    };
    while num > 1 {
        encoded_pairs.remove(pos);
        num -= 1;
    }
    encoded_pairs[pos] = EncodedPair { vocab_id: vd.vocab_num, length: vd.word.len() };
    return encoded_pairs;
}

#[test]
fn test_encode() {
    let mut vocab:HashMap<Vec<u8>, u32> = HashMap::new();
    vocab.insert(vec![32], 0); // ' '
    vocab.insert(vec![97], 1); // 'a'
    vocab.insert(vec![99], 2); // 'c'
    vocab.insert(vec![101], 3); // 'e'
    vocab.insert(vec![104], 4); // 'h'
    vocab.insert(vec![116], 5); // 't'
    vocab.insert(vec![116,104], 6); // 'th'
    vocab.insert(vec![32,99], 7); // ' c'
    vocab.insert(vec![32,97],8); // ' a'
    vocab.insert(vec![116,104,101],9); // 'the'
    vocab.insert(vec![32,97, 116],10); // ' at'
    
    // the
    let the = String::from("the");
    let the_answer = encode(the, &vocab);
    assert_ne!(the_answer, None);
    let the_answer = match the_answer{
        None => return,
        Some(x) => x
    };
    let the_key:Vec<u32> = vec![9];
    assert_eq!(the_answer, the_key);
    
    // cat
    let cat = String::from(" cat");
    let cat_answer = encode(cat, &vocab);
    assert_ne!(cat_answer, None);
    let cat_answer = match cat_answer{
        None => return,
        Some(x) => x
    };
    let cat_key:Vec<u32> = vec![7, 1, 5];
    assert_eq!(cat_answer, cat_key);

    // ate
    let ate = String::from(" ate");
    let ate_answer = encode(ate, &vocab);
    assert_ne!(ate_answer, None);
    let ate_answer = match ate_answer{
        None => return,
        Some(x) => x
    };
    let ate_key:Vec<u32> = vec![10, 3];
    assert_eq!(ate_answer, ate_key);

    return ();

}


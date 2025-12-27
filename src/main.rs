use fancy_regex::Regex; use std::ffi::FromVecWithNulError;
// for regex
use std::fs;
use std::collections::HashMap;
use std::vec;

// struct for pretokens
// lifetime specifiers are a little weird, the 'a things
struct Pretoken {
    count:i32,
    alphabet_list:Vec<Vec<u8>>,
}

struct AlphabetPair<'a> {
    a1:&'a Vec<u8>,
    a2:&'a Vec<u8>,
    count:i32,
    pretoken_list:Vec<&'a Pretoken>,
}

fn bigger_byte_vec(a:&Vec<u8>, b:&Vec<u8>) -> i8 {
    // -1 means first bigger, 0 same, 1 second bigger
    let mut i:usize = 0;
    loop {
        let a_byte = a.get(i);
        let b_byte = b.get(i);

        match (a_byte, b_byte) {
            (None, None) => break,
            (None, _) => return 1,
            (_, None) => return -1,
            (Some(a),Some(b)) => {
                if a < b {
                    return 1;
                }else if b < a{
                    return -1;
                }
            }
        }
        i = i + 1;
    }
    return 0;
}

#[test]
fn check_byte_comp() {
    let empty:Vec<u8> = Vec::new();
    let a:Vec<u8> = "a".as_bytes().to_vec();
    let ab:Vec<u8> = "ab".as_bytes().to_vec();
    assert_eq!(bigger_byte_vec(&empty, &a), 1);
    assert_eq!(bigger_byte_vec(&ab, &a), -1);
    assert_eq!(bigger_byte_vec(&a, &ab), 1);
    assert_eq!(bigger_byte_vec(&a, &a), 0);
}

fn greater_pair<'a>(a:&'a AlphabetPair, b:&'a AlphabetPair) -> i8{
    if a.count > b.count {
        return -1;
    }else if b.count > a.count {
        return 1;
    }
    // in final case we want to check the byte tuples
    let first_comp = bigger_byte_vec(a.a1, b.a1);
    if first_comp == -1 {
        return -1;
    }else if first_comp == 1 {
        return 1;
    }
    // check second
    let second_comp = bigger_byte_vec(a.a2, b.a2);
    if second_comp == 1 {
        return 1;
    }else {
        return -1;
    }
}
#[test]
fn check_alphabet_pair_comp() {
    let pair1 = AlphabetPair {
        a1:&"ab".as_bytes().to_vec(),
        a2:&"cd".as_bytes().to_vec(),
        count:1,
        pretoken_list:Vec::new(),
    };
    let pair2 = AlphabetPair {
        a1:&"ab".as_bytes().to_vec(),
        a2:&"cd".as_bytes().to_vec(),
        count:10,
        pretoken_list:Vec::new(),
    };
    let pair3 = AlphabetPair {
        a1:&"xy".as_bytes().to_vec(),
        a2:&"cd".as_bytes().to_vec(),
        count:5,
        pretoken_list:Vec::new(),
    };
    let pair4 = AlphabetPair {
        a1:&"xy".as_bytes().to_vec(),
        a2:&"cd".as_bytes().to_vec(),
        count:1,
        pretoken_list:Vec::new(),
    };

    assert_eq!(greater_pair(&pair1, &pair2), 1);
    assert_eq!(greater_pair(&pair2, &pair3), -1);
    assert_eq!(greater_pair(&pair1, &pair3), 1);
    assert_eq!(greater_pair(&pair1, &pair4), 1);
    assert_eq!(greater_pair(&pair3, &pair4), -1);
}


fn train() {
  println!("Hello, world!");
    // compile regex
    let re = Regex::new(r"'(?:[sdmt]|ll|ve|re)| ?\p{L}+| ?\p{N}+| ?[^\s\p{L}\p{N}]+|\s+(?!\S)|\s+");
    // pattern match on if error
    let re = match re {
        Ok(x) => x,
        Err(e) => {
            println!("regex issue {e}");
            return
        }
    };    
    // read in file
    let contents = fs::read_to_string("data/corpus.en");
    let contents = match contents {
        Ok(s) => s,
        Err(e) => {
            println!("issue reading file: {e}");
            return
        }
    };

    // add values to a hash map
    let mut pretoken_hash:HashMap<String, i32> = HashMap::new();
    let re_iter = re.find_iter(&contents);
    for re_match in re_iter {
        let group = match re_match {
            Ok(g) => g,
            Err(e) => {
                println!("issue with group {e}");
                continue;
            }
        }.as_str().to_string();
        let current_count = match pretoken_hash.get(&group) {
            Some(i) => *i,
            None => 0,
        };
        let new_count = current_count + 1;
        pretoken_hash.insert(group, new_count);
    }
    // print some summary stats
    let num_pretokens = pretoken_hash.len();
    println!("total number of pretokens: {num_pretokens}");
    
    // make pretoken list 
    let mut pretokens:Vec<Pretoken> = Vec::new();
    for (s, n) in pretoken_hash.iter() {
        let mut al:Vec<Vec<u8>> = Vec::new();
        for b in s.bytes(){
            let a = vec![b];
            al.push(a);
        }
        // create pretokens 
        let pretoken = Pretoken{
            count: *n,
            alphabet_list: al,
        };
        pretokens.push(pretoken);
    }

    // alphabet pair hash
    // TODO
    
    


}


fn main() {



    // train();


}




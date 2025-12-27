use fancy_regex::Regex;
// for regex
use std::fs;
use std::collections::HashMap;
use std::vec;

// struct for pretokens
// lifetime specifiers are a little weird, the 'a things
struct Pretoken {
    count:usize,
    alphabet_list:Vec<Vec<u8>>,
}

struct AlphabetPair<'a> {
    pair:(Vec<u8>,Vec<u8>),
    count:usize,
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
    let first_comp = bigger_byte_vec(&a.pair.0, &b.pair.0);
    if first_comp != 0 {
        return first_comp;
    }
    // check second
    let second_comp = bigger_byte_vec(&a.pair.1, &b.pair.1);
    return second_comp;
}
#[test]
fn check_alphabet_pair_comp() {
    let pair1 = AlphabetPair {
        pair:("ab".as_bytes().to_vec(),"cd".as_bytes().to_vec()),
        count:1,
        pretoken_list:Vec::new(),
    };
    let pair2 = AlphabetPair {
        pair:("ab".as_bytes().to_vec(),"cd".as_bytes().to_vec()),
        count:10,
        pretoken_list:Vec::new(),
    };
    let pair3 = AlphabetPair {
        pair:("xy".as_bytes().to_vec(), "cd".as_bytes().to_vec()),
        count:5,
        pretoken_list:Vec::new(),
    };
    let pair4 = AlphabetPair {
        pair:("xy".as_bytes().to_vec(), "cd".as_bytes().to_vec()),
        count:1,
        pretoken_list:Vec::new(),
    };

    assert_eq!(greater_pair(&pair1, &pair2), 1);
    assert_eq!(greater_pair(&pair2, &pair3), -1);
    assert_eq!(greater_pair(&pair1, &pair3), 1);
    assert_eq!(greater_pair(&pair1, &pair4), 1);
    assert_eq!(greater_pair(&pair3, &pair4), -1);
    assert_eq!(greater_pair(&pair3, &pair3), 0);
}


fn get_alphabet_pair_loc(ap:&AlphabetPair, ap_list:&Vec<&AlphabetPair>) -> (bool,usize) {
    // returns location pretoken should be inserted and 
    // true if the pretoken is in the list
    if ap_list.len() == 0 {
        return (false,0);
    }
    // want strict low/high
    let mut low:usize = 0;
    let mut high = ap_list.len();
    let low_comp = greater_pair(&ap, ap_list[low]);
    match low_comp {
        1 => return (false,0),
        0 => return (true, 0),
        _ => {}
    };
    let high_comp = greater_pair(&ap, ap_list[high-1]);
    match high_comp {
        -1 => return (false,high),
        0 => return (true, high-1),
        _ => {}
    };
    loop {
        let mid = (high + low)/2;
        // println!("low mid high {low} {mid} {high}");
        let comp_value = greater_pair(&ap, ap_list[mid]);
        if comp_value == 1 {
            high = mid;
        }else if comp_value == -1 {
            low = mid;
        }else {
            return (true, mid);
        }
        if high <= (low + 1){
            return (false,high);
        }
    }
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
    let mut pretoken_hash:HashMap<String, usize> = HashMap::new();
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
    let mut alphabet_pair_hash:HashMap<(Vec<u8>,Vec<u8>),AlphabetPair> = HashMap::new();
    for pretoken in pretokens.iter() {
        // add pairs to the alphabet pair hash 
        for i in 1..pretoken.alphabet_list.len() {
            let a1 = pretoken.alphabet_list[i-1].clone(); 
            let a2 = pretoken.alphabet_list[i].clone();
            let pair = (a1,a2);
            // check if already a key 
            let ap = alphabet_pair_hash.remove(&pair);
            let mut ap = match ap {
                // new pair
                None => {
                    AlphabetPair {
                        pair:pair.clone(),
                        count: 0,
                        pretoken_list:Vec::new(),
                    }
                },
                Some(ap) => ap,
            };
            // add pretoken count/pretoken to list
            ap.count += pretoken.count;
            ap.pretoken_list.push(&pretoken);

            alphabet_pair_hash.insert(pair, ap);

        }        
    }
    // vector of pretokens, sorted by count/alphabetically
    let mut alphabet_pair_sort:Vec<&AlphabetPair> = Vec::new();
    for ap in alphabet_pair_hash.values() {
        let (_, ind) = get_alphabet_pair_loc(ap, &alphabet_pair_sort);
        alphabet_pair_sort.insert(ind, ap);
    }

    let max_ap = alphabet_pair_sort.pop();
    match max_ap {
        None => {
            println!("something wrong with alphabet pair sort");
        },
        Some(ap) => {
            let pair = &ap.pair;
            println!("max pair is {pair:?}");
        },
    }

    return {};
}





fn main() {
    train();

}




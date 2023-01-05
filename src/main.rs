use std::{collections::HashMap, fmt::Display, fs::File, path::PathBuf};

use itertools::Itertools;
use memmap2::MmapOptions;
use rand::{distributions::WeightedIndex, prelude::Distribution, thread_rng, Rng};
use tinyvec::TinyVec;
use unicode_segmentation::UnicodeSegmentation;


fn main() {
    let INPUT_LENGTH = 2; //1 or 2
    let OUTPUT_LENGTH = 2000; //1 or 2
    let STARTING_POINT = "goes here"; //must exist in the training data



    let m = MarkovBuilder::new()

        .input_length(INPUT_LENGTH)
        .data_path("data")
        .train();
    println!("{}",  m.generate(STARTING_POINT, OUTPUT_LENGTH));
}

struct Markov {
    map: HashMap<Vec<String>, HashMap<String, usize>>,
}

impl Markov {
    fn generate(&self, input: &str, length: u64) -> String {
        let length = {
            if length < 2 {
                2
            } else {
                length
            }
        };

        let tup_size = self.map.keys().next().unwrap().len();
        let mut rng = thread_rng();
        let mut s = Vec::new();
        
            for w in input.split_whitespace().take(tup_size) {
                s.push(w);
            }
       
        for _ in 0..length - tup_size as u64 {
            let last = s.iter().rev().take(tup_size).rev().map(|x|x.to_string()).collect::<Vec<_>>();
            //dbg!(last.clone());
            let next = self.map.get(&last).unwrap();
            let options = next.iter().collect_vec();
            let dist = WeightedIndex::new(options.iter().map(|x| x.1)).unwrap();
            s.push(options[dist.sample(&mut rng)].0);
        }
        let mut string = String::new();
        for word in s {
            string.push_str(word);
            string.push(' ');
        }
        string.pop();

        string
    }
}

struct MarkovBuilder {
    input_length: usize,
    data_path: PathBuf,
}

impl MarkovBuilder {
    fn new() -> Self {
        Self {
            input_length: 1,
            data_path: PathBuf::from(""),
        }
    }
    fn input_length(self, l: usize) -> Self {
        Self {
            input_length: l,
            data_path: self.data_path,
        }
    }
    fn data_path<T>(self, p: T) -> Self
    where
        std::path::PathBuf: std::convert::From<T>,
    {
        let path = PathBuf::from(p);
        if path.exists() && path.is_file() {
            Self {
                input_length: self.input_length,
                data_path: path,
            }
        } else {
            panic!("The file {} does not exist!", path.to_str().unwrap_or(""));
        }
    }

    fn train(self) -> Markov {
        let file = File::open(self.data_path).expect("Could not open file");
        let mmap = unsafe {
            MmapOptions::new()
                .populate()
                .map(&file)
                .expect("Failed to mmap file")
        };

        let mut markov = Markov {
            map: HashMap::new(),
        };

        let words = unsafe { std::str::from_utf8_unchecked(&mmap) }
            .split_whitespace()
            .collect::<Vec<_>>();

        
        match self.input_length {
            1 => {
                for (prev1, next) in words.iter().circular_tuple_windows() {
                    let piece = markov.map.get_mut(&vec![prev1.to_string()]);
                    if let Some(piece) = piece {
                        if let Some(prob) = piece.get_mut(*next) {
                            *prob += 1;
                        } else {
                            piece.insert(next.to_owned().to_owned(), 1);
                        }
                    } else {
                        markov.map.insert(vec![prev1.to_owned().to_string()], {
                            let mut h = HashMap::new();
                            h.insert(next.to_owned().to_owned(), 1);
                            h
                        });
                    }
                }        
            }
            2 => {
                for (prev1, prev2, next) in words.iter().circular_tuple_windows() {
                    let piece = markov.map.get_mut(&vec![prev1.to_string(), prev2.to_string()]);
                    if let Some(piece) = piece {
                        if let Some(prob) = piece.get_mut(*next) {
                            *prob += 1;
                        } else {
                            piece.insert(next.to_owned().to_owned(), 1);
                        }
                    } else {
                        markov.map.insert(vec![prev1.to_owned().to_string(), prev2.to_owned().to_string()], {
                            let mut h = HashMap::new();
                            h.insert(next.to_owned().to_owned(), 1);
                            h
                        });
                    }
                }
            }
            _ => panic!("Unsupported input length")
        }

        markov
    }
}

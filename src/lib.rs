#![feature(test)]
#![feature(portable_simd)]
#[macro_use]
extern crate lazy_static;

use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::simd::f32x16;

struct Index {
    index: Vec<Vec<f32>>,
}

struct Score {
    id: usize,
    score: f32,
}

impl PartialEq<Self> for Score {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd<Self> for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.score.partial_cmp(&self.score)
    }
}

impl Eq for Score {}

impl Ord for Score {
    // Reverse the order so that the highest score is first.
    fn cmp(&self, other: &Self) -> Ordering {
        other.score.total_cmp(&self.score)
    }
}

// This currently only supports vectors that have a length with a
// multiple of 16.
impl Index {
    fn new() -> Index {
        Index {
            index: vec![],
        }
    }
    fn add(&mut self, data: Vec<f32>) {
        assert_eq!(data.len() % 16, 0);
        // Precompute the unit vector and store it
        let unit_factor = square(&data).sqrt();
        self.index.push(data.into_iter().map(|x| x / unit_factor).collect());
    }
    // Use cosine similarity to search index
    fn search(&self, query: &Vec<f32>, topk: usize) -> Vec<(usize, f32)> {
        assert!(topk > 0);
        let mut result: BinaryHeap<Score> = BinaryHeap::new();
        // Precompute the unit coefficient for the search vector.
        let query_unit = 1.0 / square(&query).sqrt();
        for (i, v) in self.index.iter().enumerate() {
            let score = cosine_similarity(query, v, query_unit);
            if result.len() == topk {
                if let Some(lowest) = result.peek() {
                    if score < lowest.score {
                        continue;
                    }
                }
                result.pop();
            }
            result.push(Score { id: i, score });
        }
        result
            .into_sorted_vec()
            .into_iter()
            .map(|s| (s.id, s.score))
            .collect()
    }
    fn len(&self) -> usize {
        self.index.len()
    }
}

// As this isn't used in the main loop there is no reason to optimize it with SIMD.
fn square(a: &Vec<f32>) -> f32 {
    let mut result = 0.0;
    for i in 0..a.len() {
        result += a[i] * a[i];
    }
    result
}

// Leverages SIMD on the CPU to calculate the cosine similarity between two vectors.
// I have found that on some architectures (amd64) LLVM will automatically vectorize the naive
// implementation this but on others (M1) it will not.
fn cosine_similarity(a: &Vec<f32>, b: &Vec<f32>, a_unit: f32) -> f32 {
    let lanes = 16;
    let partitions = a.len() / lanes;
    // Use simd to calculate cosine similarity
    let mut dot: f32 = 0.0;
    // Loop over partitions
    for i in 0..partitions {
        let i1 = i * lanes;
        let i2 = (i + 1) * lanes;
        let a_simd = f32x16::from_slice(&a.as_slice()[i1..i2]);
        let b_simd = f32x16::from_slice(&b.as_slice()[i1..i2]);
        dot += (a_simd * b_simd).reduce_sum();
    }
    dot * a_unit
}

#[cfg(test)]
mod tests {
    use crate::Index;

    extern crate test;

    use rand::distributions::Standard;
    use rand::Rng;
    use test::Bencher;

    /// The following test function is necessary for the header generation.
    #[safer_ffi::cfg_headers]
    #[test]
    fn generate_headers() -> std::io::Result<()> {
        safer_ffi::headers::builder()
            .to_file("include/bfes.h")?
            .generate()
    }

    #[test]
    fn search_test() {
        let (index, v) = prepare();
        let result = index.search(&v, 10);
        assert_eq!(result.len(), 10);
        let mut last = f32::MAX;
        for (_i, score) in result.iter().enumerate() {
            assert!(score.1 < last);
            last = score.1;
        }
        println!("{:?}", result);
    }

    #[bench]
    fn bench_cosine_similarity(b: &mut Bencher) {
        let (index, v) = prepare();
        // Search the index
        b.iter(|| {
            index.search(&v, 10);
        });
    }

    fn prepare() -> (Index, Vec<f32>) {
        // Thread rng
        let rng = rand::thread_rng();
        // Make a new index
        let mut index = Index::new();
        // Generate 100000 random 512 dimension vectors
        for _ in 0..100000 {
            let v: Vec<f32> = rng.clone().sample_iter(Standard).take(512).collect();
            index.add(v);
        }
        // Generate a random 512 dimension vector
        let v: Vec<f32> = rng.sample_iter(Standard).take(512).collect();
        (index, v)
    }
}

// Here is the C API for Index. It works very well from Swift if you want to use it on
// iOS or Mac.
use ::safer_ffi::prelude::*;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;
use std::slice;
use std::sync::Mutex;

lazy_static! {
    static ref INDEX_MANAGER: Mutex<HashMap<String, Box<Index>>> = Mutex::new(HashMap::new());
}

fn cchar_to_string(name: *const c_char) -> String {
    let idx_name;
    unsafe {
        idx_name = CStr::from_ptr(name).to_string_lossy().into_owned();
    }
    idx_name
}

#[ffi_export]
pub extern "C" fn bfes_new_index(name: *const c_char) {
    let idx_name = cchar_to_string(name);

    INDEX_MANAGER
        .lock()
        .unwrap()
        .insert(idx_name, Box::new(Index::new()));
}

#[ffi_export]
pub extern "C" fn bfes_add(name: *const c_char, features: *const f32, dimension: usize) -> usize {
    let idx_name: String = cchar_to_string(name);
    let data_slice = unsafe { slice::from_raw_parts(features as *const f32, dimension) };
    let buf = data_slice.to_vec();

    match &mut INDEX_MANAGER.lock().unwrap().get_mut(&idx_name) {
        Some(index) => {
            index.add(Vec::from(buf));
            index.len()
        }
        None => 0,
    }
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SearchResult {
    index: usize,
    score: f32,
}

#[ffi_export]
pub extern "C" fn bfes_search(
    name: *const c_char,
    k: usize,
    features: *const f32,
    dimension: usize,
) -> repr_c::Vec<SearchResult> {
    let idx_name: String = cchar_to_string(name);
    let data_slice = unsafe { slice::from_raw_parts(features, dimension) };
    let buf = data_slice.to_vec();
    let topk = k;

    let mut result: Vec<SearchResult> = vec![];
    if let Some(index) = INDEX_MANAGER.lock().unwrap().get(&idx_name) {
        index.search(&Vec::from(buf), topk).iter().for_each(|x| {
            result.push(SearchResult {
                index: x.0,
                score: x.1,
            })
        })
    }
    result.into()
}

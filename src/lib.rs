#![feature(test)]

struct Index {
    index: Vec<Vec<f32>>,
}

impl Index {
    fn new() -> Index {
        Index {
            index: vec![]
        }
    }
    fn add(&mut self, data: Vec<f32>) {
        self.index.push(data);
    }
    // Use cosine similarity to search index
    fn search(&self, data: &Vec<f32>, topk: usize) -> Vec<(usize, f32)> {
        let mut result: Vec<(usize, f32)> = vec![];
        for (i, v) in self.index.iter().enumerate() {
            let score = cosine_similarity(data, v);
            result.push((i, score));
        }
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        result.truncate(topk);
        result
    }
}

fn cosine_similarity(a: &Vec<f32>, b: &Vec<f32>) -> f32 {
    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }
    dot / (norm_a * norm_b).sqrt()
}


extern crate test;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    use test::Bencher;

    #[bench]
    fn bench_cosine_similarity(b: &mut Bencher) {
        // Make a new index
        let mut index = super::Index::new();
        // Generate 100000 random 512 dimension vectors
        for _ in 0..100000 {
            let mut v: Vec<f32> = vec![];
            for _ in 0..512 {
                v.push(rand::random::<f32>());
            }
            index.add(v);
        }
        // Generate a random 512 dimension vector
        let mut v: Vec<f32> = vec![];
        for _ in 0..512 {
            v.push(rand::random::<f32>());
        }
        // Search the index
        b.iter(|| {
            index.search(&v, 10);
        });
    }
}

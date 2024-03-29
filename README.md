# bfes

## Brute force embedding search

Given a set of embeddings, and a query embedding, this algorithm searches for the
nearest embeddings for each query. As it is a brute force search, you don't
need to regenerate the index and can add new embeddings to the index at any
time. The algorithm is O(n) in time and space. It compares embeddings using
cosine similarity.

## Performance

Time to brute force search 100,000 512-dimensional embeddings:

Windows 11, AMD Ryzen 9 5950x @ 3.4 GHz

test tests::bench_cosine_similarity ... bench:  20,018,120 ns/iter (+/- 2,042,521)

Mac OS X, M1 Max MacbookPro18,4

test tests::bench_cosine_similarity ... bench:  11,302,216 ns/iter (+/- 185,505)

Mac OS X, M1 Macmini9,1

test tests::bench_cosine_similarity ... bench:  9,559,170 ns/iter (+/- 592,620)

Ubuntu 18.04, AWS c6i.large

test tests::bench_cosine_similarity ... bench:  25,452,715 ns/iter (+/- 848,001)

Ubuntu 22.04, AWS c6a.large

test tests::bench_cosine_similarity ... bench:  17,306,132 ns/iter (+/- 78,319)

Ubuntu 22.04, AWS c6g.medium

test tests::bench_cosine_similarity ... bench:  30,262,326 ns/iter (+/- 64,509)

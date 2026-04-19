# bitvec — ETNA Tasks

Total tasks: 24

ETNA tasks are **mutation/property/witness triplets**. Each row below is one runnable task: the command executes the framework-specific adapter against the buggy variant branch and should report a counterexample.

Run against a variant by first checking out its branch (`git checkout etna/<variant>`) or applying its patch on a clean tree (`git apply patches/<variant>.patch`).

## Task Index

| Task | Variant | Framework | Property | Witness(es) | Command |
|------|---------|-----------|----------|-------------|---------|
| 001 | `split_at_mut_rejects_len_c71ea23_1` | proptest | `property_split_at_mut_accepts_len` | `witness_split_at_mut_accepts_len_case_empty_right`, `witness_split_at_mut_accepts_len_case_zero_len` | `cargo run --release --bin etna -- proptest SplitAtMutAcceptsLen` |
| 002 | `split_at_mut_rejects_len_c71ea23_1` | quickcheck | `property_split_at_mut_accepts_len` | `witness_split_at_mut_accepts_len_case_empty_right`, `witness_split_at_mut_accepts_len_case_zero_len` | `cargo run --release --bin etna -- quickcheck SplitAtMutAcceptsLen` |
| 003 | `split_at_mut_rejects_len_c71ea23_1` | crabcheck | `property_split_at_mut_accepts_len` | `witness_split_at_mut_accepts_len_case_empty_right`, `witness_split_at_mut_accepts_len_case_zero_len` | `cargo run --release --bin etna -- crabcheck SplitAtMutAcceptsLen` |
| 004 | `split_at_mut_rejects_len_c71ea23_1` | hegel | `property_split_at_mut_accepts_len` | `witness_split_at_mut_accepts_len_case_empty_right`, `witness_split_at_mut_accepts_len_case_zero_len` | `cargo run --release --bin etna -- hegel SplitAtMutAcceptsLen` |
| 005 | `vec_insert_rejects_end_8e48751_1` | proptest | `property_vec_insert_accepts_end` | `witness_vec_insert_accepts_end_case_push_true`, `witness_vec_insert_accepts_end_case_empty_vec` | `cargo run --release --bin etna -- proptest VecInsertAcceptsEnd` |
| 006 | `vec_insert_rejects_end_8e48751_1` | quickcheck | `property_vec_insert_accepts_end` | `witness_vec_insert_accepts_end_case_push_true`, `witness_vec_insert_accepts_end_case_empty_vec` | `cargo run --release --bin etna -- quickcheck VecInsertAcceptsEnd` |
| 007 | `vec_insert_rejects_end_8e48751_1` | crabcheck | `property_vec_insert_accepts_end` | `witness_vec_insert_accepts_end_case_push_true`, `witness_vec_insert_accepts_end_case_empty_vec` | `cargo run --release --bin etna -- crabcheck VecInsertAcceptsEnd` |
| 008 | `vec_insert_rejects_end_8e48751_1` | hegel | `property_vec_insert_accepts_end` | `witness_vec_insert_accepts_end_case_push_true`, `witness_vec_insert_accepts_end_case_empty_vec` | `cargo run --release --bin etna -- hegel VecInsertAcceptsEnd` |
| 009 | `leading_trailing_homogeneous_b08c4bd_1` | proptest | `property_leading_trailing_fallback` | `witness_leading_trailing_fallback_case_all_ones`, `witness_leading_trailing_fallback_case_all_zeros` | `cargo run --release --bin etna -- proptest LeadingTrailingFallback` |
| 010 | `leading_trailing_homogeneous_b08c4bd_1` | quickcheck | `property_leading_trailing_fallback` | `witness_leading_trailing_fallback_case_all_ones`, `witness_leading_trailing_fallback_case_all_zeros` | `cargo run --release --bin etna -- quickcheck LeadingTrailingFallback` |
| 011 | `leading_trailing_homogeneous_b08c4bd_1` | crabcheck | `property_leading_trailing_fallback` | `witness_leading_trailing_fallback_case_all_ones`, `witness_leading_trailing_fallback_case_all_zeros` | `cargo run --release --bin etna -- crabcheck LeadingTrailingFallback` |
| 012 | `leading_trailing_homogeneous_b08c4bd_1` | hegel | `property_leading_trailing_fallback` | `witness_leading_trailing_fallback_case_all_ones`, `witness_leading_trailing_fallback_case_all_zeros` | `cargo run --release --bin etna -- hegel LeadingTrailingFallback` |
| 013 | `bitvec_partial_cmp_reversed_f3d5b43_1` | proptest | `property_bitvec_partial_cmp_matches` | `witness_bitvec_partial_cmp_matches_case_lt`, `witness_bitvec_partial_cmp_matches_case_gt` | `cargo run --release --bin etna -- proptest BitVecPartialCmpMatches` |
| 014 | `bitvec_partial_cmp_reversed_f3d5b43_1` | quickcheck | `property_bitvec_partial_cmp_matches` | `witness_bitvec_partial_cmp_matches_case_lt`, `witness_bitvec_partial_cmp_matches_case_gt` | `cargo run --release --bin etna -- quickcheck BitVecPartialCmpMatches` |
| 015 | `bitvec_partial_cmp_reversed_f3d5b43_1` | crabcheck | `property_bitvec_partial_cmp_matches` | `witness_bitvec_partial_cmp_matches_case_lt`, `witness_bitvec_partial_cmp_matches_case_gt` | `cargo run --release --bin etna -- crabcheck BitVecPartialCmpMatches` |
| 016 | `bitvec_partial_cmp_reversed_f3d5b43_1` | hegel | `property_bitvec_partial_cmp_matches` | `witness_bitvec_partial_cmp_matches_case_lt`, `witness_bitvec_partial_cmp_matches_case_gt` | `cargo run --release --bin etna -- hegel BitVecPartialCmpMatches` |
| 017 | `clone_from_bitslice_src_bug_935cad8_1` | proptest | `property_clone_from_bitslice_copies_src` | `witness_clone_from_bitslice_copies_src_case_flip_all`, `witness_clone_from_bitslice_copies_src_case_alternating` | `cargo run --release --bin etna -- proptest CloneFromBitsliceCopiesSrc` |
| 018 | `clone_from_bitslice_src_bug_935cad8_1` | quickcheck | `property_clone_from_bitslice_copies_src` | `witness_clone_from_bitslice_copies_src_case_flip_all`, `witness_clone_from_bitslice_copies_src_case_alternating` | `cargo run --release --bin etna -- quickcheck CloneFromBitsliceCopiesSrc` |
| 019 | `clone_from_bitslice_src_bug_935cad8_1` | crabcheck | `property_clone_from_bitslice_copies_src` | `witness_clone_from_bitslice_copies_src_case_flip_all`, `witness_clone_from_bitslice_copies_src_case_alternating` | `cargo run --release --bin etna -- crabcheck CloneFromBitsliceCopiesSrc` |
| 020 | `clone_from_bitslice_src_bug_935cad8_1` | hegel | `property_clone_from_bitslice_copies_src` | `witness_clone_from_bitslice_copies_src_case_flip_all`, `witness_clone_from_bitslice_copies_src_case_alternating` | `cargo run --release --bin etna -- hegel CloneFromBitsliceCopiesSrc` |
| 021 | `octal_fmt_buffer_size_aeef0be_1` | proptest | `property_octal_fmt_no_panic` | `witness_octal_fmt_no_panic_case_one_u64`, `witness_octal_fmt_no_panic_case_three_u64` | `cargo run --release --bin etna -- proptest OctalFmtNoPanic` |
| 022 | `octal_fmt_buffer_size_aeef0be_1` | quickcheck | `property_octal_fmt_no_panic` | `witness_octal_fmt_no_panic_case_one_u64`, `witness_octal_fmt_no_panic_case_three_u64` | `cargo run --release --bin etna -- quickcheck OctalFmtNoPanic` |
| 023 | `octal_fmt_buffer_size_aeef0be_1` | crabcheck | `property_octal_fmt_no_panic` | `witness_octal_fmt_no_panic_case_one_u64`, `witness_octal_fmt_no_panic_case_three_u64` | `cargo run --release --bin etna -- crabcheck OctalFmtNoPanic` |
| 024 | `octal_fmt_buffer_size_aeef0be_1` | hegel | `property_octal_fmt_no_panic` | `witness_octal_fmt_no_panic_case_one_u64`, `witness_octal_fmt_no_panic_case_three_u64` | `cargo run --release --bin etna -- hegel OctalFmtNoPanic` |

## Witness catalog

Each witness is a deterministic concrete test in `tests/etna_witnesses.rs`. Base build: passes. Variant-active build: fails.

- `witness_split_at_mut_accepts_len_case_empty_right` — `property_split_at_mut_accepts_len([false, true, false, true])` → `Pass`. Under `split_at_mut_rejects_len_c71ea23_1` the `split_at_mut(len)` call panics instead of yielding an empty right half.
- `witness_split_at_mut_accepts_len_case_zero_len` — `property_split_at_mut_accepts_len([])` → `Pass`. Same bug observed at `split_at_mut(0)` on an empty slice.
- `witness_vec_insert_accepts_end_case_push_true` — `property_vec_insert_accepts_end([false, false, true], true)` → `Pass`. Under `vec_insert_rejects_end_8e48751_1` the end-index `insert` panics.
- `witness_vec_insert_accepts_end_case_empty_vec` — `property_vec_insert_accepts_end([], true)` → `Pass`. Same bug observed at `insert(0, _)` on an empty vec.
- `witness_leading_trailing_fallback_case_all_ones` — `property_leading_trailing_fallback([true; 5], true)` → `Pass`. Under `leading_trailing_homogeneous_b08c4bd_1` an all-ones 5-bit slice reports `leading_ones() = 0` instead of `5`.
- `witness_leading_trailing_fallback_case_all_zeros` — `property_leading_trailing_fallback([false; 5], false)` → `Pass`. Same bug observed via the all-zeros fallback.
- `witness_bitvec_partial_cmp_matches_case_lt` — `property_bitvec_partial_cmp_matches([false, true], [true, false])` → `Pass`. Under `bitvec_partial_cmp_reversed_f3d5b43_1` `BitVec.partial_cmp(&BitSlice)` returns `Greater` while `BitVec.partial_cmp(&BitVec)` returns `Less`.
- `witness_bitvec_partial_cmp_matches_case_gt` — `property_bitvec_partial_cmp_matches([true, true, true], [false, false, false])` → `Pass`. Same bug observed with the opposite polarity.
- `witness_clone_from_bitslice_copies_src_case_flip_all` — `property_clone_from_bitslice_copies_src([false; 9], [true; 9])` → `Pass`. Under `clone_from_bitslice_src_bug_935cad8_1` `dst` stays all-zero after the clone because the non-coerce fallback is an identity loop.
- `witness_clone_from_bitslice_copies_src_case_alternating` — `property_clone_from_bitslice_copies_src([true; 5], [false, true, false, true, false])` → `Pass`. Same bug observed with alternating source bits.
- `witness_octal_fmt_no_panic_case_one_u64` — `property_octal_fmt_no_panic(0)` → `Pass` (1 × u64 body chunk = 64 bits, must emit ≥ 22 octal digits). Under `octal_fmt_buffer_size_aeef0be_1` the under-sized buffer emits only 21.
- `witness_octal_fmt_no_panic_case_three_u64` — `property_octal_fmt_no_panic(2)` → `Pass` (3 × u64 = 192 bits, must emit ≥ 66 digits). Same bug observed across multiple body chunks.

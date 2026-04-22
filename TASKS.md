# bitvec-rs — ETNA Tasks

Total tasks: 24

## Task Index

| Task | Variant | Framework | Property | Witness |
|------|---------|-----------|----------|---------|
| 001 | `bitvec_partial_cmp_reversed_f3d5b43_1` | proptest | `BitvecPartialCmpMatches` | `witness_bitvec_partial_cmp_matches_case_lt` |
| 002 | `bitvec_partial_cmp_reversed_f3d5b43_1` | quickcheck | `BitvecPartialCmpMatches` | `witness_bitvec_partial_cmp_matches_case_lt` |
| 003 | `bitvec_partial_cmp_reversed_f3d5b43_1` | crabcheck | `BitvecPartialCmpMatches` | `witness_bitvec_partial_cmp_matches_case_lt` |
| 004 | `bitvec_partial_cmp_reversed_f3d5b43_1` | hegel | `BitvecPartialCmpMatches` | `witness_bitvec_partial_cmp_matches_case_lt` |
| 005 | `clone_from_bitslice_src_bug_935cad8_1` | proptest | `CloneFromBitsliceCopiesSrc` | `witness_clone_from_bitslice_copies_src_case_flip_all` |
| 006 | `clone_from_bitslice_src_bug_935cad8_1` | quickcheck | `CloneFromBitsliceCopiesSrc` | `witness_clone_from_bitslice_copies_src_case_flip_all` |
| 007 | `clone_from_bitslice_src_bug_935cad8_1` | crabcheck | `CloneFromBitsliceCopiesSrc` | `witness_clone_from_bitslice_copies_src_case_flip_all` |
| 008 | `clone_from_bitslice_src_bug_935cad8_1` | hegel | `CloneFromBitsliceCopiesSrc` | `witness_clone_from_bitslice_copies_src_case_flip_all` |
| 009 | `leading_trailing_homogeneous_b08c4bd_1` | proptest | `LeadingTrailingFallback` | `witness_leading_trailing_fallback_case_all_ones` |
| 010 | `leading_trailing_homogeneous_b08c4bd_1` | quickcheck | `LeadingTrailingFallback` | `witness_leading_trailing_fallback_case_all_ones` |
| 011 | `leading_trailing_homogeneous_b08c4bd_1` | crabcheck | `LeadingTrailingFallback` | `witness_leading_trailing_fallback_case_all_ones` |
| 012 | `leading_trailing_homogeneous_b08c4bd_1` | hegel | `LeadingTrailingFallback` | `witness_leading_trailing_fallback_case_all_ones` |
| 013 | `octal_fmt_buffer_size_aeef0be_1` | proptest | `OctalFmtNoPanic` | `witness_octal_fmt_no_panic_case_one_u64` |
| 014 | `octal_fmt_buffer_size_aeef0be_1` | quickcheck | `OctalFmtNoPanic` | `witness_octal_fmt_no_panic_case_one_u64` |
| 015 | `octal_fmt_buffer_size_aeef0be_1` | crabcheck | `OctalFmtNoPanic` | `witness_octal_fmt_no_panic_case_one_u64` |
| 016 | `octal_fmt_buffer_size_aeef0be_1` | hegel | `OctalFmtNoPanic` | `witness_octal_fmt_no_panic_case_one_u64` |
| 017 | `split_at_mut_rejects_len_c71ea23_1` | proptest | `SplitAtMutAcceptsLen` | `witness_split_at_mut_accepts_len_case_empty_right` |
| 018 | `split_at_mut_rejects_len_c71ea23_1` | quickcheck | `SplitAtMutAcceptsLen` | `witness_split_at_mut_accepts_len_case_empty_right` |
| 019 | `split_at_mut_rejects_len_c71ea23_1` | crabcheck | `SplitAtMutAcceptsLen` | `witness_split_at_mut_accepts_len_case_empty_right` |
| 020 | `split_at_mut_rejects_len_c71ea23_1` | hegel | `SplitAtMutAcceptsLen` | `witness_split_at_mut_accepts_len_case_empty_right` |
| 021 | `vec_insert_rejects_end_8e48751_1` | proptest | `VecInsertAcceptsEnd` | `witness_vec_insert_accepts_end_case_push_true` |
| 022 | `vec_insert_rejects_end_8e48751_1` | quickcheck | `VecInsertAcceptsEnd` | `witness_vec_insert_accepts_end_case_push_true` |
| 023 | `vec_insert_rejects_end_8e48751_1` | crabcheck | `VecInsertAcceptsEnd` | `witness_vec_insert_accepts_end_case_push_true` |
| 024 | `vec_insert_rejects_end_8e48751_1` | hegel | `VecInsertAcceptsEnd` | `witness_vec_insert_accepts_end_case_push_true` |

## Witness Catalog

- `witness_bitvec_partial_cmp_matches_case_lt` — base passes, variant fails
- `witness_bitvec_partial_cmp_matches_case_gt` — base passes, variant fails
- `witness_clone_from_bitslice_copies_src_case_flip_all` — base passes, variant fails
- `witness_clone_from_bitslice_copies_src_case_alternating` — base passes, variant fails
- `witness_leading_trailing_fallback_case_all_ones` — base passes, variant fails
- `witness_leading_trailing_fallback_case_all_zeros` — base passes, variant fails
- `witness_octal_fmt_no_panic_case_one_u64` — base passes, variant fails
- `witness_octal_fmt_no_panic_case_three_u64` — base passes, variant fails
- `witness_split_at_mut_accepts_len_case_empty_right` — base passes, variant fails
- `witness_split_at_mut_accepts_len_case_zero_len` — base passes, variant fails
- `witness_vec_insert_accepts_end_case_push_true` — base passes, variant fails
- `witness_vec_insert_accepts_end_case_empty_vec` — base passes, variant fails

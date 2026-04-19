# bitvec — Injected Bugs

Total mutations: 6

All variants are patch-based; apply the listed patch to a clean HEAD to reproduce the buggy build. Each `etna/<variant>` branch is a pre-applied snapshot.

## Bug Index

| # | Name | Variant | File | Injection | Fix Commit |
|---|------|---------|------|-----------|------------|
| 1 | `split_at_mut` rejects `mid == self.len()` | `split_at_mut_rejects_len_c71ea23_1` | `patches/split_at_mut_rejects_len_c71ea23_1.patch` | patch | `c71ea233bbf62e7a9fd4a0ea06d4eaa1fb6b0e11` |
| 2 | `BitVec::insert` rejects `index == self.len()` | `vec_insert_rejects_end_8e48751_1` | `patches/vec_insert_rejects_end_8e48751_1.patch` | patch | `8e48751fa6cf0f6e64fb5a73b27bab0676a25a1f` |
| 3 | Homogeneous-slice `{leading,trailing}_{ones,zeros}` fallback | `leading_trailing_homogeneous_b08c4bd_1` | `patches/leading_trailing_homogeneous_b08c4bd_1.patch` | patch | `b08c4bd61fdadad7e5f8c2cd64bb98d29edff5b0` |
| 4 | `BitVec` vs `BitSlice` `partial_cmp` direction | `bitvec_partial_cmp_reversed_f3d5b43_1` | `patches/bitvec_partial_cmp_reversed_f3d5b43_1.patch` | patch | `f3d5b43a9ae4bef6a0b45d4db5ad27adf3cfe3ee` |
| 5 | `BitSlice::clone_from_bitslice` non-coerce fallback | `clone_from_bitslice_src_bug_935cad8_1` | `patches/clone_from_bitslice_src_bug_935cad8_1.patch` | patch | `935cad89f4ad87a92f72d9f2ce48d3d59ab7f23a` |
| 6 | `Octal::fmt` buffer ceiling-division | `octal_fmt_buffer_size_aeef0be_1` | `patches/octal_fmt_buffer_size_aeef0be_1.patch` | patch | `aeef0bed4ac83f0cdf3d97c2f43957eab24d30f5` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `split_at_mut_rejects_len_c71ea23_1` | `property_split_at_mut_accepts_len` | `witness_split_at_mut_accepts_len_case_empty_right`, `witness_split_at_mut_accepts_len_case_zero_len` |
| `vec_insert_rejects_end_8e48751_1` | `property_vec_insert_accepts_end` | `witness_vec_insert_accepts_end_case_push_true`, `witness_vec_insert_accepts_end_case_empty_vec` |
| `leading_trailing_homogeneous_b08c4bd_1` | `property_leading_trailing_fallback` | `witness_leading_trailing_fallback_case_all_ones`, `witness_leading_trailing_fallback_case_all_zeros` |
| `bitvec_partial_cmp_reversed_f3d5b43_1` | `property_bitvec_partial_cmp_matches` | `witness_bitvec_partial_cmp_matches_case_lt`, `witness_bitvec_partial_cmp_matches_case_gt` |
| `clone_from_bitslice_src_bug_935cad8_1` | `property_clone_from_bitslice_copies_src` | `witness_clone_from_bitslice_copies_src_case_flip_all`, `witness_clone_from_bitslice_copies_src_case_alternating` |
| `octal_fmt_buffer_size_aeef0be_1` | `property_octal_fmt_no_panic` | `witness_octal_fmt_no_panic_case_one_u64`, `witness_octal_fmt_no_panic_case_three_u64` |

## Framework Coverage

| Property | etna | proptest | quickcheck | crabcheck | hegel |
|----------|:----:|:--------:|:----------:|:---------:|:-----:|
| `property_split_at_mut_accepts_len` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `property_vec_insert_accepts_end` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `property_leading_trailing_fallback` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `property_bitvec_partial_cmp_matches` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `property_clone_from_bitslice_copies_src` | ✓ | ✓ | ✓ | ✓ | ✓ |
| `property_octal_fmt_no_panic` | ✓ | ✓ | ✓ | ✓ | ✓ |

## Bug Details

### 1. `split_at_mut` rejects `mid == self.len()`

- **Variant**: `split_at_mut_rejects_len_c71ea23_1`
- **Location**: `patches/split_at_mut_rejects_len_c71ea23_1.patch` (applies to `src/slice/api.rs`)
- **Property**: `property_split_at_mut_accepts_len`
- **Witness(es)**: `witness_split_at_mut_accepts_len_case_empty_right`, `witness_split_at_mut_accepts_len_case_zero_len`
- **Fix commit**: `c71ea233bbf62e7a9fd4a0ea06d4eaa1fb6b0e11` — `fix split_at_mut as well`
- **Invariant violated**: `split_at_mut(mid)` must accept `mid == self.len()`; the right half is simply empty.
- **How the mutation triggers**: Reverts the `assert_in_bounds` bound from `0 ..= self.len()` back to `0 .. self.len()`, so splitting at the exact length panics instead of yielding `(self, empty)`.

### 2. `BitVec::insert` rejects `index == self.len()`

- **Variant**: `vec_insert_rejects_end_8e48751_1`
- **Location**: `patches/vec_insert_rejects_end_8e48751_1.patch` (applies to `src/vec/api.rs`)
- **Property**: `property_vec_insert_accepts_end`
- **Witness(es)**: `witness_vec_insert_accepts_end_case_push_true`, `witness_vec_insert_accepts_end_case_empty_vec`
- **Fix commit**: `8e48751fa6cf0f6e64fb5a73b27bab0676a25a1f` — `vec/api: fix insert for index == len`
- **Invariant violated**: `BitVec::insert(self.len(), v)` must be equivalent to `push(v)` — appending a bit at the tail, not panicking.
- **How the mutation triggers**: Reverts the inclusive upper bound so `assert_in_bounds(index, 0 .. self.len())` rejects the exact-length case, producing a panic on the push-equivalent call.

### 3. Homogeneous-slice `{leading,trailing}_{ones,zeros}` fallback

- **Variant**: `leading_trailing_homogeneous_b08c4bd_1`
- **Location**: `patches/leading_trailing_homogeneous_b08c4bd_1.patch` (applies to `src/slice/api.rs`)
- **Property**: `property_leading_trailing_fallback`
- **Witness(es)**: `witness_leading_trailing_fallback_case_all_ones`, `witness_leading_trailing_fallback_case_all_zeros`
- **Fix commit**: `b08c4bd61fdadad7e5f8c2cd64bb98d29edff5b0` — `Fix edge case for {leading,trailing}_{zero,one}`
- **Invariant violated**: For an N-bit slice whose bits are all `true` (resp. `false`), `leading_ones() == trailing_ones() == N` (resp. `…_zeros() == N`).
- **How the mutation triggers**: The `unwrap_or_else(|| self.len())` / `unwrap_or(len)` fallbacks on the `find_bit_{ones,zeros}` helpers are reverted to `unwrap_or_default()`, so the all-same-value case falls through to `0` instead of the slice length.

### 4. `BitVec` vs `BitSlice` `partial_cmp` direction

- **Variant**: `bitvec_partial_cmp_reversed_f3d5b43_1`
- **Location**: `patches/bitvec_partial_cmp_reversed_f3d5b43_1.patch` (applies to `src/vec/traits.rs`)
- **Property**: `property_bitvec_partial_cmp_matches`
- **Witness(es)**: `witness_bitvec_partial_cmp_matches_case_lt`, `witness_bitvec_partial_cmp_matches_case_gt`
- **Fix commit**: `f3d5b43a9ae4bef6a0b45d4db5ad27adf3cfe3ee` — `Fix BitVec to BitSlice partial_cmp. Fixes #215`
- **Invariant violated**: `bv.partial_cmp(bv.as_bitslice())` must match `bv.partial_cmp(&bv)` (and transitively, ordering between equal-content `BitVec`s and `BitSlice`s must agree).
- **How the mutation triggers**: Drops the `.map(|o| o.reverse())` on the `impl<T, O, Rhs> PartialOrd<Rhs> for BitVec<T, O>` blanket, so delegating through `other.partial_cmp(self.as_bitslice())` returns the inverted ordering relative to the direct `BitVec: PartialOrd<BitVec>` impl.

### 5. `BitSlice::clone_from_bitslice` non-coerce fallback

- **Variant**: `clone_from_bitslice_src_bug_935cad8_1`
- **Location**: `patches/clone_from_bitslice_src_bug_935cad8_1.patch` (applies to `src/slice.rs`)
- **Property**: `property_clone_from_bitslice_copies_src`
- **Witness(es)**: `witness_clone_from_bitslice_copies_src_case_flip_all`, `witness_clone_from_bitslice_copies_src_case_alternating`
- **Fix commit**: `935cad89f4ad87a92f72d9f2ce48d3d59ab7f23a` — `Fix a bug in BitSlice::clone_from_bitslice`
- **Invariant violated**: After `dst.clone_from_bitslice(&src)` (equal-length slices), every bit of `dst` must equal the corresponding bit of `src`.
- **How the mutation triggers**: The non-coerce fallback branch is gutted to an identity loop (semantically equivalent to the pre-935cad8 `self->self` zip), so when the `<T, O>` and `<T2, O2>` parameters differ — defeating the coerce fast path — `dst` is left unchanged. The property uses heterogeneous `<u8, Lsb0>` / `<u16, Msb0>` to force the fallback.

### 6. `Octal::fmt` buffer ceiling-division

- **Variant**: `octal_fmt_buffer_size_aeef0be_1`
- **Location**: `patches/octal_fmt_buffer_size_aeef0be_1.patch` (applies to `src/slice/traits.rs`)
- **Property**: `property_octal_fmt_no_panic`
- **Witness(es)**: `witness_octal_fmt_no_panic_case_one_u64`, `witness_octal_fmt_no_panic_case_three_u64`
- **Fix commit**: `aeef0bed4ac83f0cdf3d97c2f43957eab24d30f5` — `Ensure correct buffer size during Octal::fmt`
- **Invariant violated**: Formatting a `BitSlice` via the `Octal` trait must emit `ceil(bits / 3)` octal digits per storage-element body chunk. A `BitSlice<u64, _>` of 64 bits must emit 22 digits, not 21.
- **How the mutation triggers**: The pre-allocated digit buffer width `W = D + (M != 0)` (with `D = usize::BITS / 3`, `M = usize::BITS % 3`) is reverted to `W = D`, dropping the ceiling. The per-chunk zip against the under-sized buffer truncates one octal digit from each 64-bit body chunk.

# bitvec-rs — Injected Bugs

Total mutations: 6

## Bug Index

| # | Variant | Name | Location | Injection | Fix Commit |
|---|---------|------|----------|-----------|------------|
| 1 | `bitvec_partial_cmp_reversed_f3d5b43_1` | `bitvec_partial_cmp_reversed` | `src/vec/traits.rs` | `patch` | `f3d5b43000f6e9a52a233bf07c295929cce66387` |
| 2 | `clone_from_bitslice_src_bug_935cad8_1` | `clone_from_bitslice_src_bug` | `src/slice.rs` | `patch` | `935cad8888d0f09837b95000c44f4a56579c7108` |
| 3 | `leading_trailing_homogeneous_b08c4bd_1` | `leading_trailing_homogeneous` | `src/slice/api.rs` | `patch` | `b08c4bdde959c172d5e77fe1746e903688b804e4` |
| 4 | `octal_fmt_buffer_size_aeef0be_1` | `octal_fmt_buffer_size` | `src/slice/traits.rs` | `patch` | `aeef0be672538b8f6694f464bd947084813fb327` |
| 5 | `split_at_mut_rejects_len_c71ea23_1` | `split_at_mut_rejects_len` | `src/slice/api.rs` | `patch` | `c71ea2344fd4a728263f33d66a9708d93c52dc48` |
| 6 | `vec_insert_rejects_end_8e48751_1` | `vec_insert_rejects_end` | `src/vec/api.rs` | `patch` | `8e48751508526cbe0c656f4dffa60cf4e6e4bfb0` |

## Property Mapping

| Variant | Property | Witness(es) |
|---------|----------|-------------|
| `bitvec_partial_cmp_reversed_f3d5b43_1` | `BitvecPartialCmpMatches` | `witness_bitvec_partial_cmp_matches_case_lt`, `witness_bitvec_partial_cmp_matches_case_gt` |
| `clone_from_bitslice_src_bug_935cad8_1` | `CloneFromBitsliceCopiesSrc` | `witness_clone_from_bitslice_copies_src_case_flip_all`, `witness_clone_from_bitslice_copies_src_case_alternating` |
| `leading_trailing_homogeneous_b08c4bd_1` | `LeadingTrailingFallback` | `witness_leading_trailing_fallback_case_all_ones`, `witness_leading_trailing_fallback_case_all_zeros` |
| `octal_fmt_buffer_size_aeef0be_1` | `OctalFmtNoPanic` | `witness_octal_fmt_no_panic_case_one_u64`, `witness_octal_fmt_no_panic_case_three_u64` |
| `split_at_mut_rejects_len_c71ea23_1` | `SplitAtMutAcceptsLen` | `witness_split_at_mut_accepts_len_case_empty_right`, `witness_split_at_mut_accepts_len_case_zero_len` |
| `vec_insert_rejects_end_8e48751_1` | `VecInsertAcceptsEnd` | `witness_vec_insert_accepts_end_case_push_true`, `witness_vec_insert_accepts_end_case_empty_vec` |

## Framework Coverage

| Property | proptest | quickcheck | crabcheck | hegel |
|----------|---------:|-----------:|----------:|------:|
| `BitvecPartialCmpMatches` | ✓ | ✓ | ✓ | ✓ |
| `CloneFromBitsliceCopiesSrc` | ✓ | ✓ | ✓ | ✓ |
| `LeadingTrailingFallback` | ✓ | ✓ | ✓ | ✓ |
| `OctalFmtNoPanic` | ✓ | ✓ | ✓ | ✓ |
| `SplitAtMutAcceptsLen` | ✓ | ✓ | ✓ | ✓ |
| `VecInsertAcceptsEnd` | ✓ | ✓ | ✓ | ✓ |

## Bug Details

### 1. bitvec_partial_cmp_reversed

- **Variant**: `bitvec_partial_cmp_reversed_f3d5b43_1`
- **Location**: `src/vec/traits.rs`
- **Property**: `BitvecPartialCmpMatches`
- **Witness(es)**:
  - `witness_bitvec_partial_cmp_matches_case_lt`
  - `witness_bitvec_partial_cmp_matches_case_gt`
- **Source**: Fix BitVec to BitSlice partial_cmp. Fixes #215
  > The blanket `PartialOrd<Rhs> for BitVec<T, O>` impl delegated via `other.partial_cmp(self.as_bitslice())` but forgot to reverse the result, so comparing a `BitVec` to a `BitSlice` returned the opposite ordering from the direct `BitVec: PartialOrd<BitVec>` impl. The fix adds `.map(|o| o.reverse())`.
- **Fix commit**: `f3d5b43000f6e9a52a233bf07c295929cce66387` — Fix BitVec to BitSlice partial_cmp. Fixes #215
- **Invariant violated**: `bv.partial_cmp(bv.as_bitslice())` must match `bv.partial_cmp(&bv)` (and transitively, ordering between equal-content `BitVec`s and `BitSlice`s must agree).
- **How the mutation triggers**: Drops the `.map(|o| o.reverse())` on the `impl<T, O, Rhs> PartialOrd<Rhs> for BitVec<T, O>` blanket, so delegating through `other.partial_cmp(self.as_bitslice())` returns the inverted ordering relative to the direct `BitVec: PartialOrd<BitVec>` impl.

### 2. clone_from_bitslice_src_bug

- **Variant**: `clone_from_bitslice_src_bug_935cad8_1`
- **Location**: `src/slice.rs`
- **Property**: `CloneFromBitsliceCopiesSrc`
- **Witness(es)**:
  - `witness_clone_from_bitslice_copies_src_case_flip_all`
  - `witness_clone_from_bitslice_copies_src_case_alternating`
- **Source**: Fix a bug in BitSlice::clone_from_bitslice
  > When the coerce fast path did not apply (different `T`/`O` parameters on source and destination), `clone_from_bitslice` fell through to a `self → self` zip that left `dst` unchanged instead of copying bits from `src`. The fix makes the fallback actually read from `src`.
- **Fix commit**: `935cad8888d0f09837b95000c44f4a56579c7108` — Fix a bug in BitSlice::clone_from_bitslice
- **Invariant violated**: After `dst.clone_from_bitslice(&src)` (equal-length slices), every bit of `dst` must equal the corresponding bit of `src`.
- **How the mutation triggers**: The non-coerce fallback branch is gutted to an identity loop (semantically equivalent to the pre-935cad8 `self->self` zip), so when the `<T, O>` and `<T2, O2>` parameters differ — defeating the coerce fast path — `dst` is left unchanged. The property uses heterogeneous `<u8, Lsb0>` / `<u16, Msb0>` to force the fallback.

### 3. leading_trailing_homogeneous

- **Variant**: `leading_trailing_homogeneous_b08c4bd_1`
- **Location**: `src/slice/api.rs`
- **Property**: `LeadingTrailingFallback`
- **Witness(es)**:
  - `witness_leading_trailing_fallback_case_all_ones`
  - `witness_leading_trailing_fallback_case_all_zeros`
- **Source**: Fix edge case for {leading,trailing}_{zero,one}
  > `leading_ones`/`trailing_ones`/`leading_zeros`/`trailing_zeros` called `unwrap_or_default()` on the underlying `find_bit_*` helpers, so an all-same-value slice (where the search finds no boundary bit) returned `0` instead of the slice length. The fix switches to `unwrap_or(self.len())`.
- **Fix commit**: `b08c4bdde959c172d5e77fe1746e903688b804e4` — Fix edge case for {leading,trailing}_{zero,one}
- **Invariant violated**: For an N-bit slice whose bits are all `true` (resp. `false`), `leading_ones() == trailing_ones() == N` (resp. `…_zeros() == N`).
- **How the mutation triggers**: The `unwrap_or_else(|| self.len())` / `unwrap_or(len)` fallbacks on the `find_bit_{ones,zeros}` helpers are reverted to `unwrap_or_default()`, so the all-same-value case falls through to `0` instead of the slice length.

### 4. octal_fmt_buffer_size

- **Variant**: `octal_fmt_buffer_size_aeef0be_1`
- **Location**: `src/slice/traits.rs`
- **Property**: `OctalFmtNoPanic`
- **Witness(es)**:
  - `witness_octal_fmt_no_panic_case_one_u64`
  - `witness_octal_fmt_no_panic_case_three_u64`
- **Source**: Ensure correct buffer size during Octal::fmt
  > `Octal::fmt` preallocated a digit buffer of width `usize::BITS / 3` (floor) instead of `ceil(usize::BITS / 3)`, so each storage-element body chunk lost one octal digit. The fix adds `+ (usize::BITS % 3 != 0) as usize`, rounding up.
- **Fix commit**: `aeef0be672538b8f6694f464bd947084813fb327` — Ensure correct buffer size during Octal::fmt
- **Invariant violated**: Formatting a `BitSlice` via the `Octal` trait must emit `ceil(bits / 3)` octal digits per storage-element body chunk. A `BitSlice<u64, _>` of 64 bits must emit 22 digits, not 21.
- **How the mutation triggers**: The pre-allocated digit buffer width `W = D + (M != 0)` (with `D = usize::BITS / 3`, `M = usize::BITS % 3`) is reverted to `W = D`, dropping the ceiling. The per-chunk zip against the under-sized buffer truncates one octal digit from each 64-bit body chunk.

### 5. split_at_mut_rejects_len

- **Variant**: `split_at_mut_rejects_len_c71ea23_1`
- **Location**: `src/slice/api.rs`
- **Property**: `SplitAtMutAcceptsLen`
- **Witness(es)**:
  - `witness_split_at_mut_accepts_len_case_empty_right`
  - `witness_split_at_mut_accepts_len_case_zero_len`
- **Source**: fix split_at_mut as well
  > `BitSlice::split_at_mut` asserted `mid < self.len()`, rejecting the exact-length split that should return `(self, empty)`. The fix bumps the assertion to `mid <= self.len()`, mirroring `slice::split_at_mut`'s inclusive bound.
- **Fix commit**: `c71ea2344fd4a728263f33d66a9708d93c52dc48` — fix split_at_mut as well
- **Invariant violated**: `split_at_mut(mid)` must accept `mid == self.len()`; the right half is simply empty.
- **How the mutation triggers**: Reverts the `assert_in_bounds` bound from `0 ..= self.len()` back to `0 .. self.len()`, so splitting at the exact length panics instead of yielding `(self, empty)`.

### 6. vec_insert_rejects_end

- **Variant**: `vec_insert_rejects_end_8e48751_1`
- **Location**: `src/vec/api.rs`
- **Property**: `VecInsertAcceptsEnd`
- **Witness(es)**:
  - `witness_vec_insert_accepts_end_case_push_true`
  - `witness_vec_insert_accepts_end_case_empty_vec`
- **Source**: vec/api: fix insert for index == len
  > `BitVec::insert` bounded the index with `0 .. self.len()`, so calling `bv.insert(bv.len(), v)` — the push-equivalent case — panicked. The fix widens the bound to `0 ..= self.len()`, aligning with `Vec::insert` semantics.
- **Fix commit**: `8e48751508526cbe0c656f4dffa60cf4e6e4bfb0` — vec/api: fix insert for index == len
- **Invariant violated**: `BitVec::insert(self.len(), v)` must be equivalent to `push(v)` — appending a bit at the tail, not panicking.
- **How the mutation triggers**: Reverts the inclusive upper bound so `assert_in_bounds(index, 0 .. self.len())` rejects the exact-length case, producing a panic on the push-equivalent call.

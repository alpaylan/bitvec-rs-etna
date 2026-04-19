// Deterministic witness tests for ETNA variants.
//
// Each `witness_<name>_case_<tag>` passes on the base commit and fails under the
// corresponding `etna/<variant>` branch. Witnesses call `property_<name>`
// directly with frozen inputs; they do not touch framework machinery (no
// proptest, no quickcheck, no RNG, no clocks).

use bitvec::etna::{
    property_bitvec_partial_cmp_matches, property_clone_from_bitslice_copies_src,
    property_leading_trailing_fallback, property_octal_fmt_no_panic,
    property_split_at_mut_accepts_len, property_vec_insert_accepts_end, PropertyResult,
};

fn expect_pass(r: PropertyResult, what: &str) {
    match r {
        PropertyResult::Pass => (),
        PropertyResult::Fail(m) => panic!("{what}: property failed: {m}"),
        PropertyResult::Discard => panic!("{what}: unexpected discard"),
    }
}

// Variant: split_at_mut_rejects_len_c71ea23_1
//
// Mutation restores an exclusive upper bound on `split_at_mut`, so calling
// `split_at_mut(len)` — a valid split that yields an empty right slice — panics.
#[test]
fn witness_split_at_mut_accepts_len_case_empty_right() {
    expect_pass(
        property_split_at_mut_accepts_len(vec![false, true, false, true]),
        "split_at_mut accepts self.len()",
    );
}

#[test]
fn witness_split_at_mut_accepts_len_case_zero_len() {
    expect_pass(
        property_split_at_mut_accepts_len(vec![]),
        "split_at_mut accepts self.len() on empty",
    );
}

// Variant: vec_insert_rejects_end_8e48751_1
//
// Mutation reverts `BitVec::insert` to use an exclusive upper bound, so
// `bv.insert(bv.len(), v)` — which should be equivalent to `push` — panics.
#[test]
fn witness_vec_insert_accepts_end_case_push_true() {
    expect_pass(
        property_vec_insert_accepts_end(vec![false, false, true], true),
        "insert(len, true)",
    );
}

#[test]
fn witness_vec_insert_accepts_end_case_empty_vec() {
    expect_pass(
        property_vec_insert_accepts_end(vec![], true),
        "insert(0, _) on empty vec",
    );
}

// Variant: leading_ones_all_ones_b08c4bd_1
//
// Mutation restores the pre-b08c4bd fallback (`unwrap_or_default`) in
// `leading_ones`, so an all-ones slice reports `0` instead of `len`.
#[test]
fn witness_leading_trailing_fallback_case_all_ones() {
    expect_pass(
        property_leading_trailing_fallback(vec![true; 5], true),
        "all-ones leading_ones == len",
    );
}

#[test]
fn witness_leading_trailing_fallback_case_all_zeros() {
    expect_pass(
        property_leading_trailing_fallback(vec![false; 5], false),
        "all-zeros leading_zeros == len",
    );
}

// Variant: bitvec_partial_cmp_reversed_f3d5b43_1
//
// Mutation drops the `.map(|o| o.reverse())` in the
// `BitVec: PartialOrd<Rhs: AsRef<BitSlice>>` impl, so BitVec-vs-BitSlice
// ordering is the inverse of BitVec-vs-BitVec ordering.
#[test]
fn witness_bitvec_partial_cmp_matches_case_lt() {
    expect_pass(
        property_bitvec_partial_cmp_matches(vec![false, true], vec![true, false]),
        "0b10 < 0b01",
    );
}

#[test]
fn witness_bitvec_partial_cmp_matches_case_gt() {
    expect_pass(
        property_bitvec_partial_cmp_matches(vec![true, true, true], vec![false, false, false]),
        "0b111 > 0b000",
    );
}

// Variant: clone_from_bitslice_src_bug_935cad8_1
//
// Mutation restores the pre-935cad8 identity loop (zipping
// `self.as_mut_bitptr_range()` with `self.as_bitptr_range()` instead of
// `src.as_bitptr_range()`), so `clone_from_bitslice` leaves `self` unchanged.
#[test]
fn witness_clone_from_bitslice_copies_src_case_flip_all() {
    expect_pass(
        property_clone_from_bitslice_copies_src(
            vec![false, false, false, false, false, false, false, false, false],
            vec![true, true, true, true, true, true, true, true, true],
        ),
        "dst=all-zero, src=all-one",
    );
}

#[test]
fn witness_clone_from_bitslice_copies_src_case_alternating() {
    expect_pass(
        property_clone_from_bitslice_copies_src(
            vec![true, true, true, true, true],
            vec![false, true, false, true, false],
        ),
        "dst=all-one, src=alternating",
    );
}

// Variant: octal_fmt_buffer_size_aeef0be_1
//
// Mutation removes the ceiling-division modulus from the `Octal::fmt` buffer
// so the formatter under-allocates and overflows into `unreachable_unchecked`
// for slices whose bit-count is not a multiple of 3 above `usize::BITS`.
#[test]
fn witness_octal_fmt_no_panic_case_one_u64() {
    expect_pass(
        property_octal_fmt_no_panic(0),
        "Octal::fmt one full u64 body chunk (64 bits → 22 digits)",
    );
}

#[test]
fn witness_octal_fmt_no_panic_case_three_u64() {
    expect_pass(
        property_octal_fmt_no_panic(2),
        "Octal::fmt three full u64 body chunks (192 bits → 66 digits)",
    );
}

//! ETNA framework-neutral property functions for bitvec.
//!
//! Each `property_<name>` is a pure function taking concrete, owned inputs and
//! returning `PropertyResult`. Framework adapters (proptest/quickcheck/crabcheck/hegel)
//! in `src/bin/etna.rs` and deterministic witness tests in `tests/etna_witnesses.rs`
//! both call these functions directly — there is no re-implementation of the
//! invariant inside any adapter.

#![allow(missing_docs)]

use crate::prelude::*;
use core::cmp::Ordering;
use core::fmt::Write;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::string::{String, ToString};

pub enum PropertyResult {
    Pass,
    Fail(String),
    Discard,
}

fn bools_to_bitvec(bits: &[bool]) -> BitVec {
    let mut out = BitVec::<usize, Lsb0>::with_capacity(bits.len());
    for &b in bits {
        out.push(b);
    }
    out
}

/// Truncates a caller-supplied length hint into the tested range.
fn clamp_len(n: usize) -> usize {
    n % 64
}

/// `split_at_mut(mid)` must accept any `mid` in `0..=self.len()` without
/// panicking — when `mid == self.len()`, the right slice is simply empty.
/// Detects the c71ea23 bug where `split_at_mut` used
/// `assert_in_bounds(mid, 0..self.len())` instead of `0..=self.len()`, so
/// splitting at exactly the length panicked.
///
/// `mid_seed` is mapped via `mid_seed % (len + 1)` so the property is
/// library-faithful: any drawn seed picks a valid `mid` somewhere in the
/// inclusive range, and the `mid == len` boundary that exercises the patched
/// assertion is hit only ~1/(len+1) of the time.
pub fn property_split_at_mut_accepts_len(seed: Vec<bool>, mid_seed: usize) -> PropertyResult {
    let bits = seed;
    let mut bv = bools_to_bitvec(&bits);
    let len = bv.len();
    let mid = mid_seed % (len + 1);
    let result = catch_unwind(AssertUnwindSafe(|| {
        let slice = bv.as_mut_bitslice();
        let (left, right) = slice.split_at_mut(mid);
        (left.len(), right.len())
    }));
    match result {
        Ok((l, r)) => {
            if l == mid && r == len - mid {
                PropertyResult::Pass
            } else {
                PropertyResult::Fail(format!(
                    "split_at_mut(mid={mid}, len={len}) returned sizes ({l},{r}); expected ({mid},{})",
                    len - mid
                ))
            }
        }
        Err(_) => PropertyResult::Fail(format!(
            "split_at_mut(mid={mid}) panicked; must accept any mid in 0..=self.len()={len}"
        )),
    }
}

/// `BitVec::insert(index, value)` must accept any `index` in `0..=self.len()`.
/// The `index == self.len()` case is equivalent to `push`. Detects the 8e48751
/// bug where `insert` used `assert_in_bounds` with an exclusive upper bound,
/// so `bv.insert(bv.len(), v)` panicked.
///
/// `index_seed` is mapped via `index_seed % (len + 1)` so the property is
/// library-faithful: any drawn seed picks a valid `index` somewhere in the
/// inclusive range, and the `index == len` boundary that exercises the patched
/// assertion is hit only ~1/(len+1) of the time.
pub fn property_vec_insert_accepts_end(
    seed: Vec<bool>,
    value: bool,
    index_seed: usize,
) -> PropertyResult {
    let mut bv = bools_to_bitvec(&seed);
    let len_before = bv.len();
    let index = index_seed % (len_before + 1);
    let result = catch_unwind(AssertUnwindSafe(|| {
        bv.insert(index, value);
        bv
    }));
    match result {
        Ok(bv_after) => {
            let new_len = bv_after.len();
            if new_len != len_before + 1 {
                return PropertyResult::Fail(format!(
                    "insert({index}, _) produced len {new_len}; expected {}",
                    len_before + 1
                ));
            }
            if bv_after[index] != value {
                return PropertyResult::Fail(format!(
                    "insert({index}, {value}) placed the wrong bit at index {index}"
                ));
            }
            // Bits before `index` must be unchanged from the original seed.
            for i in 0..index {
                if bv_after[i] != seed[i] {
                    return PropertyResult::Fail(format!(
                        "insert({index}, _) corrupted bit {i}: got {} expected {}",
                        bv_after[i], seed[i]
                    ));
                }
            }
            // Bits after `index` must equal the original `seed[i-1]`.
            for i in (index + 1)..new_len {
                if bv_after[i] != seed[i - 1] {
                    return PropertyResult::Fail(format!(
                        "insert({index}, _) corrupted bit {i}: got {} expected {}",
                        bv_after[i],
                        seed[i - 1]
                    ));
                }
            }
            PropertyResult::Pass
        }
        Err(_) => PropertyResult::Fail(format!(
            "insert({index}, _) panicked — must accept any index in 0..=self.len()={len_before}"
        )),
    }
}

/// `leading_ones`/`leading_zeros`/`trailing_ones`/`trailing_zeros` must handle
/// the homogeneous-slice edge case: a slice of N bits that are all the same
/// must report length N, not 0. Detects b08c4bd where the fallback arm of
/// `unwrap_or_default()` returned 0 for an all-ones slice.
pub fn property_leading_trailing_fallback(seed: Vec<bool>, all_ones: bool) -> PropertyResult {
    let n = clamp_len(seed.len().wrapping_add(1));
    let bv: BitVec = if all_ones {
        let mut bv = BitVec::<usize, Lsb0>::with_capacity(n);
        for _ in 0..n {
            bv.push(true);
        }
        bv
    } else {
        let mut bv = BitVec::<usize, Lsb0>::with_capacity(n);
        for _ in 0..n {
            bv.push(false);
        }
        bv
    };
    let bs = bv.as_bitslice();
    let (lead, trail) = if all_ones {
        (bs.leading_ones(), bs.trailing_ones())
    } else {
        (bs.leading_zeros(), bs.trailing_zeros())
    };
    if lead == n && trail == n {
        PropertyResult::Pass
    } else {
        PropertyResult::Fail(format!(
            "homogeneous {}-bit slice (all_ones={all_ones}): leading={lead}, trailing={trail}; expected ({n},{n})",
            n
        ))
    }
}

/// `BitVec::partial_cmp(&BitSlice)` and the symmetric `BitSlice` vs `BitVec`
/// comparison must be inverses of each other. Detects f3d5b43 where the
/// `BitVec: PartialOrd<BitSlice>` impl delegated to `other.partial_cmp(self)`
/// without reversing the result.
pub fn property_bitvec_partial_cmp_matches(a: Vec<bool>, b: Vec<bool>) -> PropertyResult {
    let bv_a = bools_to_bitvec(&a);
    let bv_b = bools_to_bitvec(&b);
    let bs_b: &BitSlice = bv_b.as_bitslice();
    let vec_vs_slice: Option<Ordering> = bv_a.partial_cmp(bs_b);
    let vec_vs_vec: Option<Ordering> = bv_a.partial_cmp(&bv_b);
    if vec_vs_slice == vec_vs_vec {
        PropertyResult::Pass
    } else {
        PropertyResult::Fail(format!(
            "BitVec.partial_cmp(&BitSlice)={vec_vs_slice:?} but BitVec.partial_cmp(&BitVec)={vec_vs_vec:?}",
        ))
    }
}

/// `BitSlice::clone_from_bitslice(&src)` must make `self` bit-equal to `src`
/// (where the two have the same length). Detects 935cad8 where the non-coerce
/// fallback iterated `self.as_bitptr_range()` paired with `self.as_bitptr_range()`
/// instead of `src.as_bitptr_range()`, producing the identity mutation and
/// leaving `self` unchanged.
pub fn property_clone_from_bitslice_copies_src(
    dst_init: Vec<bool>,
    src_init: Vec<bool>,
) -> PropertyResult {
    if dst_init.is_empty() || src_init.is_empty() {
        return PropertyResult::Discard;
    }
    let n = core::cmp::min(dst_init.len(), src_init.len()) % 64 + 1;
    // Use heterogeneous <T,O> on each side so the coerce fast path does not apply.
    let mut dst: BitVec<u8, Lsb0> = BitVec::with_capacity(n);
    for i in 0..n {
        dst.push(dst_init[i % dst_init.len()]);
    }
    let mut src: BitVec<u16, Msb0> = BitVec::with_capacity(n);
    for i in 0..n {
        src.push(src_init[i % src_init.len()]);
    }
    dst.as_mut_bitslice().clone_from_bitslice(src.as_bitslice());
    for i in 0..n {
        let want = src_init[i % src_init.len()];
        if dst[i] != want {
            return PropertyResult::Fail(format!(
                "clone_from_bitslice left bit {i}={} (expected {want}); length {n}",
                dst[i]
            ));
        }
    }
    PropertyResult::Pass
}

/// Formatting a `BitSlice<u64, _>` with the `Octal` trait must emit enough
/// octal digits to cover every bit of every body chunk: a full 64-bit body
/// element needs `ceil(64 / 3) = 22` digits. Detects aeef0be where the
/// pre-allocated buffer used `usize::BITS / 3` without ceiling, so each
/// 64-bit body element rendered only 21 digits.
///
/// Using `BitVec<u64, _>` guarantees a full-usize body chunk (no head/tail
/// enclave), which is the exact shape that trips the dropped ceiling.
pub fn property_octal_fmt_no_panic(n_elems: u8) -> PropertyResult {
    let n_elems = (n_elems as usize) % 3 + 1; // 1..=3 u64 words → 64, 128, or 192 bits
    let n_bits = n_elems * 64;
    let mut bv = BitVec::<u64, Lsb0>::with_capacity(n_bits);
    for i in 0..n_bits {
        bv.push(i % 3 == 0);
    }
    let bs = bv.as_bitslice();
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut buf = String::new();
        write!(&mut buf, "{bs:o}").map(|_| buf)
    }));
    match result {
        Ok(Ok(buf)) => {
            if buf.is_empty() {
                return PropertyResult::Fail(format!(
                    "Octal::fmt produced empty output for n={n_bits}"
                ));
            }
            // Each 64-bit body chunk must contribute at least 22 octal digits.
            // Count octal digits across the whole output; the buggy buffer
            // short-writes one digit per chunk.
            let digits = buf.chars().filter(|c| matches!(*c, '0'..='7')).count();
            let expected_min = n_elems * 22;
            if digits < expected_min {
                PropertyResult::Fail(format!(
                    "Octal::fmt emitted {digits} octal digits for {n_elems} u64 chunks; expected at least {expected_min}"
                ))
            } else {
                PropertyResult::Pass
            }
        }
        Ok(Err(e)) => PropertyResult::Fail(format!("Octal::fmt returned Err: {e}")),
        Err(p) => {
            let msg = p
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| p.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_else(|| "non-string panic".into());
            PropertyResult::Fail(format!("Octal::fmt panicked at n={n_bits}: {msg}"))
        }
    }
}

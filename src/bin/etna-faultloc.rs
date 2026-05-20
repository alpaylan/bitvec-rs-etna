use std::fmt;

use bitvec::etna::{
    property_bitvec_partial_cmp_matches, property_clone_from_bitslice_copies_src,
    property_leading_trailing_fallback, property_octal_fmt_no_panic,
    property_split_at_mut_accepts_len, property_vec_insert_accepts_end, PropertyResult,
};
use crabcheck::profiling::quickcheck;
use crabcheck::quickcheck::{Arbitrary, Mutate};
use rand::Rng;

#[derive(Clone)]
struct Bits(Vec<bool>);
impl fmt::Debug for Bits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<R: Rng> Arbitrary<R> for Bits {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let len = rng.random_range(0..32u32) as usize;
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            out.push(rng.random_bool(0.5));
        }
        Bits(out)
    }
}

impl<R: Rng> Mutate<R> for Bits {
    fn mutate(&self, rng: &mut R, _n: usize) -> Self {
        let mut out = self.0.clone();
        match rng.random_range(0u8..3) {
            0 if !out.is_empty() => {
                let i = rng.random_range(0..out.len());
                out[i] = !out[i];
            }
            1 if out.len() < 32 => out.push(rng.random_bool(0.5)),
            _ if !out.is_empty() => {
                out.pop();
            }
            _ => {}
        }
        Bits(out)
    }
}

fn to_opt(r: PropertyResult) -> Option<bool> {
    match r {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 3 {
        return;
    }
    let result = match (args[1].as_str(), args[2].as_str()) {
        ("crabcheck", "SplitAtMutAcceptsLen") => {
            quickcheck(|(Bits(v), mid_seed): (Bits, usize)| {
                to_opt(property_split_at_mut_accepts_len(v, mid_seed))
            })
        }
        ("crabcheck", "VecInsertAcceptsEnd") => {
            quickcheck(|(Bits(v), b, index_seed): (Bits, bool, usize)| {
                to_opt(property_vec_insert_accepts_end(v, b, index_seed))
            })
        }
        ("crabcheck", "LeadingTrailingFallback") => {
            quickcheck(|(Bits(v), b): (Bits, bool)| {
                to_opt(property_leading_trailing_fallback(v, b))
            })
        }
        ("crabcheck", "BitvecPartialCmpMatches") => {
            quickcheck(|(Bits(a), Bits(b)): (Bits, Bits)| {
                to_opt(property_bitvec_partial_cmp_matches(a, b))
            })
        }
        ("crabcheck", "CloneFromBitsliceCopiesSrc") => {
            quickcheck(|(Bits(a), Bits(b)): (Bits, Bits)| {
                to_opt(property_clone_from_bitslice_copies_src(a, b))
            })
        }
        ("crabcheck", "OctalFmtNoPanic") => quickcheck(|n: usize| {
            to_opt(property_octal_fmt_no_panic(n as u8))
        }),
        (a, b) => panic!("Unknown: {a} {b}"),
    };
    println!("Result: {:?}", result);
}

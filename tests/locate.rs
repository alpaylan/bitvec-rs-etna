//! Fault-localization integration tests for bitvec.
//!
//! One `#[test]` per property in src/bin/etna-faultloc.rs's dispatch.

use bitvec::etna::{
    property_bitvec_partial_cmp_matches, property_clone_from_bitslice_copies_src,
    property_leading_trailing_fallback, property_octal_fmt_no_panic,
    property_split_at_mut_accepts_len, property_vec_insert_accepts_end, PropertyResult,
};
use crabcheck::quickcheck::{Arbitrary, Mutate};
use rand::Rng;
use std::fmt;

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

fn property_split_at_mut_accepts_len_test(input: (Bits, usize)) -> Option<bool> {
    let (Bits(v), mid_seed) = input;
    to_opt(property_split_at_mut_accepts_len(v, mid_seed))
}

fn property_vec_insert_accepts_end_test(input: (Bits, bool, usize)) -> Option<bool> {
    let (Bits(v), b, index_seed) = input;
    to_opt(property_vec_insert_accepts_end(v, b, index_seed))
}

fn property_leading_trailing_fallback_test(input: (Bits, bool)) -> Option<bool> {
    let (Bits(v), b) = input;
    to_opt(property_leading_trailing_fallback(v, b))
}

fn property_bitvec_partial_cmp_matches_test(input: (Bits, Bits)) -> Option<bool> {
    let (Bits(a), Bits(b)) = input;
    to_opt(property_bitvec_partial_cmp_matches(a, b))
}

fn property_clone_from_bitslice_copies_src_test(input: (Bits, Bits)) -> Option<bool> {
    let (Bits(a), Bits(b)) = input;
    to_opt(property_clone_from_bitslice_copies_src(a, b))
}

fn property_octal_fmt_no_panic_test(n: usize) -> Option<bool> {
    to_opt(property_octal_fmt_no_panic(n as u8))
}

// Manual JSON emitter (we don't depend on serde_json in dev-deps).
fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn json_f64(x: f64) -> String {
    if x.is_finite() {
        format!("{}", x)
    } else {
        "null".to_string()
    }
}

fn emit_locate_json(r: &crabcheck::profiling::LocateResult) {
    use crabcheck::quickcheck::ResultStatus;
    let status = match &r.run.status {
        ResultStatus::Failed { .. } => "Failed",
        ResultStatus::Finished => "Finished",
        ResultStatus::GaveUp => "GaveUp",
        ResultStatus::TimedOut => "TimedOut",
        ResultStatus::Aborted { .. } => "Aborted",
    };
    let top = if let Some(s) = r.top() {
        format!(
            "{{\"rank\":{},\"file\":{},\"function\":{},\"start_line\":{},\"end_line\":{},\"ochiai\":{},\"delta\":{},\"panic_overlap\":{},\"confidence\":{},\"confidence_rule\":{}}}",
            s.rank,
            json_escape(&s.region.file),
            json_escape(&s.region.function),
            s.region.start_line,
            s.region.end_line,
            json_f64(s.region.suspiciousness.ochiai as f64),
            json_f64(s.region.delta as f64),
            s.panic_overlap,
            json_escape(&format!("{}", s.confidence)),
            json_escape(s.confidence_rule),
        )
    } else {
        "null".to_string()
    };
    let top_5_items: Vec<String> = r
        .suspects
        .iter()
        .take(5)
        .map(|s| {
            format!(
                "{{\"rank\":{},\"file\":{},\"function\":{},\"start_line\":{},\"end_line\":{},\"confidence\":{},\"confidence_rule\":{},\"panic_overlap\":{}}}",
                s.rank,
                json_escape(&s.region.file),
                json_escape(&s.region.function),
                s.region.start_line,
                s.region.end_line,
                json_escape(&format!("{}", s.confidence)),
                json_escape(s.confidence_rule),
                s.panic_overlap,
            )
        })
        .collect();
    let top_5 = format!("[{}]", top_5_items.join(","));
    let diag_items: Vec<String> = r.diagnostics.iter().map(|d| json_escape(d.tag())).collect();
    let diags = format!("[{}]", diag_items.join(","));
    let out = format!(
        "{{\"status\":{},\"passed\":{},\"discarded\":{},\"n_panics\":{},\"n_suspects\":{},\"top\":{},\"top_5\":{},\"diagnostics\":{}}}",
        json_escape(status),
        r.run.passed,
        r.run.discarded,
        r.n_panics,
        r.suspects.len(),
        top,
        top_5,
        diags,
    );
    println!("@@LOCATE@@ {}", out);
}

#[test]
fn locate_split_at_mut_accepts_len() {
    let report =
        crabcheck::quickcheck_with_locate!(property_split_at_mut_accepts_len_test, "bitvec");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_vec_insert_accepts_end() {
    let report =
        crabcheck::quickcheck_with_locate!(property_vec_insert_accepts_end_test, "bitvec");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_leading_trailing_fallback() {
    let report =
        crabcheck::quickcheck_with_locate!(property_leading_trailing_fallback_test, "bitvec");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_bitvec_partial_cmp_matches() {
    let report =
        crabcheck::quickcheck_with_locate!(property_bitvec_partial_cmp_matches_test, "bitvec");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_clone_from_bitslice_copies_src() {
    let report =
        crabcheck::quickcheck_with_locate!(property_clone_from_bitslice_copies_src_test, "bitvec");
    eprintln!("{report}");
    emit_locate_json(&report);
}

#[test]
fn locate_octal_fmt_no_panic() {
    let report = crabcheck::quickcheck_with_locate!(property_octal_fmt_no_panic_test, "bitvec");
    eprintln!("{report}");
    emit_locate_json(&report);
}

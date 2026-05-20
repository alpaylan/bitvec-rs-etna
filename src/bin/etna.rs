// ETNA workload runner for bitvec.
//
// Usage: cargo run --release --bin etna -- <tool> <property>
//   tool:     etna | proptest | quickcheck | crabcheck | hegel
//   property: SplitAtMutAcceptsLen | VecInsertAcceptsEnd | LeadingTrailingFallback
//             | BitVecPartialCmpMatches | CloneFromBitsliceCopiesSrc | OctalFmtNoPanic
//             | All
//
// Each invocation emits a single JSON line to stdout with:
//   {"status":"passed|failed|aborted","tests":N,"time":"<us>us",...}
// and exits 0 (except for argv parse errors, which exit 2).

use bitvec::etna::{
    property_bitvec_partial_cmp_matches, property_clone_from_bitslice_copies_src,
    property_leading_trailing_fallback, property_octal_fmt_no_panic,
    property_split_at_mut_accepts_len, property_vec_insert_accepts_end, PropertyResult,
};
use crabcheck::quickcheck as crabcheck_qc;
use crabcheck::quickcheck::Arbitrary as CcArbitrary;
use hegel::{generators as hgen, Hegel, Settings as HegelSettings};
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, TestCaseError, TestError, TestRunner};
use quickcheck::{Arbitrary as QcArbitrary, Gen, QuickCheck, ResultStatus, TestResult};
use rand::Rng;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[derive(Default, Clone, Copy)]
struct Metrics {
    inputs: u64,
    elapsed_us: u128,
}

impl Metrics {
    fn combine(self, other: Metrics) -> Metrics {
        Metrics {
            inputs: self.inputs + other.inputs,
            elapsed_us: self.elapsed_us + other.elapsed_us,
        }
    }
}

type Outcome = (Result<(), String>, Metrics);

fn to_err(r: PropertyResult) -> Result<(), String> {
    match r {
        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
        PropertyResult::Fail(m) => Err(m),
    }
}

const ALL_PROPERTIES: &[&str] = &[
    "SplitAtMutAcceptsLen",
    "VecInsertAcceptsEnd",
    "LeadingTrailingFallback",
    "BitVecPartialCmpMatches",
    "CloneFromBitsliceCopiesSrc",
    "OctalFmtNoPanic",
];

fn run_all<F: FnMut(&str) -> Outcome>(mut f: F) -> Outcome {
    let mut total = Metrics::default();
    for p in ALL_PROPERTIES {
        let (r, m) = f(p);
        total = total.combine(m);
        if let Err(e) = r {
            return (Err(e), total);
        }
    }
    (Ok(()), total)
}

fn run_etna_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_etna_property);
    }
    let t0 = Instant::now();
    let result = match property {
        "SplitAtMutAcceptsLen" => to_err(property_split_at_mut_accepts_len(
            vec![false, true, false, true],
            4, // mid_seed % (len+1=5) == 4 == len → boundary case
        )),
        "VecInsertAcceptsEnd" => to_err(property_vec_insert_accepts_end(
            vec![false, false, true],
            true,
            3, // index_seed % (len+1=4) == 3 == len → push-equivalent
        )),
        "LeadingTrailingFallback" => {
            to_err(property_leading_trailing_fallback(vec![true; 5], true))
        }
        "BitVecPartialCmpMatches" => to_err(property_bitvec_partial_cmp_matches(
            vec![false, true],
            vec![true, false],
        )),
        "CloneFromBitsliceCopiesSrc" => to_err(property_clone_from_bitslice_copies_src(
            vec![false; 9],
            vec![true; 9],
        )),
        "OctalFmtNoPanic" => to_err(property_octal_fmt_no_panic(0)),
        _ => return (Err(format!("Unknown property: {property}")), Metrics::default()),
    };
    let elapsed_us = t0.elapsed().as_micros();
    (result, Metrics { inputs: 1, elapsed_us })
}

// ───────────── shared generator: Bits ─────────────
//
// Target shape matches proptest: `vec(any::<bool>(), 0..32)` —
// length uniform in 0..=31, each element 50/50 bool.
#[derive(Clone)]
struct Bits(Vec<bool>);

impl fmt::Debug for Bits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl fmt::Display for Bits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

impl QcArbitrary for Bits {
    fn arbitrary(g: &mut Gen) -> Self {
        let len = g.random_range(0..32u32) as usize;
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            out.push(g.random_range(0..=1u8) == 1);
        }
        Bits(out)
    }
}

impl<R: Rng> CcArbitrary<R> for Bits {
    fn generate(rng: &mut R, _n: usize) -> Self {
        let len = rng.random_range(0..32u32) as usize;
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            out.push(rng.random_bool(0.5));
        }
        Bits(out)
    }
}

// --- Proptest ---

fn bool_vec_strategy() -> BoxedStrategy<Vec<bool>> {
    prop::collection::vec(any::<bool>(), 0..32).boxed()
}

fn run_proptest_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_proptest_property);
    }
    let counter = Arc::new(AtomicU64::new(0));
    let t0 = Instant::now();
    let mut runner = TestRunner::new(ProptestConfig { cases: 40_000_000, ..ProptestConfig::default() });
    let c = counter.clone();
    let result: Result<(), String> = match property {
        "SplitAtMutAcceptsLen" => runner
            .run(&(bool_vec_strategy(), any::<usize>()), move |(seed, mid_seed)| {
                c.fetch_add(1, Ordering::Relaxed);
                let seed_cex = seed.clone();
                match property_split_at_mut_accepts_len(seed, mid_seed) {
                    PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                    PropertyResult::Fail(_) => Err(TestCaseError::fail(format!("({:?} {})", seed_cex, mid_seed))),
                }
            })
            .map_err(|e| match e { TestError::Fail(reason, _) => reason.to_string(), other => other.to_string() }),
        "VecInsertAcceptsEnd" => runner
            .run(&(bool_vec_strategy(), any::<bool>(), any::<usize>()), move |(seed, value, index_seed)| {
                c.fetch_add(1, Ordering::Relaxed);
                let seed_cex = seed.clone();
                match property_vec_insert_accepts_end(seed, value, index_seed) {
                    PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                    PropertyResult::Fail(_) => Err(TestCaseError::fail(format!("({:?} {} {})", seed_cex, value, index_seed))),
                }
            })
            .map_err(|e| match e { TestError::Fail(reason, _) => reason.to_string(), other => other.to_string() }),
        "LeadingTrailingFallback" => runner
            .run(
                &(bool_vec_strategy(), any::<bool>()),
                move |(seed, all_ones)| {
                    c.fetch_add(1, Ordering::Relaxed);
                    let seed_cex = seed.clone();
                    match property_leading_trailing_fallback(seed, all_ones) {
                        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                        PropertyResult::Fail(_) => Err(TestCaseError::fail(format!("({:?} {})", seed_cex, all_ones))),
                    }
                },
            )
            .map_err(|e| match e { TestError::Fail(reason, _) => reason.to_string(), other => other.to_string() }),
        "BitVecPartialCmpMatches" => runner
            .run(&(bool_vec_strategy(), bool_vec_strategy()), move |(a, b)| {
                c.fetch_add(1, Ordering::Relaxed);
                let a_cex = a.clone();
                let b_cex = b.clone();
                match property_bitvec_partial_cmp_matches(a, b) {
                    PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                    PropertyResult::Fail(_) => Err(TestCaseError::fail(format!("({:?} {:?})", a_cex, b_cex))),
                }
            })
            .map_err(|e| match e { TestError::Fail(reason, _) => reason.to_string(), other => other.to_string() }),
        "CloneFromBitsliceCopiesSrc" => runner
            .run(&(bool_vec_strategy(), bool_vec_strategy()), move |(d, s)| {
                c.fetch_add(1, Ordering::Relaxed);
                let d_cex = d.clone();
                let s_cex = s.clone();
                match property_clone_from_bitslice_copies_src(d, s) {
                    PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                    PropertyResult::Fail(_) => Err(TestCaseError::fail(format!("({:?} {:?})", d_cex, s_cex))),
                }
            })
            .map_err(|e| match e { TestError::Fail(reason, _) => reason.to_string(), other => other.to_string() }),
        "OctalFmtNoPanic" => runner
            .run(&any::<u8>(), move |n| {
                c.fetch_add(1, Ordering::Relaxed);
                match property_octal_fmt_no_panic(n) {
                    PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                    PropertyResult::Fail(_) => Err(TestCaseError::fail(format!("({})", n))),
                }
            })
            .map_err(|e| match e { TestError::Fail(reason, _) => reason.to_string(), other => other.to_string() }),
        _ => {
            return (
                Err(format!("Unknown property for proptest: {property}")),
                Metrics::default(),
            );
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = counter.load(Ordering::Relaxed);
    (result, Metrics { inputs, elapsed_us })
}

// --- QuickCheck (fn-pointer, so per-property adapters) ---

static QC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn qc_split_at_mut_accepts_len(Bits(seed): Bits, mid_seed: usize) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_split_at_mut_accepts_len(seed, mid_seed) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_vec_insert_accepts_end(Bits(seed): Bits, value: bool, index_seed: usize) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_vec_insert_accepts_end(seed, value, index_seed) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_leading_trailing_fallback(Bits(seed): Bits, all_ones: bool) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_leading_trailing_fallback(seed, all_ones) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_bitvec_partial_cmp_matches(Bits(a): Bits, Bits(b): Bits) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_bitvec_partial_cmp_matches(a, b) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_clone_from_bitslice_copies_src(Bits(a): Bits, Bits(b): Bits) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_clone_from_bitslice_copies_src(a, b) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_octal_fmt_no_panic(n: u8) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_octal_fmt_no_panic(n) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn run_quickcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_quickcheck_property);
    }
    QC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let mut qc = QuickCheck::new().tests(40_000_000).max_tests(80_000_000);
    let result = match property {
        "SplitAtMutAcceptsLen" => {
            qc.quicktest(qc_split_at_mut_accepts_len as fn(Bits, usize) -> TestResult)
        }
        "VecInsertAcceptsEnd" => {
            qc.quicktest(qc_vec_insert_accepts_end as fn(Bits, bool, usize) -> TestResult)
        }
        "LeadingTrailingFallback" => {
            qc.quicktest(qc_leading_trailing_fallback as fn(Bits, bool) -> TestResult)
        }
        "BitVecPartialCmpMatches" => {
            qc.quicktest(qc_bitvec_partial_cmp_matches as fn(Bits, Bits) -> TestResult)
        }
        "CloneFromBitsliceCopiesSrc" => qc.quicktest(
            qc_clone_from_bitslice_copies_src as fn(Bits, Bits) -> TestResult,
        ),
        "OctalFmtNoPanic" => qc.quicktest(qc_octal_fmt_no_panic as fn(u8) -> TestResult),
        _ => {
            return (
                Err(format!("Unknown property for quickcheck: {property}")),
                Metrics::default(),
            );
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = QC_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match result.status {
        ResultStatus::Finished => Ok(()),
        ResultStatus::Failed { arguments } => Err(format!(
            "({})",
            arguments.join(" ")
        )),
        ResultStatus::Aborted { err } => Err(format!("quickcheck aborted: {err:?}")),
        ResultStatus::TimedOut => Err("quickcheck timed out".to_string()),
        ResultStatus::GaveUp => Err(format!(
            "quickcheck gave up: passed={}, discarded={}",
            result.n_tests_passed, result.n_tests_discarded
        )),
    };
    (status, metrics)
}

// --- Crabcheck ---

static CC_COUNTER: AtomicU64 = AtomicU64::new(0);

fn cc_split_at_mut_accepts_len((Bits(seed), mid_seed): (Bits, usize)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_split_at_mut_accepts_len(seed, mid_seed) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_vec_insert_accepts_end(
    (Bits(seed), v, index_seed): (Bits, bool, usize),
) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_vec_insert_accepts_end(seed, v, index_seed) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_leading_trailing_fallback((Bits(seed), v): (Bits, bool)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_leading_trailing_fallback(seed, v) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_bitvec_partial_cmp_matches((Bits(a), Bits(b)): (Bits, Bits)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_bitvec_partial_cmp_matches(a, b) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_clone_from_bitslice_copies_src((Bits(a), Bits(b)): (Bits, Bits)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_clone_from_bitslice_copies_src(a, b) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_octal_fmt_no_panic(n: u8) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_octal_fmt_no_panic(n) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn run_crabcheck_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_crabcheck_property);
    }
    CC_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let result = match property {
        "SplitAtMutAcceptsLen" => crabcheck_qc::quickcheck(cc_split_at_mut_accepts_len),
        "VecInsertAcceptsEnd" => crabcheck_qc::quickcheck(cc_vec_insert_accepts_end),
        "LeadingTrailingFallback" => crabcheck_qc::quickcheck(cc_leading_trailing_fallback),
        "BitVecPartialCmpMatches" => crabcheck_qc::quickcheck(cc_bitvec_partial_cmp_matches),
        "CloneFromBitsliceCopiesSrc" => {
            crabcheck_qc::quickcheck(cc_clone_from_bitslice_copies_src)
        }
        "OctalFmtNoPanic" => crabcheck_qc::quickcheck(cc_octal_fmt_no_panic),
        _ => {
            return (
                Err(format!("Unknown property for crabcheck: {property}")),
                Metrics::default(),
            );
        }
    };
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = CC_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match result.status {
        crabcheck_qc::ResultStatus::Finished => Ok(()),
        crabcheck_qc::ResultStatus::Failed { arguments } => {
            Err(format!("({})", arguments.join(" ")))
        },
        crabcheck_qc::ResultStatus::TimedOut => Err("crabcheck timed out".to_string()),
        crabcheck_qc::ResultStatus::GaveUp => Err(format!(
            "crabcheck gave up: passed={}, discarded={}",
            result.passed, result.discarded
        )),
        crabcheck_qc::ResultStatus::Aborted { error } => {
            Err(format!("crabcheck aborted: {error}"))
        }
    };
    (status, metrics)
}

// --- Hegel ---

static HG_COUNTER: AtomicU64 = AtomicU64::new(0);

fn hegel_settings() -> HegelSettings {
    HegelSettings::new().test_cases(40_000_000)
}

fn hegel_draw_bool_vec(tc: &hegel::TestCase) -> Vec<bool> {
    // Match proptest: len 0..=31, each element 50/50 bool.
    let len = tc.draw(hgen::integers::<u32>().min_value(0).max_value(31)) as usize;
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        out.push(tc.draw(hgen::integers::<u8>().min_value(0).max_value(1)) == 1);
    }
    out
}

fn run_hegel_property(property: &str) -> Outcome {
    if property == "All" {
        return run_all(run_hegel_property);
    }
    HG_COUNTER.store(0, Ordering::Relaxed);
    let t0 = Instant::now();
    let settings = hegel_settings();
    let run_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match property {
        "SplitAtMutAcceptsLen" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let seed = hegel_draw_bool_vec(&tc);
                // Draw mid_seed across 0..=len so the boundary case (mid == len)
                // is hit ~1/(len+1) of the time, library-faithful style.
                let max_mid = seed.len() as u32;
                let mid_seed =
                    tc.draw(hgen::integers::<u32>().min_value(0).max_value(max_mid)) as usize;
                let seed_cex = seed.clone();
                if let PropertyResult::Fail(_) = property_split_at_mut_accepts_len(seed, mid_seed) {
                    panic!("({:?} {})", seed_cex, mid_seed);
                }
            })
            .settings(settings.clone())
            .run();
        }
        "VecInsertAcceptsEnd" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let seed = hegel_draw_bool_vec(&tc);
                let v = tc.draw(hgen::integers::<u8>().min_value(0).max_value(1)) == 1;
                // Draw index_seed across 0..=len so the push-equivalent case
                // (index == len) is hit ~1/(len+1) of the time.
                let max_index = seed.len() as u32;
                let index_seed = tc.draw(
                    hgen::integers::<u32>().min_value(0).max_value(max_index),
                ) as usize;
                let seed_cex = seed.clone();
                if let PropertyResult::Fail(_) =
                    property_vec_insert_accepts_end(seed, v, index_seed)
                {
                    panic!("({:?} {} {})", seed_cex, v, index_seed);
                }
            })
            .settings(settings.clone())
            .run();
        }
        "LeadingTrailingFallback" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let seed = hegel_draw_bool_vec(&tc);
                let v = tc.draw(hgen::integers::<u8>().min_value(0).max_value(1)) == 1;
                let seed_cex = seed.clone();
                if let PropertyResult::Fail(_) = property_leading_trailing_fallback(seed, v) {
                    panic!("({:?} {})", seed_cex, v);
                }
            })
            .settings(settings.clone())
            .run();
        }
        "BitVecPartialCmpMatches" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let a = hegel_draw_bool_vec(&tc);
                let b = hegel_draw_bool_vec(&tc);
                let a_cex = a.clone();
                let b_cex = b.clone();
                if let PropertyResult::Fail(_) = property_bitvec_partial_cmp_matches(a, b) {
                    panic!("({:?} {:?})", a_cex, b_cex);
                }
            })
            .settings(settings.clone())
            .run();
        }
        "CloneFromBitsliceCopiesSrc" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let d = hegel_draw_bool_vec(&tc);
                let s = hegel_draw_bool_vec(&tc);
                let d_cex = d.clone();
                let s_cex = s.clone();
                if let PropertyResult::Fail(_) = property_clone_from_bitslice_copies_src(d, s) {
                    panic!("({:?} {:?})", d_cex, s_cex);
                }
            })
            .settings(settings.clone())
            .run();
        }
        "OctalFmtNoPanic" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let n = tc.draw(hgen::integers::<u8>());
                if let PropertyResult::Fail(_) = property_octal_fmt_no_panic(n) {
                    panic!("({})", n);
                }
            })
            .settings(settings.clone())
            .run();
        }
        _ => panic!("__unknown_property:{property}"),
    }));
    let elapsed_us = t0.elapsed().as_micros();
    let inputs = HG_COUNTER.load(Ordering::Relaxed);
    let metrics = Metrics { inputs, elapsed_us };
    let status = match run_result {
        Ok(()) => Ok(()),
        Err(e) => {
            let msg = if let Some(s) = e.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = e.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "hegel panicked with non-string payload".to_string()
            };
            if let Some(rest) = msg.strip_prefix("__unknown_property:") {
                return (
                    Err(format!("Unknown property for hegel: {rest}")),
                    Metrics::default(),
                );
            }
            Err(msg.strip_prefix("Property test failed: ").unwrap_or(&msg).to_string())
        }
    };
    (status, metrics)
}

fn run(tool: &str, property: &str) -> Outcome {
    match tool {
        "etna" => run_etna_property(property),
        "proptest" => run_proptest_property(property),
        "quickcheck" => run_quickcheck_property(property),
        "crabcheck" => run_crabcheck_property(property),
        "hegel" => run_hegel_property(property),
        _ => (Err(format!("Unknown tool: {tool}")), Metrics::default()),
    }
}

fn json_str(s: &str) -> String {
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

fn emit_json(
    tool: &str,
    property: &str,
    status: &str,
    metrics: Metrics,
    counterexample: Option<&str>,
    error: Option<&str>,
) {
    let cex = counterexample.map_or("null".to_string(), json_str);
    let err = error.map_or("null".to_string(), json_str);
    println!(
        "{{\"status\":{},\"tests\":{},\"discards\":0,\"time\":{},\"counterexample\":{},\"error\":{},\"tool\":{},\"property\":{}}}",
        json_str(status),
        metrics.inputs,
        json_str(&format!("{}us", metrics.elapsed_us)),
        cex,
        err,
        json_str(tool),
        json_str(property),
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <tool> <property>", args[0]);
        eprintln!("Tools: etna | proptest | quickcheck | crabcheck | hegel");
        eprintln!("Properties: {}", ALL_PROPERTIES.join(" | "));
        std::process::exit(2);
    }
    let (tool, property) = (args[1].as_str(), args[2].as_str());

    // Silence library-under-test panic noise (frameworks catch panics internally
    // but the default hook still prints "thread 'main' panicked at ..." to stderr).
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(tool, property)));
    std::panic::set_hook(previous_hook);

    let (result, metrics) = match caught {
        Ok(outcome) => outcome,
        Err(payload) => {
            let msg = if let Some(s) = payload.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = payload.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "panic with non-string payload".to_string()
            };
            emit_json(
                tool,
                property,
                "aborted",
                Metrics::default(),
                None,
                Some(&format!("adapter panic: {msg}")),
            );
            return;
        }
    };

    match result {
        Ok(()) => emit_json(tool, property, "passed", metrics, None, None),
        Err(msg) => emit_json(tool, property, "failed", metrics, Some(&msg), None),
    }
}

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
use hegel::{generators as hgen, Hegel, Settings as HegelSettings};
use proptest::prelude::*;
use proptest::test_runner::{Config as ProptestConfig, TestCaseError, TestRunner};
use quickcheck::{QuickCheck, ResultStatus, TestResult};
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
        "SplitAtMutAcceptsLen" => to_err(property_split_at_mut_accepts_len(vec![
            false, true, false, true,
        ])),
        "VecInsertAcceptsEnd" => {
            to_err(property_vec_insert_accepts_end(vec![false, false, true], true))
        }
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
    let mut runner = TestRunner::new(ProptestConfig::default());
    let c = counter.clone();
    let result: Result<(), String> = match property {
        "SplitAtMutAcceptsLen" => runner
            .run(&bool_vec_strategy(), move |seed| {
                c.fetch_add(1, Ordering::Relaxed);
                match property_split_at_mut_accepts_len(seed) {
                    PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                    PropertyResult::Fail(m) => Err(TestCaseError::fail(m)),
                }
            })
            .map_err(|e| e.to_string()),
        "VecInsertAcceptsEnd" => runner
            .run(&(bool_vec_strategy(), any::<bool>()), move |(seed, value)| {
                c.fetch_add(1, Ordering::Relaxed);
                match property_vec_insert_accepts_end(seed, value) {
                    PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                    PropertyResult::Fail(m) => Err(TestCaseError::fail(m)),
                }
            })
            .map_err(|e| e.to_string()),
        "LeadingTrailingFallback" => runner
            .run(
                &(bool_vec_strategy(), any::<bool>()),
                move |(seed, all_ones)| {
                    c.fetch_add(1, Ordering::Relaxed);
                    match property_leading_trailing_fallback(seed, all_ones) {
                        PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                        PropertyResult::Fail(m) => Err(TestCaseError::fail(m)),
                    }
                },
            )
            .map_err(|e| e.to_string()),
        "BitVecPartialCmpMatches" => runner
            .run(&(bool_vec_strategy(), bool_vec_strategy()), move |(a, b)| {
                c.fetch_add(1, Ordering::Relaxed);
                match property_bitvec_partial_cmp_matches(a, b) {
                    PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                    PropertyResult::Fail(m) => Err(TestCaseError::fail(m)),
                }
            })
            .map_err(|e| e.to_string()),
        "CloneFromBitsliceCopiesSrc" => runner
            .run(&(bool_vec_strategy(), bool_vec_strategy()), move |(d, s)| {
                c.fetch_add(1, Ordering::Relaxed);
                match property_clone_from_bitslice_copies_src(d, s) {
                    PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                    PropertyResult::Fail(m) => Err(TestCaseError::fail(m)),
                }
            })
            .map_err(|e| e.to_string()),
        "OctalFmtNoPanic" => runner
            .run(&any::<u8>(), move |n| {
                c.fetch_add(1, Ordering::Relaxed);
                match property_octal_fmt_no_panic(n) {
                    PropertyResult::Pass | PropertyResult::Discard => Ok(()),
                    PropertyResult::Fail(m) => Err(TestCaseError::fail(m)),
                }
            })
            .map_err(|e| e.to_string()),
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

// Deterministically expand a u64 seed into a bounded Vec<bool>.
fn seed_to_bool_vec(seed: u64) -> Vec<bool> {
    let len = (seed % 32) as usize;
    let mut out = Vec::with_capacity(len);
    let mut s = seed.rotate_right(5);
    for _ in 0..len {
        out.push(s & 1 == 1);
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
    }
    out
}

fn qc_split_at_mut_accepts_len(seed: u64) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_split_at_mut_accepts_len(seed_to_bool_vec(seed)) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_vec_insert_accepts_end(seed: u64, value: bool) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_vec_insert_accepts_end(seed_to_bool_vec(seed), value) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_leading_trailing_fallback(seed: u64, all_ones: bool) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_leading_trailing_fallback(seed_to_bool_vec(seed), all_ones) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_bitvec_partial_cmp_matches(a: u64, b: u64) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_bitvec_partial_cmp_matches(seed_to_bool_vec(a), seed_to_bool_vec(b)) {
        PropertyResult::Pass => TestResult::passed(),
        PropertyResult::Discard => TestResult::discard(),
        PropertyResult::Fail(_) => TestResult::failed(),
    }
}

fn qc_clone_from_bitslice_copies_src(a: u64, b: u64) -> TestResult {
    QC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_clone_from_bitslice_copies_src(seed_to_bool_vec(a), seed_to_bool_vec(b)) {
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
    let mut qc = QuickCheck::new().tests(200).max_tests(2000);
    let result = match property {
        "SplitAtMutAcceptsLen" => {
            qc.quicktest(qc_split_at_mut_accepts_len as fn(u64) -> TestResult)
        }
        "VecInsertAcceptsEnd" => {
            qc.quicktest(qc_vec_insert_accepts_end as fn(u64, bool) -> TestResult)
        }
        "LeadingTrailingFallback" => {
            qc.quicktest(qc_leading_trailing_fallback as fn(u64, bool) -> TestResult)
        }
        "BitVecPartialCmpMatches" => {
            qc.quicktest(qc_bitvec_partial_cmp_matches as fn(u64, u64) -> TestResult)
        }
        "CloneFromBitsliceCopiesSrc" => qc.quicktest(
            qc_clone_from_bitslice_copies_src as fn(u64, u64) -> TestResult,
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
            "quickcheck failed with counterexample: ({})",
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

fn cc_seed_to_bool_vec(n: usize) -> Vec<bool> {
    let len = n % 32;
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        out.push((n.wrapping_mul(2654435761).wrapping_add(i)) & 1 == 1);
    }
    out
}

fn cc_split_at_mut_accepts_len(n: usize) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_split_at_mut_accepts_len(cc_seed_to_bool_vec(n)) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_vec_insert_accepts_end((n, v): (usize, usize)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_vec_insert_accepts_end(cc_seed_to_bool_vec(n), v & 1 == 1) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_leading_trailing_fallback((n, v): (usize, usize)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_leading_trailing_fallback(cc_seed_to_bool_vec(n), v & 1 == 1) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_bitvec_partial_cmp_matches((a, b): (usize, usize)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_bitvec_partial_cmp_matches(cc_seed_to_bool_vec(a), cc_seed_to_bool_vec(b)) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_clone_from_bitslice_copies_src((a, b): (usize, usize)) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_clone_from_bitslice_copies_src(cc_seed_to_bool_vec(a), cc_seed_to_bool_vec(b)) {
        PropertyResult::Pass => Some(true),
        PropertyResult::Fail(_) => Some(false),
        PropertyResult::Discard => None,
    }
}

fn cc_octal_fmt_no_panic(n: usize) -> Option<bool> {
    CC_COUNTER.fetch_add(1, Ordering::Relaxed);
    match property_octal_fmt_no_panic(n as u8) {
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
        crabcheck_qc::ResultStatus::Failed { arguments } => Err(format!(
            "crabcheck failed with counterexample: ({})",
            arguments.join(" ")
        )),
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
    HegelSettings::new().test_cases(200).seed(Some(0xB17_EC_1))
}

fn hegel_draw_bool_vec(tc: &hegel::TestCase) -> Vec<bool> {
    let len = tc.draw(hgen::integers::<u8>()) as usize % 32;
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        out.push(tc.draw(hgen::integers::<u8>()) & 1 == 1);
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
                if let PropertyResult::Fail(m) = property_split_at_mut_accepts_len(seed) {
                    panic!("{m}");
                }
            })
            .settings(settings.clone())
            .run();
        }
        "VecInsertAcceptsEnd" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let seed = hegel_draw_bool_vec(&tc);
                let v = tc.draw(hgen::integers::<u8>()) & 1 == 1;
                if let PropertyResult::Fail(m) = property_vec_insert_accepts_end(seed, v) {
                    panic!("{m}");
                }
            })
            .settings(settings.clone())
            .run();
        }
        "LeadingTrailingFallback" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let seed = hegel_draw_bool_vec(&tc);
                let v = tc.draw(hgen::integers::<u8>()) & 1 == 1;
                if let PropertyResult::Fail(m) = property_leading_trailing_fallback(seed, v) {
                    panic!("{m}");
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
                if let PropertyResult::Fail(m) = property_bitvec_partial_cmp_matches(a, b) {
                    panic!("{m}");
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
                if let PropertyResult::Fail(m) = property_clone_from_bitslice_copies_src(d, s) {
                    panic!("{m}");
                }
            })
            .settings(settings.clone())
            .run();
        }
        "OctalFmtNoPanic" => {
            Hegel::new(|tc: hegel::TestCase| {
                HG_COUNTER.fetch_add(1, Ordering::Relaxed);
                let n = tc.draw(hgen::integers::<u32>()) as u8;
                if let PropertyResult::Fail(m) = property_octal_fmt_no_panic(n) {
                    panic!("{m}");
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
            Err(format!("hegel found counterexample: {msg}"))
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

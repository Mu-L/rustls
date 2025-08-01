use std::sync::Arc;

use fxhash::FxHashMap;
use itertools::Itertools;
use rustls_test::KeyType;

use crate::Side;
use crate::callgrind::InstructionCounts;

/// Validates a benchmark collection, returning an error if the provided benchmarks are invalid
///
/// Benchmarks can be invalid because of the following reasons:
///
/// - Re-using an already defined benchmark name.
pub(crate) fn validate_benchmarks(benchmarks: &[Benchmark]) -> anyhow::Result<()> {
    // Detect duplicate definitions
    let duplicate_names: Vec<_> = benchmarks
        .iter()
        .map(|b| b.name.as_str())
        .duplicates()
        .collect();
    if !duplicate_names.is_empty() {
        anyhow::bail!(
            "The following benchmarks are defined multiple times: {}",
            duplicate_names.join(", ")
        );
    }

    Ok(())
}

/// Get the reported instruction counts for the provided benchmark
pub(crate) fn get_reported_instr_count(
    bench: &Benchmark,
    results: &FxHashMap<&str, InstructionCounts>,
) -> InstructionCounts {
    results[&bench.name()]
}

/// Specifies which functionality is being benchmarked
#[derive(Copy, Clone)]
pub enum BenchmarkKind {
    /// Perform the handshake and exit
    Handshake(ResumptionKind),
    /// Perform the handshake and transfer 1MB of data
    Transfer,
}

impl BenchmarkKind {
    /// Returns the [`ResumptionKind`] used in the handshake part of the benchmark
    pub fn resumption_kind(self) -> ResumptionKind {
        match self {
            Self::Handshake(kind) => kind,
            Self::Transfer => ResumptionKind::No,
        }
    }
}

/// The kind of resumption used during the handshake
#[derive(PartialEq, Clone, Copy)]
pub enum ResumptionKind {
    /// No resumption
    No,
    /// Session ID
    SessionId,
    /// Session tickets
    Tickets,
}

impl ResumptionKind {
    pub const ALL: &'static [Self] = &[Self::No, Self::SessionId, Self::Tickets];

    /// Returns a user-facing label that identifies the resumption kind
    pub fn label(&self) -> &'static str {
        match *self {
            Self::No => "no_resume",
            Self::SessionId => "session_id",
            Self::Tickets => "tickets",
        }
    }
}

/// Parameters associated to a benchmark
#[derive(Clone, Debug)]
pub struct BenchmarkParams {
    /// Which `CryptoProvider` to test
    pub provider: rustls::crypto::CryptoProvider,
    /// How to make a suitable [`rustls::server::ProducesTickets`].
    pub ticketer: &'static fn() -> Arc<dyn rustls::server::ProducesTickets>,
    /// Where to get keys for server auth
    pub auth_key: AuthKeySource,
    /// Cipher suite
    pub ciphersuite: rustls::SupportedCipherSuite,
    /// TLS version
    pub version: &'static rustls::SupportedProtocolVersion,
    /// A user-facing label that identifies these params
    pub label: String,
}

impl BenchmarkParams {
    /// Create a new set of benchmark params
    pub const fn new(
        provider: rustls::crypto::CryptoProvider,
        ticketer: &'static fn() -> Arc<dyn rustls::server::ProducesTickets>,
        auth_key: AuthKeySource,
        ciphersuite: rustls::SupportedCipherSuite,
        version: &'static rustls::SupportedProtocolVersion,
        label: String,
    ) -> Self {
        Self {
            provider,
            ticketer,
            auth_key,
            ciphersuite,
            version,
            label,
        }
    }
}

#[derive(Clone, Debug)]
pub enum AuthKeySource {
    KeyType(KeyType),
    FuzzingProvider,
}

/// A benchmark specification
pub struct Benchmark {
    /// The name of the benchmark, as shown in the benchmark results
    name: String,
    /// The benchmark kind
    pub kind: BenchmarkKind,
    /// The benchmark's parameters
    pub params: BenchmarkParams,
}

impl Benchmark {
    /// Create a new benchmark
    pub fn new(name: String, kind: BenchmarkKind, params: BenchmarkParams) -> Self {
        Self { name, kind, params }
    }

    /// Returns the benchmark's unique name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the benchmark's unique name with the side appended to it
    pub fn name_with_side(&self, side: Side) -> String {
        format!("{}_{}", self.name, side.as_str())
    }
}

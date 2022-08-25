/// The input to a `cpuid` invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CpuidQuery {
    pub leaf: u32,
    pub subleaf: u32,
}

/// Simple queries do not require a subleaf.
impl From<u32> for CpuidQuery {
    fn from(leaf: u32) -> Self {
        Self { leaf, subleaf: 0 }
    }
}

/// The result of a `cpuid` invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuidResult {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

/// A trait for data structures that can be queried for CPUID or feature MSR information.
pub trait CpuInformation {
    /// Return the result of a `cpuid` invocation.
    ///
    /// Returns `None` if the result is unknown.
    fn cpuid(&self, query: CpuidQuery) -> Option<CpuidResult>;

    /// Return the result of a `rdmsr` invocation.
    ///
    /// Returns `None` if the result is unknown.
    fn rdmsr(&self, index: u32) -> Option<u64>;
}

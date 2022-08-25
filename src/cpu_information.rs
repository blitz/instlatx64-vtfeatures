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

/// The registers of a [CpuidResult].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuidRegister {
    Eax,
    Ebx,
    Ecx,
    Edx,
}

impl CpuidResult {
    /// Retrieve a register value from a CPUID result.
    pub fn get(&self, reg: CpuidRegister) -> u32 {
        match reg {
            CpuidRegister::Eax => self.eax,
            CpuidRegister::Ebx => self.ebx,
            CpuidRegister::Ecx => self.ecx,
            CpuidRegister::Edx => self.edx,
        }
    }
}

/// Converts a slice of 32-bit little-endian integers into a
/// `Vec<u8>`. This also trims zero bytes at the end.
fn dwords_to_bytes(dwords: &[u32]) -> Vec<u8> {
    dwords
        .iter()
        .flat_map(|dw| dw.to_le_bytes())
        .take_while(|c| *c != 0)
        .collect()
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

    /// The maximum supported standard (`0x0000_xxxx`) CPUID leaf.
    fn max_standard_leaf(&self) -> u32 {
        self.cpuid(0.into()).map(|r| r.eax).unwrap_or(0)
    }

    /// The maximum supported extended (`0x8000_xxxx`) CPUID leaf.
    fn max_extended_leaf(&self) -> u32 {
        self.cpuid(0x8000_0000.into())
            .map(|r| r.eax)
            .unwrap_or(0x8000_0000)
    }

    /// Returns the vendor string as raw bytes.
    fn vendor_bytes(&self) -> Option<Vec<u8>> {
        self.cpuid(0.into())
            .map(|r| -> Vec<u8> { dwords_to_bytes(&[r.ebx, r.edx, r.ecx]) })
    }

    /// Returns the vendor name as string.
    ///
    /// This uses lossy conversion to UTF-8 in case the string is not
    /// valid UTF-8.
    fn vendor_name(&self) -> Option<String> {
        self.vendor_bytes()
            .map(|b| -> String { String::from_utf8_lossy(&b).into_owned() })
    }

    /// The CPU model string as raw bytes.
    fn model_bytes(&self) -> Option<Vec<u8>> {
        if self.max_extended_leaf() < 0x8000_0004 {
            return None;
        }

        let r1 = self.cpuid(0x8000_0002.into())?;
        let r2 = self.cpuid(0x8000_0003.into())?;
        let r3 = self.cpuid(0x8000_0004.into())?;

        Some(dwords_to_bytes(&[
            r1.eax, r1.ebx, r1.ecx, r1.edx, r2.eax, r2.ebx, r2.ecx, r2.edx, r3.eax, r3.ebx, r3.ecx,
            r3.edx,
        ]))
    }

    /// Returns the model name as string.
    ///
    /// This uses lossy conversion to UTF-8 in case the string is not
    /// valid UTF-8.
    fn model_name(&self) -> Option<String> {
        self.model_bytes()
            .map(|b| -> String { String::from_utf8_lossy(&b).into_owned() })
    }
}

#[macro_use]
extern crate lazy_static;

use aida_parse::AidaCpuidDump;
use cpu_information::CpuInformation;
use std::error;
use std::io;
use std::io::Read;
use std::str::FromStr;

mod aida_parse;
mod cpu_information;
mod features;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MsrMatch {
    index: u32,
    must_be_set: u64,
}

#[derive(Debug, Clone, Copy)]
struct Feature {
    name: &'static str,
    must_match: &'static [MsrMatch],
}

fn does_match(cpu_info: &impl CpuInformation, msr_match: &MsrMatch) -> Option<bool> {
    cpu_info
        .rdmsr(msr_match.index)
        .map(|val| (val & msr_match.must_be_set) == msr_match.must_be_set)
}

// Checks whether a feature is available. The answer might be unknown,
// if the relevant MSRs are not available.
fn has_feature(cpu_info: &impl CpuInformation, feature: &Feature) -> Option<bool> {
    feature
        .must_match
        .iter()
        .map(|m| does_match(cpu_info, m))
        .fold(Some(true), |acc, n| acc.and_then(|b| n.map(|c| b && c)))
}

fn tristate_to_char(tristate: Option<bool>) -> char {
    match tristate {
        Some(b) => {
            if b {
                'Y'
            } else {
                'N'
            }
        }
        None => '?',
    }
}

static FEATURES: &[Feature] = &[
    Feature {
        name: "EPT                         ",
        must_match: &[MsrMatch {
            index: 0x48b,
            must_be_set: 1 << (32 + 1),
        }],
    },
    Feature {
        name: "Unrestricted Guest          ",
        must_match: &[MsrMatch {
            index: 0x48b,
            must_be_set: 1 << (32 + 7),
        }],
    },
    Feature {
        name: "VMCS Shadowing              ",
        must_match: &[MsrMatch {
            index: 0x48b,
            must_be_set: 1 << 46,
        }],
    },
    Feature {
        name: "APIC-register virtualization",
        must_match: &[MsrMatch {
            index: 0x48b,
            must_be_set: 1 << 40,
        }],
    },
    Feature {
        name: "Virtual-interrupt delivery  ",
        must_match: &[MsrMatch {
            index: 0x48b,
            must_be_set: 1 << 41,
        }],
    },
    Feature {
        name: "VMX Preemption Timer        ",
        must_match: &[MsrMatch {
            index: 0x481,
            must_be_set: 1 << (6 + 32),
        }],
    },
    Feature {
        name: "Process posted interrupts   ",
        must_match: &[MsrMatch {
            index: 0x481,
            must_be_set: 1 << (7 + 32),
        }],
    },
];

fn main() -> Result<()> {
    let mut input_bytes = Vec::new();
    io::stdin().read_to_end(&mut input_bytes)?;

    let input_string: String = String::from_utf8(input_bytes)?;

    let aida_result = AidaCpuidDump::from_str(&input_string)?;

    let unknown = "Unknown".to_owned();

    println!(
        "{} {}\n",
        aida_result.vendor_name().unwrap_or_else(|| unknown.clone()),
        aida_result.model_name().unwrap_or(unknown),
    );

    for feature in FEATURES {
        println!(
            "{}: {}",
            feature.name,
            tristate_to_char(has_feature(&aida_result, feature))
        );
    }

    Ok(())
}

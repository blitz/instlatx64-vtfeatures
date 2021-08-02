#[macro_use]
extern crate lazy_static;

use regex::Regex;
use std::error;
use std::io;
use std::io::BufRead;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
struct MsrValue {
    index: u32,
    value: u64,
}

// Parse lines of the following form into `MsrValue` structs:
// MSR 0000048B: 025F-7FFF-0000-0000
//
// If parsing is unsuccessful, `None` is returned.
fn try_parse_msr_line(s: &str) -> Option<MsrValue> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"^MSR ([0-9a-fA-F]+): ([-0-9a-fA-F]+)$").expect("a valid regex");
    }

    RE.captures(s)
        .map(|c| {
            Some(MsrValue {
                index: u32::from_str_radix(c.get(1).unwrap().as_str(), 16).ok()?,

                // Values have to have their hyphens removed for parsing.
                value: u64::from_str_radix(
                    &c.get(2)
                        .unwrap()
                        .as_str()
                        .chars()
                        .filter(|&c| c != '-')
                        .collect::<String>(),
                    16,
                )
                .ok()?,
            })
        })
        .flatten()
}

struct MsrMatch {
    index: u32,
    must_be_set: u64,
}

struct Feature {
    name: &'static str,
    must_match: &'static [MsrMatch],
}

fn does_match(msr_values: &[MsrValue], msr_match: &MsrMatch) -> Option<bool> {
    msr_values
        .iter()
        .find(|&m| m.index == msr_match.index)
        .map(|m| (m.value & msr_match.must_be_set) == msr_match.must_be_set)
}

// Checks whether a feature is available. The answer might be unknown,
// if the relevant MSRs are not available.
fn has_feature(msr_values: &[MsrValue], feature: &Feature) -> Option<bool> {
    feature
        .must_match
        .iter()
        .map(|m| does_match(msr_values, m))
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
        name: "Process posted interrupts   ",
        must_match: &[MsrMatch {
            index: 0x481,
            must_be_set: 1 << (7 + 32),
        }],
    },
];

fn main() -> Result<()> {
    let msr_values = io::stdin()
        .lock()
        .lines()
        .collect::<io::Result<Vec<String>>>()?
        .iter()
        .filter_map(|l| try_parse_msr_line(l))
        .collect::<Vec<MsrValue>>();

    for feature in FEATURES {
        println!(
            "{}: {}",
            feature.name,
            tristate_to_char(has_feature(&msr_values, feature))
        );
    }

    Ok(())
}

//! # Parse AIDA CPUID Dumps
//!
//! Extract CPUID and MSR information out of AIDA CPUID dumps. This
//! code only interprets CPUID values from logical CPU 0. It also
//! ignores any duplicated MSRs in the input data. From manual
//! inspection, the duplicated MSRs are performance counters and not
//! interesting.
//!
//! See [AidaCpuidDump].

pub use std::collections::BTreeMap as Map;
use std::{collections::BTreeSet as Set, str::FromStr};

use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CpuidQuery {
    pub leaf: u32,
    pub subleaf: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CpuidResult {
    pub eax: u32,
    pub ebx: u32,
    pub ecx: u32,
    pub edx: u32,
}

#[derive(Debug, Clone)]
pub struct AidaCpuidDump {
    pub cpuid: Map<CpuidQuery, CpuidResult>,
    pub msrs: Map<u32, u64>,
}

#[derive(Debug, Clone)]
pub struct ParseAidaCpuidDumpError {}

impl std::fmt::Display for ParseAidaCpuidDumpError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Failed to parse AIDA CPUID dump")
    }
}

/// Low-level representation of a single input line after first
/// parsing round.
#[derive(Debug, Clone, PartialEq)]
enum InputLine {
    /// A header in the input.
    ///
    /// For example, `------[ Logical CPU #0 ]------` would be parsed
    /// as group header with name `Logical CPU #0`.
    GroupHeader { name: String },

    /// A CPUID line in the input.
    Cpuid {
        query: CpuidQuery,
        result: CpuidResult,
    },

    /// A MSR line in the input.
    Msr { index: u32, value: u64 },
}

/// Parse a group header line or return [None].
fn try_match_group_header(input: &str) -> Option<InputLine> {
    lazy_static! {
        static ref GROUP_HEADER_RE: Regex =
            Regex::new(r"^------\[ (.+) ]------$").expect("a valid regex");
    }

    let matches = GROUP_HEADER_RE.captures(input)?;

    Some(InputLine::GroupHeader {
        name: matches
            .get(1)
            .expect("capture group populated after match")
            .as_str()
            .to_owned(),
    })
}

/// Parse a hex string to an `u32`.
///
/// This function will ignore any dashes in the input.
///
/// **Note:** This function assumes correct input and will panic if
/// the string cannot be parsed.
fn hex_as_u32(input: &str) -> u32 {
    u32::from_str_radix(&input.chars().filter(|&c| c != '-').collect::<String>(), 16)
        .expect("can't parse input after regex matched")
}

/// Parse a hex string to an `u64`.
///
/// This function has the same requirements and limitations as
/// [hex_as_u32]. See there for details.
fn hex_as_u64(input: &str) -> u64 {
    u64::from_str_radix(&input.chars().filter(|&c| c != '-').collect::<String>(), 16)
        .expect("can't parse input after regex matched")
}

/// Parse a CPUID line or return [None].
fn try_match_cpuid(input: &str) -> Option<InputLine> {
    lazy_static! {
        static ref CPUID_RE: Regex =
            Regex::new(r"^CPUID ([0-9a-fA-F]+): ([0-9a-fA-F]{8})-([0-9a-fA-F]{8})-([0-9a-fA-F]{8})-([0-9a-fA-F]{8})(?: \[SL ([0-9a-fA-F]{2})\]|.*)$").expect("a valid regex");
    }

    let matches = CPUID_RE.captures(input)?;

    Some(InputLine::Cpuid {
        query: CpuidQuery {
            leaf: hex_as_u32(matches.get(1).expect("CPUID leaf match").as_str()),
            subleaf: matches.get(6).map(|m| hex_as_u32(m.as_str())).unwrap_or(0),
        },

        result: CpuidResult {
            eax: hex_as_u32(matches.get(2).expect("CPUID eax match").as_str()),
            ebx: hex_as_u32(matches.get(3).expect("CPUID ebx match").as_str()),
            ecx: hex_as_u32(matches.get(4).expect("CPUID ecx match").as_str()),
            edx: hex_as_u32(matches.get(5).expect("CPUID edx match").as_str()),
        },
    })
}

/// Parse a MSR line or return [None].
fn try_match_msr(input: &str) -> Option<InputLine> {
    lazy_static! {
        static ref MSR_RE: Regex =
            Regex::new(r"^MSR ([0-9a-fA-F]+): ([-0-9a-fA-F]{19}).*$").expect("a valid regex");
    }

    let matches = MSR_RE.captures(input)?;

    Some(InputLine::Msr {
        index: hex_as_u32(matches.get(1).expect("MSR index match").as_str()),
        value: hex_as_u64(matches.get(2).expect("MSR value match").as_str()),
    })
}

impl FromStr for InputLine {
    type Err = ParseAidaCpuidDumpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        try_match_group_header(s)
            .or_else(|| try_match_cpuid(s))
            .or_else(|| try_match_msr(s))
            .ok_or(ParseAidaCpuidDumpError {})
    }
}

impl FromStr for AidaCpuidDump {
    type Err = ParseAidaCpuidDumpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // We first parse each line of the input. Non-matching lines
        // are discarded.
        let parsed_lines = s
            .lines()
            .filter_map(|line| -> Option<InputLine> { InputLine::from_str(line).ok() });

        // Now we fold each input into the groups they belong to. For
        // this we use a list of groups plus their content as
        // accumulator. Everything before the first group header
        // belongs to the unnamed group.
        let groups_vec: Vec<(String, Vec<InputLine>)> =
            parsed_lines.fold(vec![("".to_string(), vec![])], |mut acc, line| {
                if let InputLine::GroupHeader { name } = line {
                    // Start a new group.
                    acc.push((name, vec![]));
                } else {
                    // Extend the last group.
                    acc.last_mut().expect("at least one item").1.push(line);
                }

                acc
            });

        // Time for some sanity checking.

        if groups_vec.len()
            != groups_vec
                .iter()
                .map(|(k, _v)| k)
                .collect::<Set<&String>>()
                .len()
        {
            // Duplicate group names.
            //
            // TODO Better errors.
            return Err(ParseAidaCpuidDumpError {});
        }

        // Turn the parsed groups into an easy-to-query map.
        let groups: Map<String, Vec<InputLine>> = groups_vec.into_iter().collect();

        // Construct our final return value.
        Ok(AidaCpuidDump {
            cpuid: groups
                .get("Logical CPU #0")
                .ok_or(ParseAidaCpuidDumpError {})?
                .iter()
                .filter_map(|line| {
                    if let InputLine::Cpuid { query, result } = line {
                        Some((*query, *result))
                    } else {
                        None
                    }
                })
                .collect(),
            msrs: groups
                .get("MSR Registers")
                .ok_or(ParseAidaCpuidDumpError {})?
                .iter()
                .filter_map(|line| {
                    if let InputLine::Msr { index, value } = line {
                        Some((*index, *value))
                    } else {
                        None
                    }
                })
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_numbers() {
        assert_eq!(hex_as_u32("65746E49"), 0x65746E49);
        assert_eq!(hex_as_u32("FFF9-FFFE"), 0xFFF9FFFE);

        assert_eq!(hex_as_u64("FFF9FFFE04006172"), 0xFFF9_FFFE_0400_6172);
        assert_eq!(hex_as_u64("FFF9-FFFE-0400-6172"), 0xFFF9_FFFE_0400_6172);
    }

    #[test]
    fn group_headers_are_recognized() {
        assert_eq!(
            try_match_group_header("------[ Logical CPU #0 ]------"),
            Some(InputLine::GroupHeader {
                name: "Logical CPU #0".to_string()
            })
        );

        assert_eq!(
            try_match_group_header("------[ MSR Registers ]------"),
            Some(InputLine::GroupHeader {
                name: "MSR Registers".to_string()
            })
        );

        assert_eq!(try_match_group_header(""), None);
        assert_eq!(try_match_group_header("Random other input"), None);
    }

    #[test]
    fn cpuid_is_recognized() {
        assert_eq!(try_match_cpuid(""), None);
        assert_eq!(try_match_cpuid("Random other input"), None);

        assert_eq!(
            try_match_cpuid("CPUID 00000000: 00000016-756E6547-6C65746E-49656E69 [GenuineIntel]"),
            Some(InputLine::Cpuid {
                query: CpuidQuery {
                    leaf: 0,
                    subleaf: 0,
                },
                result: CpuidResult {
                    eax: 0x16,
                    ebx: 0x756E6547,
                    ecx: 0x6C65746E,
                    edx: 0x49656E69,
                }
            })
        );

        assert_eq!(
            try_match_cpuid("CPUID 00000001: 000906ED-0E100800-7FFAFBBF-BFEBFBFF"),
            Some(InputLine::Cpuid {
                query: CpuidQuery {
                    leaf: 1,
                    subleaf: 0,
                },
                result: CpuidResult {
                    eax: 0x906ED,
                    ebx: 0x0E100800,
                    ecx: 0x7FFAFBBF,
                    edx: 0xBFEBFBFF,
                }
            })
        );

        assert_eq!(
            try_match_cpuid("CPUID 00000004: 1C03C163-03C0003F-00003FFF-00000006 [SL 03]"),
            Some(InputLine::Cpuid {
                query: CpuidQuery {
                    leaf: 4,
                    subleaf: 3,
                },
                result: CpuidResult {
                    eax: 0x1C03C163,
                    ebx: 0x03C0003F,
                    ecx: 0x00003FFF,
                    edx: 0x00000006,
                }
            })
        );
    }

    #[test]
    fn msr_is_recognized() {
        assert_eq!(try_match_msr(""), None);
        assert_eq!(try_match_msr("Random other input"), None);

        assert_eq!(
            try_match_msr("MSR 000001FC: 0000-0000-0030-1CC3"),
            Some(InputLine::Msr {
                index: 0x1fc,
                value: 0x301cc3,
            })
        );

        assert_eq!(try_match_msr("MSR 00000300: < FAILED >"), None);

        assert_eq!(
            try_match_msr("MSR 0000030A: 0000-0000-0000-0000 [S200]"),
            Some(InputLine::Msr {
                index: 0x30a,
                value: 0x0,
            })
        );
    }

    #[test]
    fn aida_input_is_parsed() {
        let input = "
------[ Logical CPU #0 ]------

allcpu: Package 0 / Core 0 / Thread 0: Valid

CPUID 00000000: 00000016-756E6547-6C65746E-49656E69 [GenuineIntel]
CPUID 00000001: 000906ED-00100800-7FFAFBBF-BFEBFBFF

------[ Logical CPU #1 ]------

allcpu: Package 0 / Core 0 / Thread 1: Valid, Virtual

CPUID 00000004: 1C004121-01C0003F-0000003F-00000000 [SL 00]
CPUID 00000004: 1C004122-01C0003F-0000003F-00000000 [SL 01]

------[ MSR Registers ]------

MSR 00000017: 0004-0000-0000-0000 [PlatID = 1]
MSR 0000001B: 0000-0000-FEE0-0900
";

        let aida_dump = AidaCpuidDump::from_str(input).expect("to be able to parse example input");

        assert_eq!(aida_dump.cpuid.len(), 2);
        assert_eq!(
            aida_dump
                .cpuid
                .get(&CpuidQuery {
                    leaf: 1,
                    subleaf: 0
                })
                .expect("to find CPUID leaf"),
            &CpuidResult {
                eax: 0x000906ED,
                ebx: 0x00100800,
                ecx: 0x7FFAFBBF,
                edx: 0xBFEBFBFF,
            }
        );

        assert_eq!(aida_dump.msrs.len(), 2);
        assert_eq!(
            *aida_dump.msrs.get(&0x17).expect("to find MSR value"),
            0x0004000000000000
        );
        assert_eq!(
            *aida_dump.msrs.get(&0x1b).expect("to find MSR value"),
            0x00000000FEE00900
        );
    }
}

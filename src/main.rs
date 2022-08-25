#[macro_use]
extern crate lazy_static;

use aida_parse::AidaCpuidDump;
use cpu_information::CpuInformation;
use features::Feature;
use std::error;
use std::io;
use std::io::Read;
use std::str::FromStr;

mod aida_parse;
mod cpu_information;
mod features;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

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

fn features() -> Vec<Feature> {
    use cpu_information::CpuidRegister::*;
    use features::BoolExpression::*;
    vec![
        Feature::new("AVX", CpuidBitSet(1.into(), Ecx, 28)),
        Feature::new("MMX", CpuidBitSet(1.into(), Edx, 23)),
        Feature::new("SHA", CpuidBitSet(7.into(), Ebx, 29)),
        Feature::new("ENCLV", CpuidBitSet(0x12.into(), Eax, 5)),
        Feature::new("EPT", MsrBitSet(0x48b, 32 + 1)),
        Feature::new("Unrestricted Guest", MsrBitSet(0x48b, 32 + 7)),
        Feature::new("VMCS Shadowing", MsrBitSet(0x48b, 46)),
        Feature::new("APIC-register virtualization", MsrBitSet(0x48b, 40)),
        Feature::new("Virtual-interrupt delivery", MsrBitSet(0x48b, 41)),
        Feature::new("VMX preemption timer", MsrBitSet(0x48b, 32 + 6)),
        Feature::new("Process posted interrupts", MsrBitSet(0x48b, 32 + 7)),
    ]
}
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

    for feature in features().into_iter() {
        println!(
            "{:30}: {}",
            feature.name,
            tristate_to_char(feature.is_present(&aida_result)),
        );
    }

    Ok(())
}

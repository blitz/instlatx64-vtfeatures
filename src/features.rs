use std::ops::{BitAnd, BitOr, Not};

use crate::cpu_information::{CpuInformation, CpuidQuery, CpuidRegister};

pub type Bit = u8;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoolExpression {
    CpuidBitSet(CpuidQuery, CpuidRegister, Bit),
    MsrBitSet(u32, Bit),

    And(Box<BoolExpression>, Box<BoolExpression>),
    Or(Box<BoolExpression>, Box<BoolExpression>),
    Not(Box<BoolExpression>),
}

impl BoolExpression {
    pub fn evaluate(&self, cpu_info: &dyn CpuInformation) -> Option<bool> {
        Some(match self {
            BoolExpression::CpuidBitSet(query, reg, bit) => {
                assert!(u32::from(*bit) < u32::BITS);
                (cpu_info.cpuid(*query)?.get(*reg) & (1 << bit)) != 0
            }
            BoolExpression::MsrBitSet(index, bit) => {
                assert!(u32::from(*bit) < u64::BITS);
                (cpu_info.rdmsr(*index)? & (1 << bit)) != 0
            }
            BoolExpression::And(expr1, expr2) => {
                expr1.evaluate(cpu_info)? && expr2.evaluate(cpu_info)?
            }
            BoolExpression::Or(expr1, expr2) => {
                expr1.evaluate(cpu_info)? || expr2.evaluate(cpu_info)?
            }
            BoolExpression::Not(expr) => !expr.evaluate(cpu_info)?,
        })
    }
}

impl BitAnd for BoolExpression {
    type Output = BoolExpression;

    fn bitand(self, rhs: Self) -> Self::Output {
        BoolExpression::And(self.into(), rhs.into())
    }
}

impl BitOr for BoolExpression {
    type Output = BoolExpression;

    fn bitor(self, rhs: Self) -> Self::Output {
        BoolExpression::Or(self.into(), rhs.into())
    }
}

impl Not for BoolExpression {
    type Output = BoolExpression;

    fn not(self) -> Self::Output {
        BoolExpression::Not(self.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Feature {
    pub name: String,
    expr: BoolExpression,
}

impl Feature {
    pub fn new(name: &str, expr: BoolExpression) -> Self {
        Self {
            expr,
            name: name.to_owned(),
        }
    }

    pub fn is_present(&self, cpu_info: &dyn CpuInformation) -> Option<bool> {
        self.expr.evaluate(cpu_info)
    }
}

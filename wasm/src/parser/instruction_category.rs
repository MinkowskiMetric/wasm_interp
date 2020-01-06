use std::convert::TryFrom;
use std::io;

use crate::parser::{InstructionAccumulator, Opcode};

#[derive(Debug, PartialEq)]
pub enum InstructionCategory {
    SingleByte,       // No arguments
    SingleLebInteger, // Single argument, can be I32 or I64
    SingleFloat,      // Single argument of type F32
    SingleDouble,     // Single argument of type F64
    Block(bool),      // One or two sub expressions
    Else,             // No arguments
    End,              // No arguments
    TwoLebInteger,    // Two I32 arguments
    BranchTable,      // Vector of I32 arguments containing at least one entry
}

impl InstructionCategory {
    pub fn from_lead_byte(lead_byte: u8) -> io::Result<InstructionCategory> {
        Ok(Self::from_opcode(Opcode::from_byte(lead_byte)?))
    }

    pub fn from_opcode(opcode: Opcode) -> InstructionCategory {
        match opcode {
            // Most of the instructions are single byte instructions, so only the special
            // cases are listed here
            Opcode::Block | Opcode::Loop => InstructionCategory::Block(false),
            Opcode::If => InstructionCategory::Block(true),
            Opcode::Else => InstructionCategory::Else,
            Opcode::End => InstructionCategory::End,
            Opcode::Br | Opcode::BrIf => InstructionCategory::SingleLebInteger,
            Opcode::BrTable => InstructionCategory::BranchTable,
            Opcode::Call => InstructionCategory::SingleLebInteger,
            Opcode::CallIndirect => InstructionCategory::TwoLebInteger,
            Opcode::LocalGet
            | Opcode::LocalSet
            | Opcode::LocalTee
            | Opcode::GlobalGet
            | Opcode::GlobalSet => InstructionCategory::SingleLebInteger,
            Opcode::I32Load
            | Opcode::I64Load
            | Opcode::F32Load
            | Opcode::F64Load
            | Opcode::I32Load8S
            | Opcode::I32Load8U
            | Opcode::I32Load16S
            | Opcode::I32Load16U
            | Opcode::I64Load8S
            | Opcode::I64Load8U
            | Opcode::I64Load16S
            | Opcode::I64Load16U
            | Opcode::I64Load32S
            | Opcode::I64Load32U
            | Opcode::I32Store
            | Opcode::I64Store
            | Opcode::F32Store
            | Opcode::F64Store
            | Opcode::I32Store8
            | Opcode::I32Store16
            | Opcode::I64Store8
            | Opcode::I64Store16
            | Opcode::I64Store32 => InstructionCategory::TwoLebInteger,
            Opcode::MemorySize | Opcode::MemoryGrow => InstructionCategory::SingleLebInteger,
            Opcode::I32Const | Opcode::I64Const => InstructionCategory::SingleLebInteger,
            Opcode::F32Const => InstructionCategory::SingleFloat,
            Opcode::F64Const => InstructionCategory::SingleDouble,

            _ => InstructionCategory::SingleByte,
        }
    }

    pub fn ensure_instruction<T: InstructionAccumulator>(
        &self,
        acc: &mut T,
        offset: usize,
    ) -> io::Result<usize> {
        match self {
            InstructionCategory::SingleByte
            | InstructionCategory::Else
            | InstructionCategory::End => acc.ensure_bytes(offset + 1).map(|_| 1),
            InstructionCategory::SingleLebInteger => {
                acc.ensure_leb_at(offset + 1).map(|leb_size| 1 + leb_size)
            }
            InstructionCategory::SingleFloat => acc.ensure_bytes(offset + 5).map(|_| 5),
            InstructionCategory::SingleDouble => acc.ensure_bytes(offset + 9).map(|_| 9),
            InstructionCategory::Block(allow_else) => {
                self.ensure_block_instruction(*allow_else, acc, offset)
            }
            InstructionCategory::TwoLebInteger => self.ensure_two_leb_integer(acc, offset),
            InstructionCategory::BranchTable => self.ensure_branch_table(acc, offset),
        }
    }

    fn ensure_two_leb_integer<T: InstructionAccumulator>(
        &self,
        acc: &mut T,
        offset: usize,
    ) -> io::Result<usize> {
        let align_size = acc.ensure_leb_at(offset + 1)?;
        let offset_size = acc.ensure_leb_at(offset + 1 + align_size)?;

        Ok(1 + align_size + offset_size)
    }

    fn ensure_block_instruction<T: InstructionAccumulator>(
        &self,
        allow_else: bool,
        acc: &mut T,
        offset: usize,
    ) -> io::Result<usize> {
        // Child instruction offset always starts at 2 because all blocks begin with a block type
        let mut next_child_offset: usize = offset + 2;
        let mut seen_else = false;

        loop {
            // Make sure that we have the lead byte of the next instruction
            acc.ensure_bytes(next_child_offset + 1)?;

            // Now get the lead byte
            let child_lead_byte = acc.get_byte(next_child_offset);
            let child_instr_cat = InstructionCategory::from_lead_byte(child_lead_byte)?;

            // Now ensure that we have that instruction
            let child_instr_size = child_instr_cat.ensure_instruction(acc, next_child_offset)?;

            if child_instr_cat == InstructionCategory::Else {
                if seen_else || !allow_else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unexpected else in block",
                    ));
                }

                // Move past the else
                next_child_offset += child_instr_size;

                // Record that we've seen the else so we don't allow a second one
                seen_else = true;
            } else if child_instr_cat == InstructionCategory::End {
                // Subtract the original offset to get the instruction size
                return Ok(next_child_offset + child_instr_size - offset);
            } else {
                // Move on to the next instruction
                next_child_offset += child_instr_size;
            }
        }
    }

    fn ensure_branch_table<T: InstructionAccumulator>(
        &self,
        acc: &mut T,
        offset: usize,
    ) -> io::Result<usize> {
        // This is a bloody complicated instruction to parse
        // Basically we have a vector of integers followed by an integer

        // We start by ensuring that the vector length is present
        let mut instr_size: usize = 1 + acc.ensure_leb_at(offset + 1)?;

        // Now we read the vector length
        let vector_length = acc.get_leb_u32_at(offset + 1);

        for _ in 0..vector_length {
            // Add on the length of the integer from the vector
            instr_size += acc.ensure_leb_at(offset + instr_size)?;
        }

        // And finally, there is the last entry
        instr_size += acc.ensure_leb_at(offset + instr_size)?;

        Ok(instr_size)
    }

    pub fn get_single_u32_arg<T: InstructionAccumulator>(&self, acc: &T, offset: usize) -> u32 {
        match self {
            InstructionCategory::SingleLebInteger => acc.get_leb_u32_at(offset + 1),
            _ => panic!("Not valid for instruction type"),
        }
    }

    pub fn get_single_i32_arg<T: InstructionAccumulator>(&self, acc: &T, offset: usize) -> i32 {
        match self {
            InstructionCategory::SingleLebInteger => acc.get_leb_i32_at(offset + 1),
            _ => panic!("Not valid for instruction type"),
        }
    }

    pub fn get_single_u32_as_usize_arg<T: InstructionAccumulator>(
        &self,
        acc: &T,
        offset: usize,
    ) -> usize {
        usize::try_from(self.get_single_u32_arg(acc, offset)).unwrap()
    }

    pub fn get_single_u64_arg<T: InstructionAccumulator>(&self, acc: &T, offset: usize) -> u64 {
        match self {
            InstructionCategory::SingleLebInteger => acc.get_leb_u64_at(offset + 1),
            _ => panic!("Not valid for instruction type"),
        }
    }

    pub fn get_single_i64_arg<T: InstructionAccumulator>(&self, acc: &T, offset: usize) -> i64 {
        match self {
            InstructionCategory::SingleLebInteger => acc.get_leb_i64_at(offset + 1),
            _ => panic!("Not valid for instruction type"),
        }
    }

    pub fn get_single_f32_arg<T: InstructionAccumulator>(&self, acc: &T, offset: usize) -> f32 {
        match self {
            InstructionCategory::SingleLebInteger => acc.get_f32_at(offset + 1),
            _ => panic!("Not valid for instruction type"),
        }
    }

    pub fn get_single_f64_arg<T: InstructionAccumulator>(&self, acc: &T, offset: usize) -> f64 {
        match self {
            InstructionCategory::SingleLebInteger => acc.get_f64_at(offset + 1),
            _ => panic!("Not valid for instruction type"),
        }
    }
}

use crate::{
    core::BlockType,
    parser::{InstructionAccumulator, Opcode},
};
use anyhow::{anyhow, Result};
use std::convert::{TryFrom, TryInto};

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

#[derive(Debug)]
pub struct BlockRange {
    start: usize,
    end: usize,
}

#[derive(Debug)]
pub struct InstructionData {
    length: usize,
    block_range: Option<BlockRange>,
    else_range: Option<BlockRange>,
}

fn simple_instruction_data(length: usize) -> InstructionData {
    InstructionData {
        length,
        block_range: None,
        else_range: None,
    }
}

fn block_instruction_data(
    length: usize,
    block_range: BlockRange,
    else_range: Option<BlockRange>,
) -> InstructionData {
    InstructionData {
        length,
        block_range: Some(block_range),
        else_range,
    }
}

impl InstructionData {
    pub fn length(&self) -> usize {
        self.length
    }
}

impl InstructionCategory {
    pub fn from_lead_byte(lead_byte: u8) -> Result<InstructionCategory> {
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
    ) -> Result<InstructionData> {
        match self {
            InstructionCategory::SingleByte
            | InstructionCategory::Else
            | InstructionCategory::End => acc
                .ensure_bytes(offset + 1)
                .map(|_| simple_instruction_data(1)),
            InstructionCategory::SingleLebInteger => acc
                .ensure_leb_at(offset + 1)
                .map(|leb_size| simple_instruction_data(1 + leb_size)),
            InstructionCategory::SingleFloat => acc
                .ensure_bytes(offset + 5)
                .map(|_| simple_instruction_data(5)),
            InstructionCategory::SingleDouble => acc
                .ensure_bytes(offset + 9)
                .map(|_| simple_instruction_data(9)),
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
    ) -> Result<InstructionData> {
        let align_size = acc.ensure_leb_at(offset + 1)?;
        let offset_size = acc.ensure_leb_at(offset + 1 + align_size)?;

        Ok(simple_instruction_data(1 + align_size + offset_size))
    }

    fn ensure_block_instruction<T: InstructionAccumulator>(
        &self,
        allow_else: bool,
        acc: &mut T,
        offset: usize,
    ) -> Result<InstructionData> {
        // Validate the block type
        acc.ensure_bytes(offset + 2)?;
        BlockType::try_from(acc.get_byte(offset + 1))?;

        // Child instruction offset always starts at 2 because all blocks begin with a block type
        let mut next_child_offset = offset + 2;
        let mut range_start = next_child_offset;
        let mut block_range: Option<BlockRange> = None;

        loop {
            // Make sure that we have the lead byte of the next instruction
            acc.ensure_bytes(next_child_offset + 1)?;

            // Now get the lead byte
            let child_lead_byte = acc.get_byte(next_child_offset);
            let child_instr_cat = InstructionCategory::from_lead_byte(child_lead_byte)?;

            // Now ensure that we have that instruction
            let child_instr_size = child_instr_cat.ensure_instruction(acc, next_child_offset)?;

            if child_instr_cat == InstructionCategory::Else {
                if !block_range.is_none() || !allow_else {
                    return Err(anyhow!("Unexpected else in block"));
                }

                block_range = Some(BlockRange {
                    start: range_start - offset,
                    end: next_child_offset - offset,
                });

                // Move past the else
                next_child_offset += child_instr_size.length();
                range_start = next_child_offset;
            } else if child_instr_cat == InstructionCategory::End {
                let current_range = BlockRange {
                    start: range_start - offset,
                    end: next_child_offset - offset,
                };

                let instruction_data = if let Some(block_range) = block_range {
                    block_instruction_data(
                        next_child_offset + child_instr_size.length() - offset,
                        block_range,
                        Some(current_range),
                    )
                } else {
                    block_instruction_data(
                        next_child_offset + child_instr_size.length() - offset,
                        current_range,
                        None,
                    )
                };

                // Subtract the original offset to get the instruction size
                return Ok(instruction_data);
            } else {
                // Move on to the next instruction
                next_child_offset += child_instr_size.length();
            }
        }
    }

    fn ensure_branch_table<T: InstructionAccumulator>(
        &self,
        acc: &mut T,
        offset: usize,
    ) -> Result<InstructionData> {
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

        Ok(simple_instruction_data(instr_size))
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
            InstructionCategory::SingleFloat => acc.get_f32_at(offset + 1),
            _ => panic!("Not valid for instruction type"),
        }
    }

    pub fn get_single_f64_arg<T: InstructionAccumulator>(&self, acc: &T, offset: usize) -> f64 {
        match self {
            InstructionCategory::SingleDouble => acc.get_f64_at(offset + 1),
            _ => panic!("Not valid for instruction type"),
        }
    }

    pub fn get_pair_u32_arg<T: InstructionAccumulator>(
        &self,
        acc: &T,
        offset: usize,
    ) -> (u32, u32) {
        match self {
            InstructionCategory::TwoLebInteger => {
                // TODOTODOTODO - this can be optimized - there is no need to measure the number then read it.
                // I'll do that later as part of a wider overhaul of the leb reading
                let first_arg_size = acc.get_leb_size_at(offset + 1);
                let arg1 = acc.get_leb_u32_at(offset + 1);
                let arg2 = acc.get_leb_u32_at(offset + first_arg_size + 1);
                (arg1, arg2)
            }
            _ => panic!("Not valid for this instruction type"),
        }
    }

    pub fn get_pair_u32_as_usize_arg(
        &self,
        acc: &impl InstructionAccumulator,
        offset: usize,
    ) -> (usize, usize) {
        let (a1, a2) = self.get_pair_u32_arg(acc, offset);
        (a1.try_into().unwrap(), a2.try_into().unwrap())
    }

    pub fn get_block_type(&self, acc: &impl InstructionAccumulator, offset: usize) -> BlockType {
        match self {
            InstructionCategory::Block(_) => {
                BlockType::from_byte(acc.get_byte(offset + 1)).unwrap()
            }

            _ => panic!(
                "No block result type for instructions of category {:?}",
                self
            ),
        }
    }

    pub fn has_else_block(
        &self,
        _acc: &impl InstructionAccumulator,
        _offset: usize,
        data: &InstructionData,
    ) -> bool {
        match data {
            InstructionData {
                else_range: Some(_),
                ..
            } => true,
            _ => false,
        }
    }

    pub fn get_block<'a>(
        &'_ self,
        acc: &'a impl InstructionAccumulator,
        offset: usize,
        data: &InstructionData,
    ) -> &'a [u8] {
        match data {
            InstructionData {
                block_range: Some(block_range),
                ..
            } => acc.get_bytes(
                offset + block_range.start,
                block_range.end - block_range.start,
            ),
            _ => panic!("No else block"),
        }
    }

    pub fn get_else_block<'a>(
        &'_ self,
        acc: &'a impl InstructionAccumulator,
        offset: usize,
        data: &InstructionData,
    ) -> &'a [u8] {
        match data {
            InstructionData {
                else_range: Some(block_range),
                ..
            } => acc.get_bytes(
                offset + block_range.start,
                block_range.end - block_range.start,
            ),
            _ => panic!("No else block"),
        }
    }

    pub fn get_block_table_targets(
        &self,
        acc: &impl InstructionAccumulator,
        offset: usize,
    ) -> Vec<usize> {
        // There have got to be better ways to parse this.
        let mut instr_size: usize = 1 + acc.get_leb_size_at(offset + 1);
        let vector_length = acc.get_leb_usize_at(offset + 1);
        let mut ret = Vec::with_capacity(vector_length + 1);

        for _ in 0..(vector_length + 1) {
            let number_size = acc.get_leb_size_at(offset + instr_size);
            ret.push(acc.get_leb_u32_at(instr_size).try_into().unwrap());
            instr_size += number_size;
        }

        ret
    }
}

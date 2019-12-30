use std::io;

use crate::parser::InstructionAccumulator;

#[derive(Debug, PartialEq)]
pub enum InstructionCategory {
    SingleByte,
    SingleLebInteger,
    SingleFloat,
    SingleDouble,
    Block(bool),
    Else,
    End,
    MemSizeGrow,
    MemInstr,
    IndirectCall,
    BranchTable,
}

impl InstructionCategory {
    pub fn from_lead_byte(lead_byte: u8) -> io::Result<InstructionCategory> {
        match lead_byte {
            0x00..=0x01 => Ok(InstructionCategory::SingleByte),
            0x02..=0x03 => Ok(InstructionCategory::Block(false)),
            0x04 => Ok(InstructionCategory::Block(true)),
            0x05 => Ok(InstructionCategory::Else),
            //  0x06 ..= 0x0A are not listed in the spec
            0x0B => Ok(InstructionCategory::End),
            0x0C..=0x0D => Ok(InstructionCategory::SingleLebInteger),
            0x0E => Ok(InstructionCategory::BranchTable),
            0x0F => Ok(InstructionCategory::SingleByte),
            0x10 => Ok(InstructionCategory::SingleLebInteger),
            0x11 => Ok(InstructionCategory::IndirectCall),
            //  0x12 ..= 0x19 are not listed in the spec
            0x1a..=0x1b => Ok(InstructionCategory::SingleByte),
            //  0x1c ..= 0x1f are not listed in the spec
            0x20..=0x24 => Ok(InstructionCategory::SingleLebInteger),
            //  0x25 ..= 0x27 are not listed in the spec
            0x28..=0x3E => Ok(InstructionCategory::MemInstr),
            0x3F..=0x40 => Ok(InstructionCategory::MemSizeGrow),
            0x41..=0x42 => Ok(InstructionCategory::SingleLebInteger),
            0x43 => Ok(InstructionCategory::SingleFloat),
            0x44 => Ok(InstructionCategory::SingleDouble),
            0x45..=0xBF => Ok(InstructionCategory::SingleByte),
            //  0xC0 ..= 0xFF are not listed in the spec
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unknown instruction lead byte",
            )),
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
            InstructionCategory::MemSizeGrow => self.ensure_mem_size_grow(acc, offset),
            InstructionCategory::MemInstr => self.ensure_mem_instr(acc, offset),
            InstructionCategory::IndirectCall => self.ensure_indirect_call(acc, offset),
            InstructionCategory::BranchTable => self.ensure_branch_table(acc, offset),
        }
    }

    fn ensure_mem_instr<T: InstructionAccumulator>(
        &self,
        acc: &mut T,
        offset: usize,
    ) -> io::Result<usize> {
        let align_size = acc.ensure_leb_at(offset + 1)?;
        let offset_size = acc.ensure_leb_at(offset + 1 + align_size)?;

        Ok(1 + align_size + offset_size)
    }

    fn ensure_mem_size_grow<T: InstructionAccumulator>(
        &self,
        acc: &mut T,
        offset: usize,
    ) -> io::Result<usize> {
        let mem_idx_size = acc.ensure_leb_at(offset + 1)?;

        if acc.get_leb_u32_at(offset + 1) != 0x00 {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid memory value in mem grow or shrink call",
            ))
        } else {
            Ok(1 + mem_idx_size)
        }
    }

    fn ensure_indirect_call<T: InstructionAccumulator>(
        &self,
        acc: &mut T,
        offset: usize,
    ) -> io::Result<usize> {
        let arg_size = acc.ensure_leb_at(offset + 1)?;
        let mem_size = acc.ensure_leb_at(offset + 1 + arg_size)?;

        if acc.get_leb_u32_at(offset + 1 + arg_size) != 0x00 {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid memory value in indirect call - should be 0x00",
            ))
        } else {
            Ok(1 + arg_size + mem_size)
        }
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
}

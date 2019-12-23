use std::io;
use crate::type_section_data::ValueType;

// We have a couple of interesting problems around consuming instructions.
// When we read them from a file, we're only interested in walking the instruction list,
// but we need to keep them in a space efficient way (the original format is pretty good for
// that), which means we're also going to need to parse the instructions from a vector.
// We could use a reader to do that I guess. That works well because all of our utilities
// work like that. But... It makes random access hard - we're going to need to augment
// the reader with "seeking" I guess.

// Using a reader during execution doesn't make any sense though - we will end up having
// to accumulate the instructions, which is daft. So instead, we use an instruction iterator which
// we might have different implementations of

// Maybe trying to force all of this into a rust paradigm doesn't make any sense.

#[derive(Debug)]
pub enum BlockType {
    Void,
    Value(ValueType),
}

trait InstructionAccumulator {
    fn ensure_bytes(&mut self, bytes: usize) -> io::Result<()>;
    fn get_byte(&self, offset: usize) -> u8;

    fn ensure_leb_at(&mut self, offset: usize) -> io::Result<usize> {
        let mut number_length: usize = 1;
        loop {
            self.ensure_bytes(offset + number_length)?;

            if 0 == (self.get_byte(offset + number_length - 1) & 0x80) {
                return Ok(number_length);
            }

            number_length += 1;
        }
    }

    fn get_leb_u32_at(&self, offset: usize) -> u32 {
        let mut pos: usize = offset;
        let mut result: u32 = 0;
        let mut shift = 0;

        loop {
            let byte = self.get_byte(pos);
            pos += 1;
            result |= u32::from(byte & 0x7f) << shift;
            if (byte & 0x80) == 0 {
                return result;
            }
            shift += 7;
        }
    }
}

#[derive(Debug, PartialEq)]
enum InstructionCategory {
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
            0x00 ..= 0x01       => Ok(InstructionCategory::SingleByte),
            0x02 ..= 0x03       => Ok(InstructionCategory::Block(false)),
            0x04                => Ok(InstructionCategory::Block(true)),
            0x05                => Ok(InstructionCategory::Else),
        //  0x06 ..= 0x0A are not listed in the spec
            0x0B                => Ok(InstructionCategory::End),
            0x0C ..= 0x0D       => Ok(InstructionCategory::SingleLebInteger),
            0x0E                => Ok(InstructionCategory::BranchTable),
            0x0F                => Ok(InstructionCategory::SingleByte),
            0x10                => Ok(InstructionCategory::SingleLebInteger),
            0x11                => Ok(InstructionCategory::IndirectCall),
        //  0x12 ..= 0x19 are not listed in the spec
            0x1a ..= 0x1b       => Ok(InstructionCategory::SingleByte),
        //  0x1c ..= 0x1f are not listed in the spec
            0x20 ..= 0x24       => Ok(InstructionCategory::SingleLebInteger),
        //  0x25 ..= 0x27 are not listed in the spec
            0x28 ..= 0x3E       => Ok(InstructionCategory::MemInstr),
            0x3F ..= 0x40       => Ok(InstructionCategory::MemSizeGrow),
            0x41 ..= 0x42       => Ok(InstructionCategory::SingleLebInteger),
            0x43                => Ok(InstructionCategory::SingleFloat),
            0x44                => Ok(InstructionCategory::SingleDouble),
            0x45 ..= 0xBF       => Ok(InstructionCategory::SingleByte),
        //  0xC0 ..= 0xFF are not listed in the spec

            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown instruction lead byte")),
        }
    }

    pub fn ensure_instruction<T: InstructionAccumulator>(&self, acc: &mut T, offset: usize) -> io::Result<usize> {
        match self {
            InstructionCategory::SingleByte | InstructionCategory::Else | InstructionCategory::End 
                                                    => acc.ensure_bytes(offset + 1).map(|_| 1),
            InstructionCategory::SingleLebInteger   => acc.ensure_leb_at(offset + 1).map(|leb_size| 1 + leb_size),
            InstructionCategory::SingleFloat        => acc.ensure_bytes(offset + 5).map(|_| 5),
            InstructionCategory::SingleDouble       => acc.ensure_bytes(offset + 9).map(|_| 9),
            InstructionCategory::Block(allow_else)  => self.ensure_block_instruction(*allow_else, acc, offset),
            InstructionCategory::MemSizeGrow        => self.ensure_mem_size_grow(acc, offset),
            InstructionCategory::MemInstr           => self.ensure_mem_instr(acc, offset),
            InstructionCategory::IndirectCall       => self.ensure_indirect_call(acc, offset),
            InstructionCategory::BranchTable        => self.ensure_branch_table(acc, offset),
        }
    }

    fn ensure_mem_instr<T: InstructionAccumulator>(&self, acc: &mut T, offset: usize) -> io::Result<usize> {
        let align_size = acc.ensure_leb_at(offset + 1)?;
        let offset_size = acc.ensure_leb_at(offset + 1 + align_size)?;

        Ok(1 + align_size + offset_size)
    }

    fn ensure_mem_size_grow<T: InstructionAccumulator>(&self, acc: &mut T, offset: usize) -> io::Result<usize> {
        let mem_idx_size = acc.ensure_leb_at(offset + 1)?;

        if acc.get_leb_u32_at(offset + 1) != 0x00 {
            Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid memory value in mem grow or shrink call"))
        } else {
            Ok(1 + mem_idx_size)
        }
    }

    fn ensure_indirect_call<T: InstructionAccumulator>(&self, acc: &mut T, offset: usize) -> io::Result<usize> {
        let arg_size = acc.ensure_leb_at(offset + 1)?;
        let mem_size = acc.ensure_leb_at(offset + 1 + arg_size)?;

        if acc.get_leb_u32_at(offset + 1 + arg_size) != 0x00 {
            Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid memory value in indirect call - should be 0x00"))
        } else {
            Ok(1 + arg_size + mem_size)
        }
    }
    
    fn ensure_block_instruction<T: InstructionAccumulator>(&self, allow_else: bool, acc: &mut T, offset: usize) -> io::Result<usize> {
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
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Unexpected else in block"));
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

    fn ensure_branch_table<T: InstructionAccumulator>(&self, acc: &mut T, offset: usize) -> io::Result<usize> {
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

struct ReaderInstructionAccumulator<'a, T: io::Read> {
    reader: &'a mut T,              // Where we get the instructions from
    buf: Vec<u8>,                   // We accumulate the instructions in here
    next_inst: usize,               // The position of the next instruction byte in the buffer
}

impl<'a, T> ReaderInstructionAccumulator<'a, T> where T: io::Read {
    pub fn new(reader: &'a mut T) -> Self {
        Self {
            reader: reader,
            buf: Vec::new(),
            next_inst: 0,
        }
    }

    pub fn move_to_next(&mut self) -> io::Result<bool> {
        // Move past the current instruction
        self.next_inst = self.buf.len();

        // Start by ensuring we have at least one byte
        self.ensure_bytes(1)?;

        let lead_byte = self.get_byte(0);
        let instruction_category = InstructionCategory::from_lead_byte(lead_byte)?;
        instruction_category.ensure_instruction(self, 0)?;

        Ok(instruction_category != InstructionCategory::End)
    }

    pub fn instr_bytes(self) -> Vec<u8> {
        self.buf
    }

    pub fn this_instr_bytes(&self) -> &[u8] {
        &self.buf[self.next_inst..]
    }
}

impl<'a, T> InstructionAccumulator for ReaderInstructionAccumulator<'a, T> where T: io::Read {
    fn ensure_bytes(&mut self, bytes: usize) -> io::Result<()> {
        let required_bytes = self.next_inst + bytes;
        const BUF_SIZE: usize = 16;
        let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];

        while self.buf.len() < required_bytes {
            // Got to be careful not to read more bytes than we were asked to
            let bytes_to_read = std::cmp::min(BUF_SIZE, required_bytes - self.buf.len());

            self.reader.read_exact(&mut buf[0..bytes_to_read])?;
            self.buf.extend_from_slice(&buf[0..bytes_to_read]);
        }

        Ok(())
    }

    fn get_byte(&self, idx: usize) -> u8 {
        assert!(self.buf.len() > self.next_inst + idx, "Byte is not available");

        self.buf[self.next_inst + idx]
    }
}

#[derive(Debug)]
pub struct Expr {
    // So, a basic expr is just the bytes that make up the expression
    instr: Vec<u8>,
}

impl Expr {
    pub fn read<T: io::Read>(reader: &mut T) -> io::Result<Expr> {
        let mut acc = ReaderInstructionAccumulator::new(reader);
        
        while acc.move_to_next()? {
            // Nothing in here - we're just accumulating the instructions
        }

        Ok(Expr { instr: acc.instr_bytes() })
    }
}
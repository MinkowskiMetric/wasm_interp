use std::io;
use std::io::prelude::*;

use crate::parser::{InstructionAccumulator, InstructionCategory};

struct ReaderInstructionAccumulator<'a, T: Read> {
    reader: &'a mut T,              // Where we get the instructions from
    buf: Vec<u8>,                   // We accumulate the instructions in here
    next_inst: usize,               // The position of the next instruction byte in the buffer
}

impl<'a, T> ReaderInstructionAccumulator<'a, T> where T: Read {
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

pub fn read_expression_bytes<T: Read>(reader: &mut T) -> io::Result<Vec<u8>> {
    let mut acc = ReaderInstructionAccumulator::new(reader);
    
    while acc.move_to_next()? {
        // Nothing in here - we're just accumulating the instructions
    }

    Ok(acc.instr_bytes())
}
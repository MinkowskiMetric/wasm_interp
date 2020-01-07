use crate::parser::{self, InstructionAccumulator};
use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct Instruction<'a> {
    bytes: &'a [u8],
    opcode: parser::Opcode,
    cat: parser::InstructionCategory,
    acc: parser::SliceInstructionAccumulator<'a>,
}

impl<'a> Instruction<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        // All instructions are at least one byte long, and we depend heavily on that assumption
        assert!(bytes.len() > 0);

        let opcode = parser::Opcode::from_byte(bytes[0]).unwrap();
        let cat = parser::InstructionCategory::from_opcode(opcode.clone());
        let mut acc = parser::make_slice_accumulator(bytes);
        assert!(cat.ensure_instruction(&mut acc, 0).is_ok());

        Self {
            bytes,
            opcode,
            cat,
            acc,
        }
    }

    #[allow(dead_code)]
    fn lead_byte(&self) -> u8 {
        self.bytes[0]
    }

    #[allow(dead_code)]
    pub fn opcode(&self) -> parser::Opcode {
        self.opcode.clone()
    }

    #[allow(dead_code)]
    fn category(&self) -> &parser::InstructionCategory {
        &self.cat
    }

    #[allow(dead_code)]
    fn is_block_start(&self) -> bool {
        match self.cat {
            parser::InstructionCategory::Block(_) => true,
            _ => false,
        }
    }

    fn is_block_end(&self) -> bool {
        self.cat == parser::InstructionCategory::End
    }

    #[allow(dead_code)]
    pub fn get_single_u32_arg(&self) -> u32 {
        self.cat.get_single_u32_arg(&self.acc, 0)
    }

    pub fn get_single_i32_arg(&self) -> i32 {
        self.cat.get_single_i32_arg(&self.acc, 0)
    }

    #[allow(dead_code)]
    pub fn get_single_u64_arg(&self) -> u64 {
        self.cat.get_single_u64_arg(&self.acc, 0)
    }

    pub fn get_single_i64_arg(&self) -> i64 {
        self.cat.get_single_i64_arg(&self.acc, 0)
    }

    pub fn get_single_u32_as_usize_arg(&self) -> usize {
        self.cat.get_single_u32_as_usize_arg(&self.acc, 0)
    }

    pub fn get_single_f32_arg(&self) -> f32 {
        self.cat.get_single_f32_arg(&self.acc, 0)
    }

    pub fn get_single_f64_arg(&self) -> f64 {
        self.cat.get_single_f64_arg(&self.acc, 0)
    }

    #[allow(dead_code)]
    pub fn get_pair_u32_arg(&self) -> (u32,u32) {
        self.cat.get_pair_u32_arg(&self.acc, 0)
    }

    pub fn get_pair_u32_as_usize_arg(&self) -> (usize,usize) {
        self.cat.get_pair_u32_as_usize_arg(&self.acc, 0)
    }
}

pub struct InstructionIterator<'a, Source: InstructionSource> {
    source: &'a Source,
    current_instr_start: usize,
    current_instr_end: usize,
}

impl<'a, Source: InstructionSource> InstructionIterator<'a, Source> {
    pub fn new(source: &'a Source) -> Self {
        Self {
            source,
            current_instr_start: 0,
            current_instr_end: 0,
        }
    }

    fn next_internal(&mut self) -> Result<Instruction<'a>> {
        // So, we can forget about any previous instruction now and move on
        self.current_instr_start = self.current_instr_end;

        // There must be at least one byte beyond the end of the current instruction, otherwise
        // we shouldn't have got here
        assert!(self.source.get_instruction_bytes().len() > self.current_instr_end);

        let lead_byte = self.get_byte(0);
        let lead_byte = parser::InstructionCategory::from_lead_byte(lead_byte)?;
        let instr_length = lead_byte.ensure_instruction(self, 0)?;

        self.current_instr_end += instr_length;

        Ok(Instruction::new(
            &self.source.get_instruction_bytes()[self.current_instr_start..self.current_instr_end],
        ))
    }
}

impl<'a, Source: InstructionSource> Iterator for InstructionIterator<'a, Source> {
    type Item = Result<Instruction<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_instr_end < self.source.get_instruction_bytes().len() {
            match self.next_internal() {
                Ok(instr) => {
                    if instr.is_block_end() {
                        // This is the "end" instruction - we don't return this, but we check that it should
                        // be at the end of the expression
                        assert!(
                            self.current_instr_end == self.source.get_instruction_bytes().len()
                        );
                        None
                    } else {
                        Some(Ok(instr))
                    }
                }
                other => Some(other),
            }
        } else {
            None
        }
    }
}

impl<'a, Source: InstructionSource> InstructionSource for InstructionIterator<'a, Source> {
    fn get_instruction_bytes(&self) -> &[u8] {
        self.source.get_instruction_bytes()
    }
}

impl<'a, Source: InstructionSource> parser::InstructionAccumulator
    for InstructionIterator<'a, Source>
{
    fn ensure_bytes(&mut self, bytes: usize) -> Result<()> {
        if (self.current_instr_start + bytes) > self.source.get_instruction_bytes().len() {
            Err(anyhow!("Not enough instruction bytes in expression"))
        } else {
            Ok(())
        }
    }

    fn get_bytes(&self, offset: usize, length: usize) -> &[u8] {
        assert!(
            (self.current_instr_start + offset + length)
                <= self.source.get_instruction_bytes().len()
        );
        &self.source.get_instruction_bytes()
            [self.current_instr_start + offset..self.current_instr_start + offset + length]
    }
}

pub trait InstructionSource {
    fn get_instruction_bytes(&self) -> &[u8];

    fn iter<'a>(&'a self) -> InstructionIterator<'a, Self>
    where
        Self: Sized,
    {
        InstructionIterator::new(&self)
    }
}

impl<T: AsRef<[u8]>> InstructionSource for T {
    fn get_instruction_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

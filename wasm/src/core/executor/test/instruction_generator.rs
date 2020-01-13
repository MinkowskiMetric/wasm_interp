use crate::core::{stack_entry::StackEntry, BlockType};
use crate::parser::{InstructionCategory, InstructionSource, Opcode};

use std::convert::TryInto;

fn write_leb(expr_bytes: &mut Vec<u8>, val: u64, signed: bool) {
    let mut encoded_bytes: [u8; 10] = [
        (0x80 | (val & 0x7f)).try_into().unwrap(),
        (0x80 | ((val >> 7) & 0x7f)).try_into().unwrap(),
        (0x80 | ((val >> 14) & 0x7f)).try_into().unwrap(),
        (0x80 | ((val >> 21) & 0x7f)).try_into().unwrap(),
        (0x80 | ((val >> 28) & 0x7f)).try_into().unwrap(),
        (0x80 | ((val >> 35) & 0x7f)).try_into().unwrap(),
        (0x80 | ((val >> 42) & 0x7f)).try_into().unwrap(),
        (0x80 | ((val >> 49) & 0x7f)).try_into().unwrap(),
        (0x80 | ((val >> 56) & 0x7f)).try_into().unwrap(),
        ((val >> 63) & 0x7f).try_into().unwrap(),
    ];

    let is_positive = !signed || 0 == (val & 0x8000000000000000);

    let mut required_length: usize = 10;
    while required_length > 1 {
        let last_byte = encoded_bytes[required_length - 1];
        let penultimate_byte = encoded_bytes[required_length - 2];

        // check that the signal bits are set the way we expect
        assert!((last_byte & 0x80) == 0);
        assert!((penultimate_byte & 0x80) == 0x80);

        // We have to check the high bit of the previous byte as we
        // scan because it has to match the bits we're dropping, otherwise
        // when it gets sign extended it will go wrong
        let can_drop_byte = if is_positive {
            last_byte == 0x00 && (penultimate_byte & 0xC0) == 0x80
        } else if required_length == 10 {
            // We have a special case for the highest byte of a negative number just to cope with the
            // single set bit
            last_byte == 0x01 && (penultimate_byte & 0xC0) == 0xC0
        } else {
            last_byte == 0x7F && (penultimate_byte & 0xC0) == 0xC0
        };

        if can_drop_byte {
            // If we can drop the byte, then we can decrement the required length and
            // clear the top bit of the new last byte
            encoded_bytes[required_length - 2] = penultimate_byte & 0x7f;
            required_length -= 1;
        } else {
            // Otherwise, break out because we can't drop this byte
            break;
        }
    }

    // Write the bytes out as is
    expr_bytes.extend_from_slice(&encoded_bytes[0..required_length]);
}

fn write_leb_as_vector(i: u64, signed: bool) -> Vec<u8> {
    let mut vec = Vec::new();
    write_leb(&mut vec, i, signed);
    vec
}

fn write_signed_leb_as_vector(i: u64) -> Vec<u8> {
    write_leb_as_vector(i, true)
}

#[test]
fn test_leb_writer() {
    assert_eq!(write_signed_leb_as_vector(0), [0x00]);
    assert_eq!(write_signed_leb_as_vector(1), [0x01]);
    assert_eq!(write_signed_leb_as_vector(0x80), [0x80, 0x01]);
    assert_eq!(write_signed_leb_as_vector(0xFF), [0xFF, 0x01]);
    assert_eq!(write_signed_leb_as_vector(0xFFFF), [0xFF, 0xFF, 0x03]);
    assert_eq!(
        write_signed_leb_as_vector(unsafe { std::mem::transmute(-1i64) }),
        [0x7F]
    );
    assert_eq!(
        write_signed_leb_as_vector(unsafe { std::mem::transmute(-2i64) }),
        [0x7E]
    );
    assert_eq!(
        write_signed_leb_as_vector(unsafe { std::mem::transmute(-256i64) }),
        [0x80, 0x7E]
    );
    assert_eq!(
        write_signed_leb_as_vector(unsafe { std::mem::transmute(-65536i64) }),
        [0x80, 0x80, 0x7C]
    );
}

fn write_opcode(expr_bytes: &mut ExpressionWriter, opcode: Opcode) {
    expr_bytes.append_byte(opcode.into());
}

fn write_const_instruction(expr_bytes: &mut ExpressionWriter, val: StackEntry) {
    match val {
        StackEntry::I32Entry(i) => {
            expr_bytes.append_byte(Opcode::I32Const.into());
            write_leb(&mut expr_bytes.bytes, i.into(), true);
        }
        StackEntry::I64Entry(i) => {
            expr_bytes.append_byte(Opcode::I64Const.into());
            write_leb(&mut expr_bytes.bytes, i.into(), true);
        }
        StackEntry::F32Entry(i) => {
            expr_bytes.append_byte(Opcode::F32Const.into());
            expr_bytes.append_bytes(&i.to_le_bytes());
        }
        StackEntry::F64Entry(i) => {
            expr_bytes.append_byte(Opcode::F64Const.into());
            expr_bytes.append_bytes(&i.to_le_bytes());
        }

        _ => panic!("Unsupported stack entry type"),
    }
}

struct ExpressionWriterStateStack {
    allow_else: bool,
    require_else: bool,
}

pub struct ExpressionWriter {
    bytes: Vec<u8>,
    state_stack: Vec<ExpressionWriterStateStack>,
}

pub fn make_expression_writer() -> ExpressionWriter {
    ExpressionWriter {
        bytes: Vec::new(),
        state_stack: Vec::new(),
    }
}

impl InstructionSource for ExpressionWriter {
    fn get_instruction_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl ExpressionWriter {
    fn append_bytes(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    fn append_byte(&mut self, byte: u8) {
        self.bytes.push(byte);
    }

    pub fn write_const_instruction(&mut self, val: impl Into<StackEntry>) {
        write_const_instruction(self, val.into());
    }

    pub fn write_single_byte_instruction(&mut self, opcode: Opcode) {
        assert!(InstructionCategory::from_opcode(opcode) == InstructionCategory::SingleByte);
        write_opcode(self, opcode);
    }

    pub fn write_single_leb_instruction(&mut self, opcode: Opcode, val: u64) {
        assert!(InstructionCategory::from_opcode(opcode) == InstructionCategory::SingleLebInteger);
        write_opcode(self, opcode);
        write_leb(&mut self.bytes, val, false);
    }

    pub fn write_two_leb_instruction(&mut self, opcode: Opcode, val1: u64, val2: u64) {
        assert!(InstructionCategory::from_opcode(opcode) == InstructionCategory::TwoLebInteger);
        write_opcode(self, opcode);
        write_leb(&mut self.bytes, val1, false);
        write_leb(&mut self.bytes, val2, false);
    }

    pub fn write_branch_table(&mut self, opcode: Opcode, table: &[u64]) {
        assert!(InstructionCategory::from_opcode(opcode) == InstructionCategory::BranchTable);
        assert!(table.len() > 0);

        write_opcode(self, opcode);
        write_leb(&mut self.bytes, (table.len() - 1) as u64, false);
        for val in table {
            write_leb(&mut self.bytes, *val, false);
        }
    }

    pub fn write_block_instruction(mut self, opcode: Opcode, block_type: BlockType) -> Self {
        match InstructionCategory::from_opcode(opcode) {
            InstructionCategory::Block(allow_else) => {
                let require_else = allow_else && block_type != BlockType::None;

                write_opcode(&mut self, opcode);
                self.append_byte(block_type.into());

                self.state_stack.push(ExpressionWriterStateStack {
                    allow_else,
                    require_else,
                });
                self
            }
            _ => panic!("Invalid instruction category - only block instructions"),
        }
    }

    pub fn do_else(mut self) -> Self {
        {
            let ExpressionWriterStateStack {
                allow_else,
                require_else,
                ..
            } = self.state_stack.last_mut().unwrap();
            assert!(*allow_else);
            *allow_else = false;
            *require_else = false;
        }

        write_opcode(&mut self, Opcode::Else);
        self
    }

    pub fn do_end(mut self) -> Self {
        let ExpressionWriterStateStack { require_else, .. } = self.state_stack.last().unwrap();
        assert!(!*require_else);

        write_opcode(&mut self, Opcode::End);

        self.state_stack.pop();
        self
    }
}

use crate::core::{stack_entry::StackEntry, BlockType};
use crate::parser::{InstructionCategory, InstructionSource, Opcode};

use std::convert::TryInto;

pub trait ExpressionWriter: Sized {
    fn append_bytes(&mut self, bytes: &[u8]);

    fn append_byte(&mut self, byte: u8) {
        self.append_bytes(&[byte]);
    }

    fn write_const_instruction(&mut self, val: impl Into<StackEntry>) {
        write_const_instruction(self, val.into());
    }

    fn write_single_byte_instruction(&mut self, opcode: Opcode) {
        assert!(InstructionCategory::from_opcode(opcode) == InstructionCategory::SingleByte);
        write_opcode(self, opcode);
    }

    fn write_single_leb_instruction(&mut self, opcode: Opcode, val: u64) {
        assert!(InstructionCategory::from_opcode(opcode) == InstructionCategory::SingleLebInteger);
        write_opcode(self, opcode);
        write_leb(self, val, false);
    }

    fn write_two_leb_instruction(&mut self, opcode: Opcode, val1: u64, val2: u64) {
        assert!(InstructionCategory::from_opcode(opcode) == InstructionCategory::TwoLebInteger);
        write_opcode(self, opcode);
        write_leb(self, val1, false);
        write_leb(self, val2, false);
    }

    fn write_block_instruction(
        mut self,
        opcode: Opcode,
        block_type: BlockType,
    ) -> NestedExpressionWriter<Self> {
        match InstructionCategory::from_opcode(opcode) {
            InstructionCategory::Block(allow_else) => {
                let require_else = allow_else && block_type != BlockType::None;

                write_opcode(&mut self, opcode);
                self.append_byte(block_type.into());

                NestedExpressionWriter {
                    parent: self,
                    allow_else,
                    require_else,
                }
            }
            _ => panic!("Invalid instruction category - only block instructions"),
        }
    }
}

impl ExpressionWriter for Vec<u8> {
    fn append_bytes(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}

fn write_leb(expr_bytes: &mut impl ExpressionWriter, val: u64, signed: bool) {
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
    expr_bytes.append_bytes(&encoded_bytes[0..required_length]);
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

fn write_opcode(expr_bytes: &mut impl ExpressionWriter, opcode: Opcode) {
    expr_bytes.append_byte(opcode.into());
}

fn write_const_instruction(expr_bytes: &mut impl ExpressionWriter, val: StackEntry) {
    match val {
        StackEntry::I32Entry(i) => {
            expr_bytes.append_byte(Opcode::I32Const.into());
            write_leb(expr_bytes, i.into(), true);
        }
        StackEntry::I64Entry(i) => {
            expr_bytes.append_byte(Opcode::I64Const.into());
            write_leb(expr_bytes, i.into(), true);
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

pub struct NestedExpressionWriter<T: ExpressionWriter> {
    parent: T,
    allow_else: bool,
    require_else: bool,
}

impl<T: ExpressionWriter> ExpressionWriter for NestedExpressionWriter<T> {
    fn append_bytes(&mut self, bytes: &[u8]) {
        self.parent.append_bytes(bytes);
    }
}

impl<T: ExpressionWriter> NestedExpressionWriter<T> {
    pub fn do_else(mut self) -> Self {
        assert!(self.allow_else);

        write_opcode(&mut self, Opcode::Else);

        Self {
            parent: self.parent,
            allow_else: false,
            require_else: false,
        }
    }

    pub fn do_end(mut self) -> T {
        assert!(!self.require_else);

        write_opcode(&mut self, Opcode::End);

        self.parent
    }
}

pub fn make_expression_writer() -> impl ExpressionWriter + InstructionSource {
    Vec::new()
}

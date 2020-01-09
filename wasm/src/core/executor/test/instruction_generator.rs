use crate::core::stack_entry::StackEntry;
use crate::parser::Opcode;

use std::convert::TryInto;

pub fn write_leb(expr_bytes: &mut Vec<u8>, val: u64, signed: bool) {
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

pub fn write_leb_as_vector(i: u64, signed: bool) -> Vec<u8> {
    let mut vec = Vec::new();
    write_leb(&mut vec, i, signed);
    vec
}

pub fn write_signed_leb_as_vector(i: u64) -> Vec<u8> {
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

pub fn write_const_instruction(expr_bytes: &mut Vec<u8>, val: StackEntry) {
    match val {
        StackEntry::I32Entry(i) => {
            expr_bytes.push(Opcode::I32Const.into());
            write_leb(expr_bytes, i.into(), true);
        }
        StackEntry::I64Entry(i) => {
            expr_bytes.push(Opcode::I64Const.into());
            write_leb(expr_bytes, i.into(), true);
        }
        StackEntry::F32Entry(i) => {
            expr_bytes.push(Opcode::F32Const.into());
            expr_bytes.extend_from_slice(&i.to_le_bytes());
        }
        StackEntry::F64Entry(i) => {
            expr_bytes.push(Opcode::F64Const.into());
            expr_bytes.extend_from_slice(&i.to_le_bytes());
        }

        _ => panic!("Unsupported stack entry type"),
    }
}

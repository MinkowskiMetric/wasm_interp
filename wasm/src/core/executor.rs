use std::io;

use crate::core::{stack_entry::StackEntry, stack_entry::StackEntryValueType, Stack};
use crate::parser::{self, Opcode};

fn convert_stack_entry_to_value<ParamType: StackEntryValueType>(
    e: StackEntry,
) -> io::Result<ParamType> {
    // This is only necessary because I don't have a clean sensible error handling strategy
    match ParamType::try_into_value(e) {
        Ok(v) => Ok(v),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Failed to convert stack value",
        )),
    }
}

fn binary_op<ParamType: StackEntryValueType, Func: Fn(ParamType, ParamType) -> ParamType>(
    stack: &mut Stack,
    func: Func,
) -> io::Result<()> {
    if stack.working_count() < 2 {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Not enough items on stack for op",
        ))
    } else {
        let ops = stack.working_top(2);
        let ret = func(
            convert_stack_entry_to_value(ops[0])?,
            convert_stack_entry_to_value(ops[1])?,
        );

        stack.push(ret.from_value());
        stack.drop_entries(2, 1);
        Ok(())
    }
}

pub struct ConstantExpressionExecutor {}
pub struct ExpressionExecutor {}

static CONSTANT_EXPRESSION_EXECUTOR_INSTANCE: ConstantExpressionExecutor =
    ConstantExpressionExecutor {};
static EXPRESSION_EXECUTOR_INSTANCE: ExpressionExecutor = ExpressionExecutor {};

pub trait ConstantExpressionStore {
    fn get_global_value(&self, idx: usize) -> io::Result<StackEntry>;
}

pub trait ExpressionStore: ConstantExpressionStore {
    fn set_global_value(&mut self, idx: usize, value: StackEntry) -> io::Result<()>;
}

impl ConstantExpressionExecutor {
    pub fn instance() -> &'static Self {
        &CONSTANT_EXPRESSION_EXECUTOR_INSTANCE
    }

    // Not totally sure on the return type here right now.
    pub fn execute_constant_expression<
        ExprType: parser::InstructionSource,
        StoreType: ConstantExpressionStore,
    >(
        &self,
        expr: &ExprType,
        store: &StoreType,
        arity: usize,
    ) -> io::Result<Vec<StackEntry>> {
        let mut stack = Stack::new();

        for instruction in expr.iter() {
            let instruction = instruction?;

            match instruction.opcode() {
                // There is only a very limited set of instructions that are allowed in a constant expression
                Opcode::I32Const => {
                    stack.push(instruction.get_single_i32_arg().into());
                }
                Opcode::I64Const => {
                    stack.push(instruction.get_single_i64_arg().into());
                }
                Opcode::F32Const => {
                    stack.push(instruction.get_single_f32_arg().into());
                }
                Opcode::F64Const => {
                    stack.push(instruction.get_single_f64_arg().into());
                }

                Opcode::GlobalGet => {
                    stack.push(store.get_global_value(instruction.get_single_u32_as_usize_arg())?);
                }

                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Instruction is not valid in constant expression",
                    ));
                }
            }
        }

        if stack.working_count() < arity {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not enough values returned by constant expression",
            ));
        }

        Ok(stack.frame()[stack.working_limit() - arity..stack.working_limit()].to_vec())
    }
}

impl ExpressionExecutor {
    pub fn instance() -> &'static Self {
        &EXPRESSION_EXECUTOR_INSTANCE
    }

    fn get_stack_top(stack: &mut Stack, n: usize) -> io::Result<&[StackEntry]> {
        if stack.working_count() < n {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Not enough values on stack",
            ))
        } else {
            Ok(stack.working_top(n))
        }
    }

    pub fn execute<ExprType: parser::InstructionSource, StoreType: ExpressionStore>(
        &self,
        expr: &ExprType,
        stack: &mut Stack,
        store: &mut StoreType,
    ) -> io::Result<()> {
        for instruction in expr.iter() {
            let instruction = instruction?;

            match instruction.opcode() {
                // There is only a very limited set of instructions that are allowed in a constant expression
                Opcode::I32Const => {
                    stack.push(instruction.get_single_i32_arg().into());
                }
                Opcode::I64Const => {
                    stack.push(instruction.get_single_i64_arg().into());
                }
                Opcode::F32Const => {
                    stack.push(instruction.get_single_f32_arg().into());
                }
                Opcode::F64Const => {
                    stack.push(instruction.get_single_f64_arg().into());
                }

                Opcode::GlobalGet => {
                    stack.push(store.get_global_value(instruction.get_single_u32_as_usize_arg())?)
                }
                Opcode::GlobalSet => {
                    let arg = Self::get_stack_top(stack, 1)?[0];
                    stack.pop();

                    store.set_global_value(instruction.get_single_u32_as_usize_arg(), arg)?;
                }

                Opcode::I32Add => binary_op(stack, |a: u32, b| a.wrapping_add(b))?,

                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!(
                            "Instruction {:?} is not valid in constant expression",
                            instruction
                        ),
                    ));
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::convert::TryInto;

    struct TestStore {}

    impl TestStore {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl ConstantExpressionStore for TestStore {
        fn get_global_value(&self, _idx: usize) -> io::Result<StackEntry> {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Global value not present in test store",
            ))
        }
    }

    impl ExpressionStore for TestStore {
        fn set_global_value(&mut self, _idx: usize, _value: StackEntry) -> io::Result<()> {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Global value not present in test store",
            ))
        }
    }

    fn write_signed_leb(expr_bytes: &mut Vec<u8>, val: u64) {
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

        let is_positive = 0 == (val & 0x8000000000000000);

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

    fn write_signed_leb_as_vector(i: u64) -> Vec<u8> {
        let mut vec = Vec::new();
        write_signed_leb(&mut vec, i);
        vec
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

    fn write_const_instruction(expr_bytes: &mut Vec<u8>, val: StackEntry) {
        match val {
            StackEntry::I32Entry(i) => {
                expr_bytes.push(Opcode::I32Const.into());
                write_signed_leb(expr_bytes, i.into());
            }
            StackEntry::I64Entry(i) => {
                expr_bytes.push(Opcode::I64Const.into());
                write_signed_leb(expr_bytes, i.into());
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

    fn test_constant_opcode_impl(p1: StackEntry) -> Option<StackEntry> {
        // Allocate a byte vector and generate an instruction stream that will execute the op
        let mut expr_bytes: Vec<u8> = Vec::new();
        write_const_instruction(&mut expr_bytes, p1);

        // Now we need a stack and a store to run the op against
        let mut stack = Stack::new();
        let mut test_store = TestStore::new();

        if let Err(_) =
            ExpressionExecutor::instance().execute(&expr_bytes, &mut stack, &mut test_store)
        {
            None
        } else {
            if stack.working_count() == 1 {
                Some(stack.working_top(1)[0])
            } else {
                None
            }
        }
    }

    macro_rules! test_constant_opcode {
        ($p1:expr) => {
            assert_eq!(test_constant_opcode_impl($p1.into()), Some($p1.into()));
        };
    }

    fn test_binary_opcode_impl(
        p1: StackEntry,
        p2: StackEntry,
        opcode: Opcode,
    ) -> Option<StackEntry> {
        // Allocate a byte vector and generate an instruction stream that will execute the op
        let mut expr_bytes: Vec<u8> = Vec::new();
        write_const_instruction(&mut expr_bytes, p1);
        write_const_instruction(&mut expr_bytes, p2);
        expr_bytes.push(opcode.into());

        // Now we need a stack and a store to run the op against
        let mut stack = Stack::new();
        let mut test_store = TestStore::new();

        if let Err(_) =
            ExpressionExecutor::instance().execute(&expr_bytes, &mut stack, &mut test_store)
        {
            None
        } else {
            if stack.working_count() == 1 {
                Some(stack.working_top(1)[0])
            } else {
                None
            }
        }
    }

    macro_rules! test_binary_opcode {
        ($p1:expr, $p2:expr, $opcode:expr, $r:expr) => {
            assert_eq!(
                test_binary_opcode_impl($p1.into(), $p2.into(), $opcode),
                Some($r.into())
            );
        };
    }

    #[test]
    fn test_opcodes() {
        test_constant_opcode!(0i32);
        test_constant_opcode!(1i32);
        test_constant_opcode!(2i32);
        test_constant_opcode!(-1i32);
        test_constant_opcode!(-256i32);
        test_constant_opcode!(-65536i32);
        test_constant_opcode!(0u32);
        test_constant_opcode!(1u32);
        test_constant_opcode!(2u32);
        test_constant_opcode!(256u32);
        test_constant_opcode!(0i64);
        test_constant_opcode!(0u64);
        test_constant_opcode!(0.0f32);
        test_constant_opcode!(0.0f64);
        // TODOTODOTODO - should test for integers that are too big for the opcodes to make sure they're handled properly.
        // I haven't written that test because currently it panics in the instruction accumulator which is obviously not the
        // right thing to do.

        test_binary_opcode!(7i32, 8i32, Opcode::I32Add, 15u32);
        test_binary_opcode!(7i32, -1i32, Opcode::I32Add, 6u32);
    }
}

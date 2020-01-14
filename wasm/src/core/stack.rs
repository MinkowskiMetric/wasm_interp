use crate::core::{stack_entry::StackEntry, FuncType, Locals, ValueType};
use anyhow::{anyhow, Result};

struct LocalsFlatteningIterator<'a, T: Iterator<Item = &'a Locals>> {
    iter: T,
    current: Option<&'a Locals>,
    remaining: u32,
}

impl<'a, T: Iterator<Item = &'a Locals>> Iterator for LocalsFlatteningIterator<'a, T> {
    type Item = Locals;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self {
                LocalsFlatteningIterator { current: None, .. }
                | LocalsFlatteningIterator {
                    current: Some(_),
                    remaining: 0,
                    ..
                } => {
                    self.current = self.iter.next();
                    self.remaining = self.current.map_or(0, |l| l.count());

                    // If we got none back from the parent, then stop looping now
                    if self.current.is_none() {
                        return None;
                    }
                }

                LocalsFlatteningIterator {
                    current: Some(locals),
                    remaining,
                    ..
                } => {
                    // Decrement the remaining count
                    self.remaining = *remaining - 1;

                    // Return a single local entry
                    return Some(Locals::new(1, locals.value_type()));
                }
            }
        }
    }
}

fn flatten_locals<'a, T: Iterator<Item = &'a Locals>>(iter: T) -> LocalsFlatteningIterator<'a, T> {
    LocalsFlatteningIterator {
        iter,
        current: None,
        remaining: 0,
    }
}

#[derive(Debug)]
struct StackLabel {
    sp: usize,
    arity: usize,
}

#[derive(Debug)]
pub struct StackFrame {
    sp: usize,
    parameter_count: usize,
    local_count: usize,
    label_stack: Vec<StackLabel>,
    return_types: Vec<ValueType>,
}

impl StackFrame {
    pub fn new(
        sp: usize,
        parameter_count: usize,
        local_count: usize,
        return_types: Vec<ValueType>,
    ) -> Self {
        Self {
            sp,
            parameter_count,
            local_count,
            label_stack: Vec::new(),
            return_types,
        }
    }

    pub fn frame_base(&self) -> usize {
        self.sp
    }

    pub fn parameter_base(&self) -> usize {
        self.sp
    }

    pub fn parameter_count(&self) -> usize {
        self.parameter_count
    }

    pub fn parameter_limit(&self) -> usize {
        self.sp + self.parameter_count()
    }

    pub fn local_base(&self) -> usize {
        self.parameter_limit()
    }

    pub fn local_count(&self) -> usize {
        self.local_count
    }

    pub fn local_limit(&self) -> usize {
        self.parameter_limit() + self.local_count()
    }

    pub fn working_base(&self) -> usize {
        match self.label_stack.last() {
            Some(label) => label.sp,
            _ => self.local_limit(),
        }
    }

    pub fn push_label(&mut self, sp: usize, arity: usize) {
        assert!(sp >= self.working_base());
        self.label_stack.push(StackLabel { sp, arity });
    }

    pub fn pop_n_labels(&mut self, count: usize) -> (usize, usize) {
        assert!(self.label_stack.len() >= count);

        let last_entry_idx = self.label_stack.len() - count;
        let StackLabel { sp, arity } = self.label_stack[last_entry_idx];

        // Then we simply resize the label stack to truncate it
        self.label_stack.truncate(last_entry_idx);

        (sp, arity)
    }

    #[allow(dead_code)]
    pub fn label_arity(&self) -> usize {
        match self.label_stack.last() {
            Some(label) => label.arity,
            _ => panic!("No label"),
        }
    }
}

#[derive(Debug)]
pub struct Stack {
    frames: Vec<StackFrame>,
    entries: Vec<StackEntry>,
}

impl Stack {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Stack {
            frames: Vec::new(),
            entries: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[allow(dead_code)]
    pub fn height(&self) -> usize {
        self.entries.len()
    }

    fn last_frame<R: Default, F: Fn(&StackFrame) -> R>(&self, func: F) -> R {
        match self.frames.last() {
            Some(f) => func(f),
            None => Default::default(),
        }
    }

    #[allow(dead_code)]
    pub fn frame_base(&self) -> usize {
        self.last_frame(|f| f.frame_base())
    }

    #[allow(dead_code)]
    pub fn parameter_base(&self) -> usize {
        self.last_frame(|f| f.parameter_base())
    }

    #[allow(dead_code)]
    pub fn parameter_count(&self) -> usize {
        self.last_frame(|f| f.parameter_count())
    }

    #[allow(dead_code)]
    pub fn parameter_limit(&self) -> usize {
        self.last_frame(|f| f.parameter_limit())
    }

    #[allow(dead_code)]
    pub fn local_base(&self) -> usize {
        self.last_frame(|f| f.local_base())
    }

    #[allow(dead_code)]
    pub fn local_count(&self) -> usize {
        self.last_frame(|f| f.local_count())
    }

    #[allow(dead_code)]
    pub fn local_limit(&self) -> usize {
        self.last_frame(|f| f.local_limit())
    }

    #[allow(dead_code)]
    pub fn working_base(&self) -> usize {
        self.last_frame(|f| f.working_base())
    }

    #[allow(dead_code)]
    pub fn working_count(&self) -> usize {
        self.height() - self.working_base()
    }

    #[allow(dead_code)]
    pub fn working_limit(&self) -> usize {
        self.height()
    }

    #[allow(dead_code)]
    pub fn frame_limit(&self) -> usize {
        self.height()
    }

    #[allow(dead_code)]
    pub fn frame(&self) -> &[StackEntry] {
        let (base, limit) = (self.frame_base(), self.frame_limit());
        &self.entries[base..limit]
    }

    #[allow(dead_code)]
    pub fn frame_mut(&mut self) -> &mut [StackEntry] {
        let (base, limit) = (self.frame_base(), self.frame_limit());
        &mut self.entries[base..limit]
    }

    #[allow(dead_code)]
    pub fn local(&self) -> &[StackEntry] {
        let (base, limit) = (self.parameter_base(), self.local_limit());
        &self.entries[base..limit]
    }

    #[allow(dead_code)]
    pub fn local_mut(&mut self) -> &mut [StackEntry] {
        let (base, limit) = (self.parameter_base(), self.local_limit());
        &mut self.entries[base..limit]
    }

    pub fn working_top(&self, n: usize) -> &[StackEntry] {
        assert!(self.working_count() >= n);
        let (base, limit) = (self.working_limit() - n, self.working_limit());
        &self.entries[base..limit]
    }

    #[allow(dead_code)]
    pub fn push(&mut self, entry: StackEntry) {
        self.entries.push(entry);
    }

    #[allow(dead_code)]
    pub fn push_from_slice(&mut self, entries: &[StackEntry]) {
        self.entries.extend_from_slice(entries);
    }

    #[allow(dead_code)]
    pub fn pop(&mut self) {
        assert!(self.working_count() > 0);

        self.entries.pop();
    }

    #[allow(dead_code)]
    pub fn pop_n(&mut self, n: usize) {
        assert!(self.working_count() >= n);

        self.entries.truncate(self.entries.len() - n);
    }

    pub fn drop_entries(&mut self, to_drop: usize, arity: usize) {
        assert!(self.working_count() >= to_drop + arity);

        let old_result_base = self.working_limit() - arity;
        let new_result_base = old_result_base - to_drop;

        let new_len = self.entries.len() - to_drop;

        for i in 0..arity {
            self.entries[new_result_base + i] = self.entries[old_result_base + i];
        }

        self.entries.truncate(new_len);
    }

    #[cfg(test)]
    pub fn push_test_frame(&mut self, local_count: u32) -> Result<()> {
        let func_type = FuncType::new(vec![], vec![]);
        let locals = vec![Locals::new(local_count, ValueType::I32)];
        self.push_typed_frame(&func_type, &locals)
    }

    pub fn push_typed_frame(&mut self, func_type: &FuncType, locals: &Vec<Locals>) -> Result<()> {
        let arg_count = func_type.arg_types().len();
        let local_count = locals.iter().map(|l| l.count() as usize).sum();
        if arg_count > self.working_count() {
            Err(anyhow!("Not enough arguments on working stack"))
        } else {
            let working_params = self.working_top(arg_count);
            let matched_args: Result<Vec<_>> = func_type
                .arg_types()
                .iter()
                .enumerate()
                .zip(working_params)
                .map(|((idx, arg_type), stack_entry)| -> Result<()> {
                    match (idx, arg_type, stack_entry) {
                        (_, ValueType::I32, StackEntry::I32Entry(_))
                        | (_, ValueType::I64, StackEntry::I64Entry(_))
                        | (_, ValueType::F32, StackEntry::F32Entry(_))
                        | (_, ValueType::F64, StackEntry::F64Entry(_)) => Ok(()),
                        (idx, ..) => Err(anyhow!("Argument {} type does not match", idx)),
                    }
                })
                .collect();

            match matched_args {
                Err(e) => Err(e),
                _ => {
                    let frame = StackFrame::new(
                        self.height() - arg_count,
                        arg_count,
                        local_count,
                        func_type.return_types().clone(),
                    );

                    // Push on zeroed out entries for the locals
                    for (_, l) in flatten_locals(locals.iter()).enumerate() {
                        debug_assert!(l.count() == 1);
                        self.push(match l.value_type() {
                            ValueType::I32 => StackEntry::I32Entry(0),
                            ValueType::I64 => StackEntry::I64Entry(0),
                            ValueType::F32 => StackEntry::F32Entry(0.0),
                            ValueType::F64 => StackEntry::F64Entry(0.0),
                        });
                    }

                    // Now push the frame
                    self.frames.push(frame);

                    Ok(())
                }
            }
        }
    }

    pub fn pop_typed_frame(&mut self) -> Result<()> {
        let last_frame = self.frames.last().unwrap();
        let return_types = &last_frame.return_types;

        if self.working_count() < return_types.len() {
            Err(anyhow!("Insufficient return values"))
        } else {
            let working_ret = self.working_top(return_types.len());
            let matched_ret: Result<Vec<_>> = return_types
                .iter()
                .enumerate()
                .zip(working_ret)
                .map(|((idx, arg_type), stack_entry)| -> Result<()> {
                    match (idx, arg_type, stack_entry) {
                        (_, ValueType::I32, StackEntry::I32Entry(_))
                        | (_, ValueType::I64, StackEntry::I64Entry(_))
                        | (_, ValueType::F32, StackEntry::F32Entry(_))
                        | (_, ValueType::F64, StackEntry::F64Entry(_)) => Ok(()),
                        (idx, ..) => Err(anyhow!("Argument {} type does not match", idx)),
                    }
                })
                .collect();

            match matched_ret {
                Err(e) => Err(e),
                _ => {
                    let arity = return_types.len();
                    let old_result_base = self.working_limit() - arity;
                    let new_result_base = self.frame_base();

                    let new_len = self.frame_base() + arity;

                    // Pop the frame entry off the stack now as we don't need it any more
                    self.frames.pop();

                    for i in 0..arity {
                        self.entries[new_result_base + i] = self.entries[old_result_base + i];
                    }

                    self.entries.truncate(new_len);

                    Ok(())
                }
            }
        }
    }

    pub fn push_label(&mut self, arity: usize) {
        let sp = self.height();
        self.frames.last_mut().unwrap().push_label(sp, arity);
    }

    pub fn pop_n_labels(&mut self, count: usize) {
        // We ask the frame to drop the labels and tell us how to fix up the
        // stack
        let (sp, arity) = self.frames.last_mut().unwrap().pop_n_labels(count);
        self.drop_entries((self.height() - sp) - arity, arity);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::convert::TryFrom;

    fn push_test_frame(
        stack: &mut Stack,
        params: &[ValueType],
        local_count: u32,
        ret_types: &[ValueType],
    ) -> Result<()> {
        let params: Vec<_> = params.iter().map(|x| x.clone()).collect();
        let func_type = FuncType::new(params, ret_types.iter().map(|x| x.clone()).collect());
        let locals = vec![Locals::new(local_count, ValueType::I32)];

        stack.push_typed_frame(&func_type, &locals)
    }

    fn check_stack_ranges(stack: &Stack) -> (usize, usize, usize, usize) {
        assert_eq!(stack.frame_base(), stack.parameter_base());

        let parameter_count = stack.parameter_count();
        assert_eq!(
            stack.parameter_limit(),
            stack.parameter_base() + parameter_count
        );
        assert_eq!(stack.local_base(), stack.parameter_limit());

        let local_count = stack.local_count();
        assert_eq!(stack.local_limit(), stack.local_base() + local_count);

        let hidden_working_count = stack.working_base() - stack.local_limit();

        let working_count = stack.working_count();
        assert_eq!(stack.working_limit(), stack.working_base() + working_count);
        assert_eq!(stack.frame_limit(), stack.working_limit());
        assert_eq!(stack.height(), stack.frame_limit());

        (
            parameter_count,
            local_count,
            hidden_working_count,
            working_count,
        )
    }

    #[test]
    fn test_empty_stack() {
        let stack = Stack::new();

        assert_eq!(stack.is_empty(), true);
        assert_eq!(stack.frame_base(), 0);
        assert_eq!(check_stack_ranges(&stack), (0, 0, 0, 0));
    }

    #[test]
    fn test_no_parameters_frame() {
        let mut stack = Stack::new();
        assert!(push_test_frame(&mut stack, &[], 4, &[]).is_ok());

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 0);
        assert_eq!(check_stack_ranges(&stack), (0, 4, 0, 0));

        // Validate that the locals are all currently "Unused"
        assert_eq!(stack.frame().len(), 4);
        assert_eq!(stack.frame_mut().len(), 4);
        assert_eq!(stack.local().len(), 4);
        assert_eq!(stack.local_mut().len(), 4);

        // Modify the locals
        for i in 0..4 {
            assert_eq!(stack.frame()[i], StackEntry::I32Entry(0));
            assert!(std::ptr::eq(&stack.frame()[i], &stack.local()[i]));

            stack.local_mut()[i] = u32::try_from(i).unwrap().into();
            assert_eq!(stack.frame()[i], u32::try_from(i).unwrap().into());
            assert_eq!(stack.local()[i], u32::try_from(i).unwrap().into());
        }

        // Now push some entries
        stack.push(StackEntry::I32Entry(4));

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 0);
        assert_eq!(check_stack_ranges(&stack), (0, 4, 0, 1));

        stack.push_from_slice(&[
            StackEntry::I32Entry(5),
            StackEntry::I32Entry(6),
            StackEntry::I32Entry(7),
        ]);

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 0);
        assert_eq!(check_stack_ranges(&stack), (0, 4, 0, 4));

        // Verify that the locals are unaffected
        for i in 0..4 {
            assert_eq!(stack.local()[i], u32::try_from(i).unwrap().into());
            assert!(std::ptr::eq(&stack.frame()[i], &stack.local()[i]));
        }

        // Verify the new entries are all correct
        for i in 4..8 {
            assert_eq!(stack.frame()[i], u32::try_from(i).unwrap().into());
        }

        // Now pop an entry
        stack.pop();

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 0);
        assert_eq!(check_stack_ranges(&stack), (0, 4, 0, 3));

        for i in 0..4 {
            assert_eq!(stack.local()[i], u32::try_from(i).unwrap().into());
            assert!(std::ptr::eq(&stack.frame()[i], &stack.local()[i]));
        }

        for i in 4..7 {
            assert_eq!(stack.frame()[i], u32::try_from(i).unwrap().into());
        }

        // Now push another entry
        stack.push(32.0f32.into());

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 0);
        assert_eq!(check_stack_ranges(&stack), (0, 4, 0, 4));

        assert_eq!(stack.frame()[7], 32.0f32.into());

        // Now pop n entries
        stack.pop_n(2);

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 0);
        assert_eq!(check_stack_ranges(&stack), (0, 4, 0, 2));

        for i in 4..6 {
            assert_eq!(stack.frame()[i], u32::try_from(i).unwrap().into());
        }

        // Push a "result" entry
        stack.push(32.0f64.into());

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 0);
        assert_eq!(check_stack_ranges(&stack), (0, 4, 0, 3));

        // Now replace the top entries with that one entry
        stack.drop_entries(2, 1);

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 0);
        assert_eq!(check_stack_ranges(&stack), (0, 4, 0, 1));

        assert_eq!(stack.frame()[4], 32.0f64.into());

        // Now push another frame, this time taking one parameter
        assert!(push_test_frame(&mut stack, &[ValueType::F64], 4, &[ValueType::F64]).is_ok());

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 4);
        assert_eq!(check_stack_ranges(&stack), (1, 4, 0, 0));
        assert_eq!(stack.local()[0], 32f64.into());

        // Now add a return value
        stack.push(42f64.into());

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 4);
        assert_eq!(check_stack_ranges(&stack), (1, 4, 0, 1));
        assert_eq!(stack.frame()[5], 42f64.into());

        // Now pop the frame
        assert!(stack.pop_typed_frame().is_ok());

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 0);
        assert_eq!(check_stack_ranges(&stack), (0, 4, 0, 1));
        assert_eq!(stack.frame()[4], 42f64.into());

        // Now push some constants
        stack.push(42f64.into());
        stack.push(42f64.into());
        stack.push(42f64.into());
        assert_eq!(check_stack_ranges(&stack), (0, 4, 0, 4));

        // Now push a label with arity of 2
        stack.push_label(2);
        assert_eq!(check_stack_ranges(&stack), (0, 4, 4, 0));

        // Locals should be unchanged
        for i in 0..4 {
            assert!(std::ptr::eq(&stack.frame()[i], &stack.local()[i]));
            assert_eq!(stack.local()[i], u32::try_from(i).unwrap().into());
        }

        // Pushing three values should be fine
        stack.push(0f64.into());
        stack.push(43f64.into());
        stack.push(44f64.into());
        assert_eq!(check_stack_ranges(&stack), (0, 4, 4, 3));
        assert_eq!(
            stack.working_top(3),
            [0f64.into(), 43f64.into(), 44f64.into()]
        );

        // Now pop the label
        stack.pop_n_labels(1);
        assert_eq!(check_stack_ranges(&stack), (0, 4, 0, 6));

        assert_eq!(
            stack.working_top(3),
            [42f64.into(), 43f64.into(), 44f64.into()]
        );
    }

    #[test]
    fn test_typed_frame() {
        let func_type = FuncType::new(
            vec![ValueType::I64, ValueType::F32],
            vec![ValueType::F64, ValueType::F64],
        );
        let locals: Vec<Locals> = vec![
            Locals::new(3, ValueType::I64),
            Locals::new(2, ValueType::F32),
        ];

        let mut stack = Stack::new();

        // This will fail because there are not enough arguments on the stack
        assert_eq!(check_stack_ranges(&stack), (0, 0, 0, 0));
        assert!(stack.push_typed_frame(&func_type, &locals).is_err());
        assert_eq!(check_stack_ranges(&stack), (0, 0, 0, 0));

        stack.push(0_u32.into());
        stack.push(0_u32.into());
        // This will fail because the parameters are the wrong types
        assert_eq!(check_stack_ranges(&stack), (0, 0, 0, 2));
        assert!(stack.push_typed_frame(&func_type, &locals).is_err());
        assert_eq!(check_stack_ranges(&stack), (0, 0, 0, 2));

        stack.pop_n(2);
        stack.push(17_i64.into());
        stack.push(18_f32.into());
        // Now it should be OK
        assert_eq!(check_stack_ranges(&stack), (0, 0, 0, 2));
        assert!(stack.push_typed_frame(&func_type, &locals).is_ok());
        assert_eq!(check_stack_ranges(&stack), (2, 5, 0, 0));

        // Check the locals have been initialized correctly
        assert_eq!(stack.local()[0], 17_i64.into());
        assert_eq!(stack.local()[1], 18_f32.into());
        assert_eq!(stack.local()[2], 0_u64.into());
        assert_eq!(stack.local()[3], 0_u64.into());
        assert_eq!(stack.local()[4], 0_u64.into());
        assert_eq!(stack.local()[5], 0_f32.into());
        assert_eq!(stack.local()[6], 0_f32.into());

        // At this point, popping the frame should fail because insufficient return values
        assert!(stack.pop_typed_frame().is_err());
        assert_eq!(check_stack_ranges(&stack), (2, 5, 0, 0));

        stack.push(0_u32.into());
        stack.push(0_u32.into());
        // This will fail because the parameters are the wrong type
        assert!(stack.pop_typed_frame().is_err());
        assert_eq!(check_stack_ranges(&stack), (2, 5, 0, 2));

        stack.pop_n(2);
        stack.push(26.0_f64.into());
        stack.push(52.0_f64.into());
        // This should succeed
        assert!(stack.pop_typed_frame().is_ok());
        assert_eq!(check_stack_ranges(&stack), (0, 0, 0, 2));
        assert_eq!(stack.working_top(2)[0], 26.0_f64.into());
        assert_eq!(stack.working_top(2)[1], 52.0_f64.into());
    }
}

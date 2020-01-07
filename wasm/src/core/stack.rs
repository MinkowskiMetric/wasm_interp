use crate::core::stack_entry::StackEntry;

#[derive(Debug)]
pub struct StackFrame {
    sp: usize,
    parameter_count: usize,
    local_count: usize,
}

impl StackFrame {
    pub fn new(sp: usize, parameter_count: usize, local_count: usize) -> Self {
        Self {
            sp,
            parameter_count,
            local_count,
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
}

#[derive(Debug)]
pub struct Stack {
    frames: Vec<StackFrame>,
    entries: Vec<StackEntry>,
    stack_height: usize,
}

impl Stack {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Stack {
            frames: Vec::new(),
            entries: Vec::new(),
            stack_height: 0,
        }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.stack_height == 0
    }

    #[allow(dead_code)]
    pub fn height(&self) -> usize {
        self.stack_height
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
        self.local_limit()
    }

    #[allow(dead_code)]
    pub fn working_count(&self) -> usize {
        self.height() - self.local_limit()
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
        // Ensure that there is room for the item
        self.ensure_entries(1);

        // Place the entry on the stack, then increment the height
        self.entries[self.stack_height] = entry;
        self.stack_height += 1;
    }

    #[allow(dead_code)]
    pub fn push_from_slice(&mut self, entries: &[StackEntry]) {
        self.ensure_entries(entries.len());

        self.entries[self.stack_height..].copy_from_slice(entries);
        self.stack_height += entries.len();
    }

    #[allow(dead_code)]
    pub fn pop(&mut self) {
        assert!(self.working_count() > 0);

        self.stack_height -= 1;
        self.entries[self.stack_height] = StackEntry::Unused;
    }

    #[allow(dead_code)]
    pub fn pop_n(&mut self, n: usize) {
        assert!(self.working_count() >= n);

        self.stack_height -= n;
        for e in self.entries[self.stack_height..self.stack_height + n].iter_mut() {
            *e = StackEntry::Unused;
        }
    }

    #[allow(dead_code)]
    pub fn drop_entries(&mut self, to_drop: usize, arity: usize) {
        assert!(self.working_count() >= to_drop + arity);

        let old_result_base = self.working_limit() - arity;
        let new_result_base = old_result_base - to_drop;

        self.stack_height -= to_drop;

        for i in 0..arity {
            self.entries[new_result_base + i] = self.entries[old_result_base + i];
        }

        for i in arity..arity + to_drop {
            self.entries[new_result_base + i] = StackEntry::Unused;
        }
    }

    #[allow(dead_code)]
    pub fn push_frame(&mut self, parameter_count: usize, local_count: usize) {
        assert!(self.working_count() >= parameter_count);

        let frame = StackFrame::new(
            self.height() - parameter_count,
            parameter_count,
            local_count,
        );

        // For safety, we ensure that there are enough entries on the stack before
        // we push the frame - otherwise if pushing the frame were to fail we could
        // end up in an inconsistent state
        self.ensure_entries(local_count);

        // Now push the frame
        self.frames.push(frame);

        // Finally, update the stack height. This can't fail since it is just an add.
        self.stack_height += local_count;
    }

    #[allow(dead_code)]
    pub fn pop_frame(&mut self, arity: usize) {
        assert!(!self.frames.is_empty());
        assert!(self.working_count() >= arity);

        let old_result_base = self.working_limit() - arity;
        let new_result_base = self.frame_base();
        let clear_limit = self.frame_limit();

        self.stack_height = self.frame_base() + arity;

        // Pop the frame entry off the stack now as we don't need it any more
        self.frames.pop();

        for i in 0..arity {
            self.entries[new_result_base + i] = self.entries[old_result_base + i];
        }

        for i in new_result_base + arity..clear_limit {
            self.entries[i] = StackEntry::Unused;
        }
    }

    fn ensure_entries(&mut self, n: usize) {
        assert!(self.stack_height <= self.entries.len());

        let required_entries = self.stack_height + n;
        if required_entries > self.entries.len() {
            let entries_to_add = required_entries - self.entries.len();

            if entries_to_add > 0 {
                self.entries
                    .extend((0..entries_to_add).map(|_| StackEntry::Unused));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::convert::TryFrom;

    fn push_test_frame(stack: &mut Stack, parameter_count: usize, local_count: usize) {
        stack.push_frame(parameter_count, local_count)
    }

    fn check_stack_ranges(stack: &Stack) -> (usize, usize, usize) {
        assert_eq!(stack.frame_base(), stack.parameter_base());

        let parameter_count = stack.parameter_count();
        assert_eq!(
            stack.parameter_limit(),
            stack.parameter_base() + parameter_count
        );
        assert_eq!(stack.local_base(), stack.parameter_limit());

        let local_count = stack.local_count();
        assert_eq!(stack.local_limit(), stack.local_base() + local_count);
        assert_eq!(stack.working_base(), stack.local_limit());

        let working_count = stack.working_count();
        assert_eq!(stack.working_limit(), stack.working_base() + working_count);
        assert_eq!(stack.frame_limit(), stack.working_limit());
        assert_eq!(stack.height(), stack.frame_limit());

        (parameter_count, local_count, working_count)
    }

    #[test]
    fn test_empty_stack() {
        let stack = Stack::new();

        assert_eq!(stack.is_empty(), true);
        assert_eq!(stack.frame_base(), 0);
        assert_eq!(check_stack_ranges(&stack), (0, 0, 0));
    }

    #[test]
    fn test_no_parameters_frame() {
        let mut stack = Stack::new();
        push_test_frame(&mut stack, 0, 4);

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 0);
        assert_eq!(check_stack_ranges(&stack), (0, 4, 0));

        // Validate that the locals are all currently "Unused"
        assert_eq!(stack.frame().len(), 4);
        assert_eq!(stack.frame_mut().len(), 4);
        assert_eq!(stack.local().len(), 4);
        assert_eq!(stack.local_mut().len(), 4);

        // Modify the locals
        for i in 0..4 {
            assert_eq!(stack.frame()[i], StackEntry::Unused);
            assert!(std::ptr::eq(&stack.frame()[i], &stack.local()[i]));

            stack.local_mut()[i] = u32::try_from(i).unwrap().into();
            assert_eq!(stack.frame()[i], u32::try_from(i).unwrap().into());
            assert_eq!(stack.local()[i], u32::try_from(i).unwrap().into());
        }

        // Now push some entries
        stack.push(StackEntry::I32Entry(4));

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 0);
        assert_eq!(check_stack_ranges(&stack), (0, 4, 1));

        stack.push_from_slice(&[
            StackEntry::I32Entry(5),
            StackEntry::I32Entry(6),
            StackEntry::I32Entry(7),
        ]);

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 0);
        assert_eq!(check_stack_ranges(&stack), (0, 4, 4));

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
        assert_eq!(check_stack_ranges(&stack), (0, 4, 3));

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
        assert_eq!(check_stack_ranges(&stack), (0, 4, 4));

        assert_eq!(stack.frame()[7], 32.0f32.into());

        // Now pop n entries
        stack.pop_n(2);

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 0);
        assert_eq!(check_stack_ranges(&stack), (0, 4, 2));

        for i in 4..6 {
            assert_eq!(stack.frame()[i], u32::try_from(i).unwrap().into());
        }

        // Push a "result" entry
        stack.push(32.0f64.into());

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 0);
        assert_eq!(check_stack_ranges(&stack), (0, 4, 3));

        // Now replace the top entries with that one entry
        stack.drop_entries(2, 1);

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 0);
        assert_eq!(check_stack_ranges(&stack), (0, 4, 1));

        assert_eq!(stack.frame()[4], 32.0f64.into());

        // Now push another frame, this time taking one parameter
        push_test_frame(&mut stack, 1, 4);

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 4);
        assert_eq!(check_stack_ranges(&stack), (1, 4, 0));
        assert_eq!(stack.local()[0], 32f64.into());

        // Now add a return value
        stack.push(42f64.into());

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 4);
        assert_eq!(check_stack_ranges(&stack), (1, 4, 1));
        assert_eq!(stack.frame()[5], 42f64.into());

        // Now pop the frame
        stack.pop_frame(1);

        assert_eq!(stack.is_empty(), false);
        assert_eq!(stack.frame_base(), 0);
        assert_eq!(check_stack_ranges(&stack), (0, 4, 1));
        assert_eq!(stack.frame()[4], 42f64.into());
    }
}

pub mod execute_core;
pub mod memory_access;
pub mod stack_ops;
pub mod store_access;

pub use execute_core::{
    evaluate_constant_expression, execute_constant_expression, execute_expression,
};
pub use store_access::{ConstantExpressionStore, ExpressionStore};

#[cfg(test)]
mod test {
    mod instruction_generator;
    #[macro_use]
    mod instruction_test_helpers;
    mod instruction_tests;
    mod test_store;
}

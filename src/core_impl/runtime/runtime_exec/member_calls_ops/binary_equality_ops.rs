use super::*;

#[path = "binary_equality_binary_ops.rs"]
mod binary_equality_binary_ops;
#[path = "binary_equality_comparison_ops.rs"]
mod binary_equality_comparison_ops;
#[path = "binary_equality_membership_ops.rs"]
mod binary_equality_membership_ops;

impl Harness {
    pub(crate) fn collect_left_associative_binary_operands<'a>(
        expr: &'a Expr,
        op: BinaryOp,
    ) -> Vec<&'a Expr> {
        let mut right_operands = Vec::new();
        let mut cursor = expr;
        loop {
            match cursor {
                Expr::Binary {
                    left,
                    op: inner_op,
                    right,
                } if *inner_op == op => {
                    right_operands.push(right.as_ref());
                    cursor = left.as_ref();
                }
                _ => break,
            }
        }

        let mut out = Vec::with_capacity(right_operands.len() + 1);
        out.push(cursor);
        while let Some(operand) = right_operands.pop() {
            out.push(operand);
        }
        out
    }
}

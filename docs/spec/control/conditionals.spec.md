-- ============================================================================
-- DOL 2.0 CONDITIONAL CONTROL FLOW SPECIFICATION
-- ============================================================================
-- Module: spec.control.conditionals
-- Version: 1.0.0
-- Purpose: Conditional constructs with MLIR SCF lowering
-- ============================================================================

module spec.control.conditionals @ 1.0.0

exegesis {
  Conditional control flow constructs for DOL 2.0.

  DOL provides if/else if/else as both statements and expressions,
  enabling imperative and functional programming styles.

  All constructs lower to MLIR SCF (Structured Control Flow) dialect
  operations for optimization and parallel execution.
}

-- ============================================================================
-- IF STATEMENT
-- ============================================================================

construct if_statement {
  syntax {
    if <condition: Bool> <then_block: Block>
    [else_if <condition: Bool> <then_block: Block>]*
    [else <else_block: Block>]
  }

  exegesis {
    Evaluates conditions sequentially, executes first matching block.
    Statement form has type Void and does not produce a value.
    Else clause is optional in statement context.
  }

  examples {
    function classify(value: Int32) -> String {
      if value > 100 {
        return "high"
      } else if value > 50 {
        return "medium"
      } else {
        return "low"
      }
    }

    function divide(a: Int32, b: Int32) -> Optional<Int32> {
      if b == 0 {
        return None
      }
      return Some(a / b)
    }
  }

  mlir_lowering {
    dol_code: """
      if x > 0 { y = x * 2 } else { y = 0 }
    """

    mlir_ir: """
      %zero = arith.constant 0 : i32
      %cond = arith.cmpi sgt, %x, %zero : i32

      scf.if %cond {
        %two = arith.constant 2 : i32
        %result = arith.muli %x, %two : i32
        memref.store %result, %y_ref[] : memref<i32>
      } else {
        memref.store %zero, %y_ref[] : memref<i32>
      }
    """
  }
}

-- ============================================================================
-- IF EXPRESSION
-- ============================================================================

construct if_expression {
  syntax {
    if <condition: Bool> { <then_expr: Expr> } else { <else_expr: Expr> }
  }

  exegesis {
    Expression form that produces a value. Both branches must have
    compatible types. Else clause is mandatory in expression context.

    Type: if cond { T } else { T } -> T
  }

  examples {
    function abs(x: Int32) -> Int32 {
      return if x < 0 { -x } else { x }
    }

    function sign(x: Int32) -> Int32 {
      return if x > 0 { 1 } else { if x < 0 { -1 } else { 0 } }
    }
  }

  mlir_lowering {
    dol_code: """
      result: Int32 = if x < 0 { -x } else { x }
    """

    mlir_ir: """
      %zero = arith.constant 0 : i32
      %cond = arith.cmpi slt, %x, %zero : i32

      %result = scf.if %cond -> i32 {
        %neg = arith.subi %zero, %x : i32
        scf.yield %neg : i32
      } else {
        scf.yield %x : i32
      }
    """
  }
}

-- ============================================================================
-- SHORT-CIRCUIT BOOLEAN OPERATORS
-- ============================================================================

construct boolean_operators {
  exegesis {
    DOL supports short-circuit evaluation:
    - `a and b`: If a is false, b is not evaluated
    - `a or b`: If a is true, b is not evaluated
    - `not a`: Logical negation

    Enables safe patterns like: `ptr != null and ptr.value > 0`
  }

  examples {
    function safe_divide(a: Int32, b: Int32) -> Optional<Int32> {
      if b != 0 and a % b == 0 {
        return Some(a / b)
      }
      return None
    }
  }

  mlir_lowering {
    note: """
      Short-circuit AND implemented as nested scf.if operations.
      Short-circuit OR implemented with scf.if/else.
    """
  }
}

-- ============================================================================
-- TYPE CONSTRAINTS
-- ============================================================================

exegesis {
  If Statement:
  - Condition: Bool (no implicit conversion from Int)
  - Branches: Any sequence of statements
  - Return type: Void
  - Else clause: Optional

  If Expression:
  - Condition: Bool
  - Both branches: Expression of type T
  - Return type: T (unified type)
  - Else clause: Mandatory
}

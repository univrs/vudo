-- ============================================================================
-- DOL 2.0 EXPRESSION VS STATEMENT CONTEXTS
-- ============================================================================
-- Module: spec.control.expressions
-- Version: 1.0.0
-- Purpose: Define expression and statement contexts in DOL
-- ============================================================================

module spec.control.expressions @ 1.0.0

exegesis {
  DOL distinguishes between expression and statement contexts.

  Expressions: Produce values, can be composed
  Statements: Perform actions, may not produce values

  Some constructs (if, match, blocks) can be either depending on context.
  This specification defines when each form applies.
}

-- ============================================================================
-- EXPRESSION CONTEXT
-- ============================================================================

expression_context {
  exegesis {
    Expression context occurs when a value is expected:
    - Right-hand side of assignment: x = <expr>
    - Function arguments: f(<expr>)
    - Return statements: return <expr>
    - Operators: <expr> + <expr>
    - Array/struct literals: [<expr>, <expr>]

    In expression context:
    - if/else requires else clause
    - match requires exhaustive coverage
    - Blocks evaluate to their final expression
  }

  examples {
    -- if as expression
    result: Int32 = if x > 0 { x } else { -x }

    -- match as expression
    name: String = match status {
      Status.Active => "active",
      Status.Inactive => "inactive",
    }

    -- Block as expression
    complex_result: Int32 = {
      a = compute_a()
      b = compute_b()
      a + b  -- Final expression is block's value
    }

    -- Nested expressions
    final = if cond1 {
      if cond2 { val1 } else { val2 }
    } else {
      val3
    }
  }
}

-- ============================================================================
-- STATEMENT CONTEXT
-- ============================================================================

statement_context {
  exegesis {
    Statement context occurs when no value is expected:
    - Top level of function body
    - Inside loop bodies
    - After semicolons (discards value)

    In statement context:
    - if/else doesn't require else
    - Expressions can be discarded with semicolon
    - Blocks evaluate all statements for side effects
  }

  examples {
    function process(items: Array<Int32>) -> Void {
      -- Statement: no else needed
      if items.is_empty() {
        return
      }

      -- Statement: loop body
      for item in items {
        -- Statement: function call for side effect
        print(item)
      }

      -- Expression discarded with semicolon
      expensive_computation();  -- Result discarded

      -- Statement: conditional without else
      if debug_mode {
        log("done")
      }
    }
  }
}

-- ============================================================================
-- BLOCK EXPRESSIONS
-- ============================================================================

construct block_expression {
  syntax {
    { <statement>* [<expression>] }
  }

  exegesis {
    Blocks are delimited by { }.

    In expression context:
    - If block ends with expression (no semicolon), that's the value
    - If block ends with statement (semicolon), value is Void

    Blocks create lexical scope for variable bindings.
  }

  examples {
    -- Block with value (expression)
    value: Int32 = {
      x = 10
      y = 20
      x + y  -- No semicolon: this is the value
    }

    -- Block without value (statement)
    {
      setup()
      initialize()
      configure()  -- Semicolon optional but implies Void
    };

    -- Nested blocks with scoping
    result: Int32 = {
      temp: Int32 = 100
      inner: Int32 = {
        temp * 2  -- Accesses outer temp
      }
      inner + temp
    }
    -- temp not accessible here
  }
}

-- ============================================================================
-- SEMICOLON RULES
-- ============================================================================

semicolon_rules {
  exegesis {
    Semicolons in DOL:

    1. REQUIRED after statements in statement context
    2. OPTIONAL after final expression in block (determines if value returned)
    3. Semicolon DISCARDS expression value (converts to statement)

    This enables both:
    - Traditional imperative style: stmt; stmt; stmt;
    - Expression-oriented style: { expr1; expr2; final_expr }
  }

  examples {
    -- Statements need semicolons
    function imperative() -> Void {
      x = 10;
      y = 20;
      print(x + y);
    }

    -- Expression blocks: no semicolon on final
    value = {
      compute_a();
      compute_b()  -- No semicolon: returned
    }

    -- Discarding values
    function ignore_result() -> Void {
      expensive_function();  -- Semicolon discards return value
    }
  }
}

-- ============================================================================
-- TYPE IMPLICATIONS
-- ============================================================================

type_implications {
  exegesis {
    Context affects type requirements:

    EXPRESSION CONTEXT:
    - Type must be non-Void (unless explicitly Void expected)
    - All branches must unify to common type
    - Exhaustiveness required for match

    STATEMENT CONTEXT:
    - Any type allowed (value discarded if not Void)
    - Branches can have different types
    - Side effects are the purpose
  }

  examples {
    -- Expression: both branches must return Int32
    x: Int32 = if cond { 1 } else { 2 }

    -- Statement: branches can differ, no return type needed
    if cond {
      print("yes")
    } else {
      return  -- Different action
    }

    -- Expression: match must be exhaustive
    result: String = match opt {
      Some(v) => v.to_string(),
      None => "none"
    }

    -- Statement: can use wildcard freely
    match opt {
      Some(v) => process(v),
      _ => {}  -- Ignore
    }
  }
}

-- ============================================================================
-- IMPLICIT RETURNS
-- ============================================================================

implicit_returns {
  exegesis {
    DOL supports implicit returns from functions:

    - If function body ends with expression (no semicolon), it's returned
    - Explicit `return` statement also works
    - Type must match declared return type

    This enables concise function definitions.
  }

  examples {
    -- Implicit return
    function double(x: Int32) -> Int32 {
      x * 2  -- Implicit return
    }

    -- Explicit return (equivalent)
    function double_explicit(x: Int32) -> Int32 {
      return x * 2
    }

    -- Mixed style
    function abs(x: Int32) -> Int32 {
      if x < 0 {
        return -x  -- Early exit
      }
      x  -- Implicit return for positive
    }

    -- Block expression as implicit return
    function complex(a: Int32, b: Int32) -> Int32 {
      temp = a + b
      temp * temp
    }
  }
}

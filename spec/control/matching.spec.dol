-- ============================================================================
-- DOL 2.0 PATTERN MATCHING SPECIFICATION
-- ============================================================================
-- Module: spec.control.matching
-- Version: 1.0.0
-- Purpose: Pattern matching with exhaustiveness checking
-- ============================================================================

module spec.control.matching @ 1.0.0

exegesis {
  Pattern matching enables exhaustive case analysis with compile-time
  exhaustiveness checking. DOL's match expression is both powerful
  and type-safe, supporting destructuring, guards, and wildcards.

  Lowering uses MLIR's scf.switch or nested scf.if operations.
}

-- ============================================================================
-- MATCH EXPRESSION
-- ============================================================================

construct match_expression {
  syntax {
    match <scrutinee> {
      <pattern> [if <guard>] => <expression>,
      <pattern> [if <guard>] => <expression>,
      ...
    }
  }

  exegesis {
    Evaluates scrutinee once, then tests against patterns in order.
    First matching pattern (with satisfied guard) determines result.

    EXHAUSTIVENESS: All possible values must be covered.
    The compiler verifies exhaustiveness at compile-time.
  }

  examples {
    function describe(opt: Optional<Int32>) -> String {
      match opt {
        Some(x) if x > 0 => "positive: " + x.to_string(),
        Some(x) if x < 0 => "negative: " + x.to_string(),
        Some(0) => "zero",
        None => "nothing"
      }
    }

    function fibonacci(n: Int32) -> Int32 {
      match n {
        0 => 0,
        1 => 1,
        n => fibonacci(n - 1) + fibonacci(n - 2)
      }
    }
  }
}

-- ============================================================================
-- PATTERN TYPES
-- ============================================================================

patterns {
  literal_pattern {
    syntax: 0, 42, "hello", true, false
    exegesis: Matches exact literal values
  }

  variable_pattern {
    syntax: x, name, value
    exegesis: Binds matched value to identifier for use in arm
  }

  wildcard_pattern {
    syntax: _
    exegesis: Matches anything, discards value
  }

  tuple_pattern {
    syntax: (a, b, c)
    exegesis: Destructures tuples, binds elements
  }

  constructor_pattern {
    syntax: Some(x), None, Ok(value), Err(e)
    exegesis: Matches enum variants, destructures payloads
  }

  array_pattern {
    syntax: [first, second, ...rest]
    exegesis: Destructures arrays with optional rest collection
  }

  struct_pattern {
    syntax: Point { x, y }, Person { name: n, age }
    exegesis: Destructures structs, can rename fields
  }

  or_pattern {
    syntax: 0 | 1 | 2
    exegesis: Matches any of several patterns
  }

  range_pattern {
    syntax: 1..10, 'a'..'z'
    exegesis: Matches values in range (inclusive start, exclusive end)
  }
}

-- ============================================================================
-- GUARDS
-- ============================================================================

construct guards {
  syntax: <pattern> if <condition> => <expr>

  exegesis {
    Guards add conditions to patterns. A pattern matches only if
    both the structural match succeeds AND the guard evaluates to true.

    Guards can reference variables bound by the pattern.
  }

  examples {
    match point {
      Point { x, y } if x == y => "on diagonal",
      Point { x, y } if x == 0 => "on y-axis",
      Point { x, y } if y == 0 => "on x-axis",
      _ => "elsewhere"
    }
  }
}

-- ============================================================================
-- EXHAUSTIVENESS
-- ============================================================================

exhaustiveness {
  exegesis {
    The compiler requires all possible values be covered.

    For enum types: all variants must be handled (or use wildcard)
    For numeric types: ranges + wildcard needed
    For Optional<T>: both Some(_) and None required
    For Result<T,E>: both Ok(_) and Err(_) required

    Non-exhaustive matches are compile-time errors.
  }

  examples {
    -- VALID: All variants covered
    match result {
      Ok(x) => x,
      Err(e) => 0
    }

    -- VALID: Wildcard covers remaining cases
    match number {
      0 => "zero",
      1 => "one",
      _ => "many"
    }

    -- ERROR: Missing None case
    -- match opt {
    --   Some(x) => x
    -- }
  }
}

-- ============================================================================
-- MLIR LOWERING
-- ============================================================================

mlir_lowering {
  simple_switch {
    dol_code: """
      match x {
        0 => "zero",
        1 => "one",
        _ => "other"
      }
    """

    mlir_ir: """
      %c0 = arith.constant 0 : i32
      %c1 = arith.constant 1 : i32

      %is_zero = arith.cmpi eq, %x, %c0 : i32
      %result = scf.if %is_zero -> !dol.string {
        %s = dol.string "zero"
        scf.yield %s : !dol.string
      } else {
        %is_one = arith.cmpi eq, %x, %c1 : i32
        %inner = scf.if %is_one -> !dol.string {
          %s = dol.string "one"
          scf.yield %s : !dol.string
        } else {
          %s = dol.string "other"
          scf.yield %s : !dol.string
        }
        scf.yield %inner : !dol.string
      }
    """
  }

  destructuring {
    note: """
      Destructuring patterns lower to field extraction operations
      followed by pattern matching on sub-components.
    """
  }
}

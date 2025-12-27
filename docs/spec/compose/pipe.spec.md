-- ============================================================================
-- DOL 2.0 PIPE AND COMPOSE OPERATORS SPECIFICATION
-- ============================================================================
-- Module: spec.compose.pipe
-- Version: 1.0.0
-- Purpose: Functional composition operators |> and >>
-- ============================================================================

module spec.compose.pipe @ 1.0.0

exegesis {
  DOL provides two primary composition operators:

  - |> (pipe): Left-to-right value transformation
  - >> (compose): Function composition into pipelines

  These enable point-free programming and readable data transformation
  chains without nested function calls.
}

-- ============================================================================
-- PIPE OPERATOR |>
-- ============================================================================

operator pipe {
  syntax: <value> |> <function>
  symbol: |>
  precedence: 1 (low)
  associativity: left

  type_signature {
    (a |> (a -> b)) -> b
    (a |> (a, ctx -> b), ctx) -> b  -- With context
  }

  exegesis {
    The pipe operator takes a value on the left and a function on the right,
    applying the function to the value. Equivalent to f(x) but reads left-to-right.

    Benefits:
    - Natural reading order (data flows left to right)
    - Avoids deeply nested parentheses
    - Each transformation step is clear
    - Easy to insert/remove/reorder steps
  }

  examples {
    -- Basic pipeline
    result = data
      |> parse
      |> validate
      |> transform
      |> serialize

    -- Equivalent nested calls (harder to read)
    result = serialize(transform(validate(parse(data))))

    -- With partial application
    result = items
      |> filter(_, is_valid)
      |> map(_, double)
      |> reduce(_, 0, add)

    -- Chained transformations
    user_name = get_user(id)
      |> get_profile(_)
      |> extract_name(_)
      |> capitalize(_)
      |> truncate(_, 50)
  }

  mlir_lowering {
    dol_code: """
      result = x |> f |> g
    """

    mlir_ir: """
      %f_result = func.call @f(%x) : (i32) -> i32
      %g_result = func.call @g(%f_result) : (i32) -> i32
    """

    note: """
      Pipe is purely syntactic sugar. Lowers directly to function calls.
      No runtime overhead - same as manual function application.
    """
  }
}

-- ============================================================================
-- COMPOSE OPERATOR >>
-- ============================================================================

operator compose {
  syntax: <function1> >> <function2>
  symbol: >>
  precedence: 2
  associativity: left

  type_signature {
    ((a -> b) >> (b -> c)) -> (a -> c)
  }

  exegesis {
    The compose operator creates a new function by chaining two functions.
    (f >> g) creates a function that applies f first, then g.

    Unlike pipe (which takes a value), compose takes two functions and
    returns a new function. This enables building reusable pipelines.
  }

  examples {
    -- Build reusable pipeline
    process = parse >> validate >> transform >> serialize

    -- Use the composed function
    result1 = process(data1)
    result2 = process(data2)

    -- Compose and use immediately
    extract_email = get_user >> get_profile >> get_email

    -- Higher-order usage
    processors = [normalize >> validate, compress >> encrypt]
  }

  mlir_lowering {
    dol_code: """
      composed = f >> g
    """

    mlir_ir: """
      // Creates a new function that calls f then g
      func.func @composed(%arg: i32) -> i32 {
        %f_result = func.call @f(%arg) : (i32) -> i32
        %g_result = func.call @g(%f_result) : (i32) -> i32
        return %g_result : i32
      }
    """
  }
}

-- ============================================================================
-- BACKWARD COMPOSE <|
-- ============================================================================

operator backward_pipe {
  syntax: <function> <| <value>
  symbol: <|
  precedence: 1
  associativity: right

  type_signature {
    ((a -> b) <| a) -> b
  }

  exegesis {
    Backward pipe applies function on left to value on right.
    Useful when function name is more important than argument.

    Less common than |> but useful in some patterns.
  }

  examples {
    -- Function emphasis
    result = serialize <| transform <| validate <| data

    -- Mixing with regular calls
    print <| format_message("Hello", user)
  }
}

-- ============================================================================
-- PLACEHOLDER SYNTAX
-- ============================================================================

placeholder {
  syntax: _ (underscore)

  exegesis {
    When a function takes multiple arguments, use _ to indicate
    where the piped value should be inserted.

    Without placeholder: value goes to first argument
    With placeholder: value goes where _ appears
  }

  examples {
    -- Default: first argument
    5 |> add(_, 3)  -- Same as add(5, 3)

    -- Explicit position
    items |> filter(_, is_valid)  -- filter(items, is_valid)

    -- Second argument
    3 |> add(5, _)  -- add(5, 3)

    -- Multiple uses create partial application
    multiplier = 2 |> multiply(_, _)  -- Creates (x) -> multiply(2, x)
  }
}

-- ============================================================================
-- TYPE INFERENCE
-- ============================================================================

type_inference {
  exegesis {
    Type inference flows through pipe chains:

    1. Infer type of left value
    2. Check right function accepts that type
    3. Result type becomes input for next stage

    Errors are reported at the failing step with context.
  }

  examples {
    -- Type flows through
    numbers: Array<Int32> = data  -- String
      |> parse             -- String -> Array<Int32>
      |> filter(_, is_even)  -- Array<Int32> -> Array<Int32>
      |> sum               -- Array<Int32> -> Int32

    -- Type error caught at step
    broken = "hello"
      |> parse_int  -- String -> Int32
      |> uppercase  -- ERROR: uppercase expects String, got Int32
  }
}

-- ============================================================================
-- INTEGRATION WITH RESULT/OPTIONAL
-- ============================================================================

monadic_pipes {
  exegesis {
    Pipe works naturally with Result and Optional types.
    Use special variants for short-circuiting behavior.
  }

  operators {
    |?>  -- Optional chain: short-circuit on None
    |!>  -- Result chain: short-circuit on Err
  }

  examples {
    -- Optional chaining
    name = get_user(id)
      |?> get_profile(_)  -- Skip if None
      |?> get_name(_)     -- Skip if None
      |?> capitalize(_)   -- Skip if None

    -- Result chaining
    data = read_file(path)
      |!> parse(_)      -- Skip if Err
      |!> validate(_)   -- Skip if Err
      |!> transform(_)  -- Skip if Err
  }
}

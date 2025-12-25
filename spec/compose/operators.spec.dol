-- ============================================================================
-- DOL 2.0 FUNCTIONAL OPERATORS SPECIFICATION
-- ============================================================================
-- Module: spec.compose.operators
-- Version: 1.0.0
-- Purpose: Apply @ and Bind := operators
-- ============================================================================

module spec.compose.operators @ 1.0.0

exegesis {
  DOL provides additional functional operators beyond pipe and compose:

  - @ (apply): Function application with precedence control
  - := (bind): Monadic bind for Result/Optional chaining

  These operators complete the functional programming toolkit.
}

-- ============================================================================
-- APPLY OPERATOR @
-- ============================================================================

operator apply {
  syntax: <function> @ <argument>
  symbol: @
  precedence: 3 (higher than pipe)
  associativity: right

  type_signature {
    (Fn(A) -> B) @ A -> B
    (Fn(A, B) -> C) @ (A, B) -> C
  }

  exegesis {
    Apply operator is low-precedence function application.
    Right-associative, so `f @ g @ x` means `f @ (g @ x)`.

    Useful when combining functions without parentheses.
  }

  examples {
    -- Basic application
    result = double @ 5  -- Same as double(5)

    -- Right associativity enables chaining
    result = print @ format @ value
    -- Parses as: print @ (format @ value)
    -- Same as: print(format(value))

    -- Compare with pipe
    result = value |> format |> print  -- Same result, left-to-right
    result = print @ format @ value     -- Same result, right-to-left

    -- Useful with partial application
    result = map(list) @ |x| x * 2
  }

  mlir_lowering {
    note: """
      @ is purely syntactic - lowers to regular function call.
      No runtime overhead.
    """
  }
}

-- ============================================================================
-- BIND OPERATOR :=
-- ============================================================================

operator bind {
  syntax: <monadic_value> := <function>
  symbol: :=
  precedence: 2
  associativity: left

  type_signature {
    -- For Optional
    (Optional<A> := Fn(A) -> Optional<B>) -> Optional<B>

    -- For Result
    (Result<A, E> := Fn(A) -> Result<B, E>) -> Result<B, E>
  }

  exegesis {
    Bind operator chains monadic operations.
    If left side is error/none, short-circuits without calling function.
    If left side has value, unwraps and applies function.

    This is the flatMap/chain operation from functional programming,
    but with infix syntax for readability.
  }

  examples {
    -- Optional chaining
    result: Optional<Int32> = get_user(id)
      := |user| get_profile(user)
      := |profile| get_age(profile)

    -- Equivalent to
    result = match get_user(id) {
      Some(user) => match get_profile(user) {
        Some(profile) => get_age(profile),
        None => None
      },
      None => None
    }

    -- Result chaining
    data: Result<Data, Error> = read_file(path)
      := |content| parse(content)
      := |parsed| validate(parsed)
      := |valid| transform(valid)

    -- Mixed with other operators
    result = get_user(id)
      := |u| u.profile
      |> format_profile
      := |formatted| save(formatted)
  }

  mlir_lowering {
    dol_code: """
      result = opt := |x| Some(x * 2)
    """

    mlir_ir: """
      // Check if Some
      %is_some = dol.optional.is_some %opt : !dol.optional<i32>

      %result = scf.if %is_some -> !dol.optional<i32> {
        // Unwrap and apply
        %val = dol.optional.unwrap %opt : i32
        %doubled = arith.muli %val, %c2 : i32
        %new_opt = dol.optional.some %doubled : !dol.optional<i32>
        scf.yield %new_opt : !dol.optional<i32>
      } else {
        // Propagate None
        %none = dol.optional.none : !dol.optional<i32>
        scf.yield %none : !dol.optional<i32>
      }
    """
  }
}

-- ============================================================================
-- OPERATOR TABLE
-- ============================================================================

operator_table {
  -- Composition operators in order of precedence

  | Symbol | Name      | Prec | Assoc | Type                    | Description              |
  |--------|-----------|------|-------|-------------------------|--------------------------|
  | >>     | compose   | 4    | left  | (a→b, b→c) → (a→c)     | Function composition     |
  | @      | apply     | 3    | right | (a→b, a) → b           | Function application     |
  | :=     | bind      | 2    | left  | (M a, a→M b) → M b     | Monadic bind            |
  | |>     | pipe      | 1    | left  | (a, a→b) → b           | Forward pipe            |
  | <|     | back-pipe | 1    | right | (a→b, a) → b           | Backward pipe           |

  exegesis {
    Precedence determines grouping:
      a |> f >> g @ x  parses as  a |> ((f >> g) @ x)

    Higher precedence binds tighter.
  }
}

-- ============================================================================
-- IDIOM BRACKETS
-- ============================================================================

idiom_brackets {
  syntax: [| <expression> |]

  exegesis {
    Idiom brackets lift regular function application to work with
    wrapped values (Optional, Result, etc).

    Inside brackets, function application is lifted:
    [| f x y |] = f <$> x <*> y (using functor/applicative)
  }

  examples {
    -- Lift function over Optionals
    result: Optional<Int32> = [| add opt_a opt_b |]

    -- Equivalent to
    result = match (opt_a, opt_b) {
      (Some(a), Some(b)) => Some(add(a, b)),
      _ => None
    }

    -- With multiple wrapped values
    full_name: Optional<String> = [| format first_opt middle_opt last_opt |]
  }

  note: """
    Idiom brackets are syntax sugar. They require the wrapped type
    to implement the Applicative trait (map + apply).
  """
}

-- ============================================================================
-- DO NOTATION
-- ============================================================================

do_notation {
  syntax {
    do {
      <pattern> <- <monadic_expr>
      <pattern> <- <monadic_expr>
      ...
      <final_expr>
    }
  }

  exegesis {
    Do notation provides imperative-looking syntax for monadic operations.
    Each `<-` binds the unwrapped value for use in subsequent lines.

    Desugars to nested := (bind) operations.
  }

  examples {
    -- Optional do-block
    result: Optional<String> = do {
      user <- get_user(id)
      profile <- get_profile(user)
      name <- get_name(profile)
      format_name(name)  -- Final expression wrapped in Some
    }

    -- Desugars to
    result = get_user(id) := |user|
      get_profile(user) := |profile|
        get_name(profile) := |name|
          Some(format_name(name))

    -- Result do-block with early return
    result: Result<Data, Error> = do {
      file <- read_file(path)
      parsed <- parse(file)
      validated <- validate(parsed)
      Ok(transform(validated))
    }
  }

  mlir_lowering {
    note: """
      Do notation is purely syntactic sugar.
      Compiler desugars to := chains before lowering.
    """
  }
}

-- ============================================================================
-- TYPE INFERENCE
-- ============================================================================

type_inference {
  exegesis {
    All functional operators support bidirectional type inference.

    The compiler infers:
    - Parameter types from usage context
    - Return types from function bodies
    - Monadic types from := chains

    Explicit type annotations rarely needed.
  }

  examples {
    -- Types inferred throughout
    result = get_data()        -- Result<Data, Error> inferred
      := |d| process(d)        -- Fn(Data) -> Result<Processed, Error>
      := |p| validate(p)       -- Fn(Processed) -> Result<Valid, Error>
      |> finalize              -- Fn(Result<Valid, Error>) -> Output

    -- Error: type mismatch caught
    broken = Some(5)
      := |x| x + 1  -- Error: expected Optional<T>, got Int32
  }
}

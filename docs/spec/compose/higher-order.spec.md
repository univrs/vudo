-- ============================================================================
-- DOL 2.0 HIGHER-ORDER FUNCTIONS SPECIFICATION
-- ============================================================================
-- Module: spec.compose.higher-order
-- Version: 1.0.0
-- Purpose: Higher-order function patterns and types
-- ============================================================================

module spec.compose.higher-order @ 1.0.0

exegesis {
  Higher-order functions are functions that take or return other functions.
  They enable powerful abstraction patterns like map, filter, reduce.

  DOL treats functions as first-class values, enabling full higher-order
  programming with type safety.
}

-- ============================================================================
-- FUNCTION AS PARAMETER
-- ============================================================================

functions_as_parameters {
  exegesis {
    Functions can be passed as arguments to other functions.
    The parameter type uses Fn(...) syntax.
  }

  examples {
    -- Apply function to value
    function apply<T, U>(value: T, f: Fn(T) -> U) -> U {
      f(value)
    }

    result = apply(5, |x| x * 2)  -- 10

    -- Apply twice
    function apply_twice<T>(value: T, f: Fn(T) -> T) -> T {
      f(f(value))
    }

    result = apply_twice(3, |x| x + 1)  -- 5

    -- Conditional application
    function apply_if<T>(value: T, condition: Bool, f: Fn(T) -> T) -> T {
      if condition { f(value) } else { value }
    }
  }
}

-- ============================================================================
-- FUNCTION AS RETURN VALUE
-- ============================================================================

functions_as_return {
  exegesis {
    Functions can return other functions, enabling currying,
    factory patterns, and function generators.
  }

  examples {
    -- Currying
    function add(a: Int32) -> Fn(Int32) -> Int32 {
      |b| a + b
    }

    add5 = add(5)
    result = add5(10)  -- 15

    -- Function factory
    function make_multiplier(factor: Int32) -> Fn(Int32) -> Int32 {
      |x| x * factor
    }

    double = make_multiplier(2)
    triple = make_multiplier(3)

    -- Predicate factory
    function greater_than(threshold: Int32) -> Fn(Int32) -> Bool {
      |x| x > threshold
    }

    is_adult = greater_than(17)
    is_senior = greater_than(64)
  }
}

-- ============================================================================
-- STANDARD HIGHER-ORDER FUNCTIONS
-- ============================================================================

standard_hofs {
  map {
    signature: Fn(Array<T>, Fn(T) -> U) -> Array<U>

    exegesis {
      Transforms each element using the provided function.
      Preserves structure, changes element types.
    }

    example {
      numbers = [1, 2, 3, 4, 5]
      doubled = map(numbers, |x| x * 2)  -- [2, 4, 6, 8, 10]
      strings = map(numbers, |x| x.to_string())  -- ["1", "2", ...]
    }
  }

  filter {
    signature: Fn(Array<T>, Fn(T) -> Bool) -> Array<T>

    exegesis {
      Keeps only elements that satisfy the predicate.
      May reduce size, preserves element types.
    }

    example {
      numbers = [1, 2, 3, 4, 5, 6]
      evens = filter(numbers, |x| x % 2 == 0)  -- [2, 4, 6]
      big = filter(numbers, |x| x > 3)  -- [4, 5, 6]
    }
  }

  reduce {
    signature: Fn(Array<T>, U, Fn(U, T) -> U) -> U

    exegesis {
      Accumulates elements into a single value using folder function.
      Also called fold/inject in other languages.
    }

    example {
      numbers = [1, 2, 3, 4, 5]
      sum = reduce(numbers, 0, |acc, x| acc + x)  -- 15
      product = reduce(numbers, 1, |acc, x| acc * x)  -- 120
      max = reduce(numbers, numbers[0], |a, b| if a > b { a } else { b })
    }
  }

  find {
    signature: Fn(Array<T>, Fn(T) -> Bool) -> Optional<T>

    exegesis {
      Returns first element satisfying predicate, or None.
      Short-circuits on first match.
    }

    example {
      numbers = [1, 2, 3, 4, 5]
      first_even = find(numbers, |x| x % 2 == 0)  -- Some(2)
      first_big = find(numbers, |x| x > 10)  -- None
    }
  }

  any {
    signature: Fn(Array<T>, Fn(T) -> Bool) -> Bool

    exegesis {
      Returns true if any element satisfies predicate.
      Short-circuits on first match.
    }

    example {
      has_negative = any(numbers, |x| x < 0)
    }
  }

  all {
    signature: Fn(Array<T>, Fn(T) -> Bool) -> Bool

    exegesis {
      Returns true if all elements satisfy predicate.
      Short-circuits on first failure.
    }

    example {
      all_positive = all(numbers, |x| x > 0)
    }
  }

  zip_with {
    signature: Fn(Array<T>, Array<U>, Fn(T, U) -> V) -> Array<V>

    exegesis {
      Combines two arrays element-wise using provided function.
      Result length is minimum of input lengths.
    }

    example {
      a = [1, 2, 3]
      b = [4, 5, 6]
      sums = zip_with(a, b, |x, y| x + y)  -- [5, 7, 9]
      products = zip_with(a, b, |x, y| x * y)  -- [4, 10, 18]
    }
  }

  flat_map {
    signature: Fn(Array<T>, Fn(T) -> Array<U>) -> Array<U>

    exegesis {
      Maps then flattens. Each element produces array of results.
      Also called bind/chain in monadic contexts.
    }

    example {
      words = ["hello", "world"]
      chars = flat_map(words, |w| w.chars())  -- ['h','e','l','l','o','w',...]
    }
  }
}

-- ============================================================================
-- CURRYING AND PARTIAL APPLICATION
-- ============================================================================

currying {
  exegesis {
    Currying transforms multi-argument function into chain of single-argument functions.
    Partial application fixes some arguments, returning function for rest.

    DOL supports both patterns through lambdas and placeholder syntax.
  }

  examples {
    -- Manual currying
    function add_curried(a: Int32) -> Fn(Int32) -> Int32 {
      |b| a + b
    }

    -- Partial application with placeholder
    add_five = add(5, _)  -- Partial: second arg unfilled
    result = add_five(3)  -- 8

    -- Multiple placeholders
    between = is_between(_, 0, 100)  -- First arg unfilled
    valid = between(50)  -- true

    -- Composition with partial
    process = data
      |> parse
      |> transform(_, config)  -- Partial application in pipe
      |> serialize
  }
}

-- ============================================================================
-- FUNCTION COMPOSITION PATTERNS
-- ============================================================================

composition_patterns {
  point_free {
    exegesis {
      Point-free style defines functions without naming arguments.
      Uses composition to build complex transformations.
    }

    example {
      -- Point-free definition
      process = parse >> validate >> transform >> serialize

      -- Equivalent with arguments
      process_verbose = |data| serialize(transform(validate(parse(data))))
    }
  }

  combinator_library {
    exegesis {
      Small composable functions that combine to form complex behavior.
    }

    example {
      -- Combinators
      identity = |x| x
      constant = |x| |_| x
      flip = |f| |a, b| f(b, a)
      compose = |f, g| |x| g(f(x))

      -- Usage
      always_five = constant(5)
      result = always_five("ignored")  -- 5

      reverse_div = flip(divide)
      result = reverse_div(2, 10)  -- 5 (10/2)
    }
  }
}

-- ============================================================================
-- TYPE CONSTRAINTS
-- ============================================================================

type_constraints {
  exegesis {
    Higher-order function types must be explicit or inferrable.

    Generic HOFs use type parameters:
      function map<T, U>(arr: Array<T>, f: Fn(T) -> U) -> Array<U>

    Type inference flows through:
      map([1,2,3], |x| x * 2)  -- T=Int32, U=Int32 inferred
  }

  variance {
    exegesis {
      Function types are contravariant in parameters, covariant in return.

      Fn(Animal) -> Cat  is subtype of  Fn(Cat) -> Animal

      This enables safe substitution of function values.
    }
  }
}

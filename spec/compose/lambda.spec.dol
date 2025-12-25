-- ============================================================================
-- DOL 2.0 LAMBDA EXPRESSIONS SPECIFICATION
-- ============================================================================
-- Module: spec.compose.lambda
-- Version: 1.0.0
-- Purpose: Anonymous functions and closures
-- ============================================================================

module spec.compose.lambda @ 1.0.0

exegesis {
  Lambda expressions create anonymous functions with optional closures.
  DOL supports multiple lambda syntaxes for different use cases.

  Lambdas are first-class values: they can be stored, passed, and returned.
  Closures capture variables from their defining scope.
}

-- ============================================================================
-- LAMBDA SYNTAX
-- ============================================================================

construct lambda {
  syntax {
    -- Full form with types
    |<param>: <Type>, <param>: <Type>| -> <ReturnType> { <body> }

    -- Type-inferred parameters
    |<param>, <param>| { <body> }

    -- Single expression (implicit return)
    |<param>| <expression>

    -- No parameters
    || { <body> }
  }

  examples {
    -- Full type annotations
    add: Fn(Int32, Int32) -> Int32 = |a: Int32, b: Int32| -> Int32 { a + b }

    -- Inferred types
    double = |x| x * 2

    -- Single expression
    square = |x| x * x

    -- Multiple statements
    process = |x| {
      validated = validate(x)
      transform(validated)
    }

    -- No parameters
    get_timestamp = || now()

    -- Multiline
    complex = |a, b| {
      temp = a + b
      squared = temp * temp
      squared / 2
    }
  }
}

-- ============================================================================
-- CLOSURES
-- ============================================================================

closures {
  exegesis {
    Lambdas can capture variables from their enclosing scope.
    Captured variables form a "closure" - the function closes over its environment.

    Capture semantics:
    - By default, variables are captured by reference (shared)
    - Use `move` for ownership transfer (unique capture)
    - Captured variables extend their lifetime until lambda is dropped
  }

  examples {
    -- Capture by reference
    multiplier = 3
    triple = |x| x * multiplier  -- Captures multiplier

    result = triple(10)  -- 30
    multiplier = 4
    result2 = triple(10)  -- 40 (uses updated multiplier)

    -- Move capture
    data = expensive_create()
    process = move |x| {
      consume(data)  -- data moved into closure
      x
    }
    -- data no longer accessible here

    -- Multiple captures
    base = 10
    offset = 5
    compute = |x| base + x + offset

    -- Mutable capture
    counter = 0
    increment = || {
      counter = counter + 1
      counter
    }
    a = increment()  -- 1
    b = increment()  -- 2
  }

  memory_model {
    exegesis {
      Closure memory:
      - Captured references are stored in closure struct
      - Closure struct allocated based on capture size
      - Small closures (few captures) may be stack-allocated
      - Large closures use heap allocation

      Ownership:
      - Reference captures: shared reference to original
      - Move captures: ownership transferred to closure
      - Mutable captures: exclusive mutable reference
    }
  }
}

-- ============================================================================
-- FUNCTION TYPES
-- ============================================================================

function_types {
  syntax {
    Fn(<ParamTypes>) -> <ReturnType>
    Fn() -> <ReturnType>
    Fn(<ParamTypes>)  -- Returns Void
  }

  exegesis {
    Function types describe the signature of callable values.
    Both named functions and lambdas conform to function types.
  }

  examples {
    -- Function type annotations
    mapper: Fn(Int32) -> Int32 = |x| x * 2
    predicate: Fn(String) -> Bool = |s| s.length() > 0
    action: Fn() -> Void = || print("hello")

    -- Higher-order functions
    apply: Fn(Int32, Fn(Int32) -> Int32) -> Int32 =
      |x, f| f(x)

    -- Function returning function
    make_adder: Fn(Int32) -> Fn(Int32) -> Int32 =
      |n| |x| x + n

    add5 = make_adder(5)
    result = add5(10)  -- 15
  }
}

-- ============================================================================
-- INLINE LAMBDAS
-- ============================================================================

inline_lambdas {
  exegesis {
    Short lambdas can be used inline with higher-order functions.
    This is the most common use of lambdas in DOL.
  }

  examples {
    -- Filter with inline predicate
    evens = numbers.filter(|x| x % 2 == 0)

    -- Map with inline transform
    doubled = numbers.map(|x| x * 2)

    -- Sort with inline comparator
    sorted = items.sort_by(|a, b| a.name.compare(b.name))

    -- Reduce with inline folder
    sum = numbers.reduce(0, |acc, x| acc + x)

    -- Complex inline
    result = data
      .filter(|item| item.active)
      .map(|item| item.value)
      .reduce(0, |sum, val| sum + val)
  }
}

-- ============================================================================
-- MLIR LOWERING
-- ============================================================================

mlir_lowering {
  non_capturing {
    dol_code: """
      f = |x: Int32| x * 2
      result = f(5)
    """

    mlir_ir: """
      // Non-capturing lambda is just a function pointer
      func.func @lambda_0(%x: i32) -> i32 {
        %c2 = arith.constant 2 : i32
        %result = arith.muli %x, %c2 : i32
        return %result : i32
      }

      %f = llvm.mlir.addressof @lambda_0 : !llvm.ptr<func<i32(i32)>>
      %result = llvm.call @lambda_0(%c5) : (i32) -> i32
    """
  }

  capturing {
    dol_code: """
      multiplier = 3
      f = |x| x * multiplier
      result = f(5)
    """

    mlir_ir: """
      // Closure struct holds captured values
      !closure_type = !llvm.struct<(i32)>  // multiplier

      func.func @lambda_1(%closure: !llvm.ptr<struct<(i32)>>, %x: i32) -> i32 {
        %mult_ptr = llvm.getelementptr %closure[0, 0] : ...
        %mult = llvm.load %mult_ptr : !llvm.ptr<i32>
        %result = arith.muli %x, %mult : i32
        return %result : i32
      }

      // Create closure
      %closure = llvm.alloca : !llvm.ptr<!closure_type>
      llvm.store %multiplier, %closure[0, 0] : ...

      // Call with closure context
      %result = llvm.call @lambda_1(%closure, %c5) : ...
    """
  }
}

-- ============================================================================
-- CONSTRAINTS
-- ============================================================================

constraints {
  type_matching {
    exegesis {
      Lambda parameter and return types must match the expected function type.
      Type inference works bidirectionally - expected type informs lambda types.
    }
  }

  ownership {
    exegesis {
      Captured variables follow DOL's ownership rules:
      - Cannot capture moved values
      - Mutable captures require exclusive access
      - Reference captures extend lifetime
    }
  }

  recursion {
    exegesis {
      Lambdas cannot directly recurse (no name to call).
      Use named functions or Y-combinator patterns for recursion.
    }

    example {
      -- This works: named function
      function factorial(n: Int32) -> Int32 {
        if n <= 1 { 1 } else { n * factorial(n - 1) }
      }

      -- For lambda recursion, use fix-point
      factorial = fix(|f, n| if n <= 1 { 1 } else { n * f(n - 1) })
    }
  }
}

-- ============================================================================
-- DOL 2.0 META-PROGRAMMING: EVAL OPERATOR SPECIFICATION
-- ============================================================================
-- Module: spec.meta.eval
-- Version: 1.0.0
-- Purpose: Complete specification of the eval operator (!) for AST execution
-- ============================================================================

module spec.meta.eval @ 1.0.0

exegesis {
  The eval operator ! executes an Abstract Syntax Tree (AST) in a given
  context, converting code-as-data back into running code.

  Eval enables:
  - Dynamic code execution from quoted AST
  - Macro expansion
  - Template instantiation
  - Context-sensitive code generation

  Eval maintains type safety by requiring that evaluated AST has known
  type information, preventing runtime type errors in generated code.
}

-- ============================================================================
-- EVAL OPERATOR SYNTAX
-- ============================================================================

operator eval {
  syntax: !<ast_expression>
  syntax: !<ast_expression> where { <bindings> }
  syntax: !<ast_expression> in <scope_reference>

  precedence: PREFIX
  associativity: N/A

  exegesis {
    Eval operator has three forms:

    1. Simple eval: !ast
       Evaluates AST in current lexical scope

    2. Eval with bindings: !ast where { x = 10, y = 20 }
       Evaluates AST with explicit variable bindings

    3. Eval in scope: !ast in other_scope
       Evaluates AST in a different scope reference
  }

  type_signature: """
    eval<T>(ast: Ast) -> T where ast.type_info == TypeInfo.of<T>()
    eval<T>(ast: Ast, bindings: Map<String, Any>) -> T
    eval<T>(ast: Ast, scope: ScopeRef) -> T
  """
}

-- ============================================================================
-- HYGIENE RULES
-- ============================================================================

hygiene eval {
  rule prevent_capture {
    exegesis {
      Variables introduced during eval must not accidentally capture
      variables in the evaluation context.

      Implementation:
      - AST nodes carry hygiene marks from their quote site
      - During eval, identifiers are resolved using hygiene marks
      - Only variables from compatible hygiene contexts can be accessed
    }

    example {
      x: Int32 = 100

      ast: Ast = '{
        x: Int32 = 200
        x + 10
      }

      result: Int32 = !ast  -- 210, not 110
      print(x)  -- Still 100
    }
  }

  rule binding_precedence {
    exegesis {
      When evaluating with explicit bindings, the precedence is:

      1. Explicit where clause bindings (highest priority)
      2. Bindings from quote site captured in AST
      3. Current evaluation context bindings (lowest priority)
    }

    example {
      x: Int32 = 10
      ast: Ast = '{ result = x + y }
      y: Int32 = 20

      value1: Int32 = !ast where { x = 100, y = 200 }  -- 300
      value2: Int32 = !ast where { x = 50 }  -- 70
      value3: Int32 = !ast  -- 30
    }
  }

  rule scope_isolation {
    exegesis {
      Variables bound during eval do not leak into the evaluation context.
    }

    example {
      ast: Ast = '{
        temp: Int32 = 42
        inner_result = temp * 2
        inner_result
      }

      result: Int32 = !ast  -- 84
      -- temp and inner_result don't exist here
    }
  }
}

-- ============================================================================
-- TYPE SAFETY
-- ============================================================================

type_safety eval {
  rule static_type_checking {
    exegesis {
      Eval is type-safe: the type of the evaluated result must be
      known at compile-time and match the expected type.
    }

    example {
      ast: Ast = '{ 10 + 20 }
      result: Int32 = !ast  -- OK: types match
      -- wrong: String = !ast  -- Error: expected String, got Int32
    }
  }

  rule binding_type_checking {
    exegesis {
      When using where clause bindings, provided values must match
      the types expected by the AST.
    }

    example {
      ast: Ast = '{ x + y }

      result: Int32 = !ast where { x = 10, y = 20 }  -- OK
      -- bad: Int32 = !ast where { x = "hello", y = 20 }  -- Error
    }
  }

  rule polymorphic_eval {
    exegesis {
      Eval can instantiate polymorphic AST with concrete types.
    }

    example {
      generic_max: Ast = '{
        function max<T>(a: T, b: T) -> T where T: Comparable {
          if a > b { return a } else { return b }
        }
      }

      !generic_max  -- Defines max function

      result_int: Int32 = max(10, 20)
      result_float: Float64 = max(1.5, 2.5)
    }
  }
}

-- ============================================================================
-- SCOPING SEMANTICS
-- ============================================================================

scoping eval {
  rule lexical_resolution {
    exegesis {
      Free variables in evaluated AST are resolved using:
      1. Hygiene marks to identify original binding context
      2. Current lexical scope for unmarked variables
      3. Explicit where clause bindings as overrides
    }

    example {
      outer: Int32 = 100

      function create_and_eval() -> Int32 {
        inner: Int32 = 200
        ast: Ast = '{ outer + inner }
        return !ast  -- 300
      }
    }
  }

  rule scope_reference {
    exegesis {
      Advanced: eval can execute in a captured scope reference.
    }

    example {
      captured_scope: ScopeRef = capture_scope {
        x: Int32 = 100
        y: Int32 = 200
      }

      ast: Ast = '{ x + y }
      result: Int32 = !ast in captured_scope  -- 300
    }
  }
}

-- ============================================================================
-- EVALUATION CONTEXT
-- ============================================================================

evaluation_context {
  struct EvalContext {
    bindings: Map<String, Value>,
    parent_scope: Optional<ScopeRef>,
    hygiene_resolver: HygieneResolver,
    type_environment: TypeEnvironment
  }

  exegesis {
    EvalContext represents the environment in which AST is evaluated.

    Components:
    - bindings: Variable name to value mapping
    - parent_scope: Lexical parent for scope chain
    - hygiene_resolver: Resolves hygienic identifiers
    - type_environment: Type information for checking
  }
}

-- ============================================================================
-- MLIR LOWERING
-- ============================================================================

mlir_lowering eval {
  compile_time_eval {
    exegesis {
      Most eval operations happen at compile-time and are completely
      eliminated from the final code. The evaluated result is inlined.
    }

    example {
      dol_code: """
        ast: Ast = '{ 10 + 20 }
        result: Int32 = !ast
      """

      mlir_ir: """
        // Quote-eval pair eliminated, result inlined
        %result = arith.constant 30 : i32
      """
    }
  }

  staged_evaluation {
    exegesis {
      When AST values flow through the program, eval becomes a
      runtime operation using the meta dialect.
    }

    example {
      dol_code: """
        function eval_template(template: Ast, x: Int32) -> Int32 {
          return !template where { value = x }
        }
      """

      mlir_ir: """
        func.func @eval_template(%template: !meta.ast<i32>, %x: i32) -> i32 {
          %ctx = meta.create_context %template : !meta.context
          %x_name = meta.constant "value" : !meta.identifier
          %ctx2 = meta.bind %ctx, %x_name, %x : !meta.context
          %result = meta.eval %template, %ctx2 : i32
          return %result : i32
        }
      """
    }
  }

  optimized_expansion {
    exegesis {
      For known AST patterns (like macro invocations), the compiler
      performs direct code expansion without runtime overhead.
    }

    example {
      dol_code: """
        macro double(x: Ast) -> Ast {
          return '{ $x * 2 }
        }

        result: Int32 = !double('{ 5 + 3 })
      """

      mlir_ir: """
        // Fully expanded and optimized at compile-time
        %c5 = arith.constant 5 : i32
        %c3 = arith.constant 3 : i32
        %c2 = arith.constant 2 : i32
        %sum = arith.addi %c5, %c3 : i32
        %result = arith.muli %sum, %c2 : i32
        // result = 16
      """
    }
  }
}

-- ============================================================================
-- EXAMPLES
-- ============================================================================

exegesis examples {
  example basic_eval {
    description: "Simple AST evaluation"

    code: """
      x: Int32 = 10
      y: Int32 = 20
      ast: Ast = '{ x + y * 2 }
      result: Int32 = !ast  -- 50
    """
  }

  example eval_with_bindings {
    description: "Eval with explicit variable bindings"

    code: """
      template: Ast = '{ base + offset * multiplier }

      result1: Int32 = !template where {
        base = 100,
        offset = 10,
        multiplier = 2
      }  -- 120

      result2: Int32 = !template where {
        base = 0,
        offset = 5,
        multiplier = 10
      }  -- 50
    """
  }

  example macro_expansion {
    description: "Using eval for macro expansion"

    code: """
      macro unless(condition: Ast, body: Ast) -> Ast {
        return '{ if not $condition { $body } }
      }

      list: Vector<Int32> = [1, 2, 3]

      !unless('{ list.is_empty() }, '{
        print("List has elements")
        process(list)
      })
    """
  }

  example staged_computation {
    description: "Multi-stage programming with eval"

    code: """
      function generate_power(n: Int32) -> Ast {
        if n == 0 {
          return '{ 1 }
        } else if n == 1 {
          return '{ x }
        } else {
          prev: Ast = generate_power(n - 1)
          return '{ $prev * x }
        }
      }

      power_3: Ast = generate_power(3)

      function cube(x: Int32) -> Int32 {
        return !power_3
      }

      result: Int32 = cube(5)  -- 125
    """
  }
}

-- ============================================================================
-- CONSTRAINTS
-- ============================================================================

constraint eval_validity {
  rule type_compatibility {
    exegesis {
      The result type of eval must be compatible with the expected type.
    }
  }

  rule binding_completeness {
    exegesis {
      All free variables in the AST must be bound.
    }
  }

  rule scope_validity {
    exegesis {
      Scope references must be valid and compatible with AST requirements.
    }
  }
}

-- ============================================================================
-- RELATED CONSTRUCTS
-- ============================================================================

related_constructs {
  quote: "The ' operator creates AST that eval executes"
  macro: "The # operator combines quote/eval for code generation"
  reflect: "The ? operator provides type information for eval"

  exegesis {
    Eval is the execution complement to quote:

    - Quote captures code as AST
    - Eval executes AST as code
    - Together they form a code-as-data cycle
  }
}

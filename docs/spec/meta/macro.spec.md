-- ============================================================================
-- DOL 2.0 META-PROGRAMMING: MACRO OPERATOR SPECIFICATION
-- ============================================================================
-- Module: spec.meta.macro
-- Version: 1.0.0
-- Purpose: Complete specification of the macro operator (#) for code generation
-- ============================================================================

module spec.meta.macro @ 1.0.0

exegesis {
  The macro operator # enables compile-time code generation by combining
  quote (') and eval (!) with pattern matching and transformation.

  Macros enable:
  - Syntactic abstraction beyond functions
  - Domain-specific language embedding
  - Compile-time computation
  - Code generation patterns
  - Zero-cost abstractions

  DOL macros are hygienic, type-safe, and debuggable.
}

-- ============================================================================
-- MACRO DEFINITION SYNTAX
-- ============================================================================

construct macro_definition {
  syntax {
    macro <name>(<params>) -> Ast {
      <body>
    }

    macro <name>(<params>) -> Ast where <constraint> {
      <body>
    }
  }

  exegesis {
    Macro definitions:
    - Take AST fragments as parameters
    - Return AST as result
    - Execute at compile-time
    - Can have type constraints on parameters

    The body uses quote (') to construct result AST
    and can include compile-time computation.
  }

  examples {
    -- Simple macro
    macro double(x: Ast) -> Ast {
      return '{ $x * 2 }
    }

    -- Multi-parameter macro
    macro swap(a: Ast, b: Ast) -> Ast {
      return '{
        temp = $a
        $a = $b
        $b = temp
      }
    }

    -- Constrained macro
    macro debug_print(expr: Ast) -> Ast where expr: Stringable {
      name: String = stringify(expr)
      return '{
        print($name + " = " + ($expr).to_string())
      }
    }
  }
}

-- ============================================================================
-- MACRO INVOCATION
-- ============================================================================

construct macro_invocation {
  syntax {
    !<macro_name>(<arguments>)
    #<macro_name>(<arguments>)
  }

  exegesis {
    Two invocation forms:
    - !macro(...) - Expand and evaluate immediately
    - #macro(...) - Return unexpanded AST for further manipulation

    Arguments are passed as AST fragments, not evaluated values.
  }

  examples {
    result: Int32 = !double('{ 5 + 3 })  -- Expands to (5 + 3) * 2 = 16

    x: Int32 = 1
    y: Int32 = 2
    !swap('x, 'y)  -- Swaps x and y

    !debug_print('{ x + y })  -- Prints "x + y = 3"
  }
}

-- ============================================================================
-- PATTERN MATCHING MACROS
-- ============================================================================

pattern_matching_macros {
  syntax {
    macro <name> {
      (<pattern1>) => <result1>,
      (<pattern2>) => <result2>,
      ...
    }
  }

  exegesis {
    Multi-armed macros match on AST patterns to select transformation.
    Enables flexible DSL construction.
  }

  examples {
    macro match_op {
      (add $a $b) => '{ $a + $b },
      (sub $a $b) => '{ $a - $b },
      (mul $a $b) => '{ $a * $b },
      (div $a $b) => '{ $a / $b }
    }

    result: Int32 = !match_op(add '5 '3)  -- 8

    macro log {
      (debug $msg) => '{ if DEBUG { print("[DEBUG] " + $msg) } },
      (info $msg)  => '{ print("[INFO] " + $msg) },
      (error $msg) => '{ print("[ERROR] " + $msg); panic($msg) }
    }

    !log(info '"Starting process")
    !log(error '"Fatal error occurred")
  }
}

-- ============================================================================
-- HYGIENE
-- ============================================================================

hygiene macro {
  rule automatic_hygiene {
    exegesis {
      DOL macros are hygienic by default:
      - Variables introduced by macro don't capture user variables
      - User variables don't capture macro internals
      - Each expansion gets unique hygiene marks
    }

    example {
      macro with_temp(body: Ast) -> Ast {
        return '{
          temp: Int32 = 0  -- Macro's temp
          $body
        }
      }

      temp: Int32 = 42  -- User's temp
      !with_temp('{ print(temp.to_string()) })  -- Prints "42", not "0"
    }
  }

  rule intentional_capture {
    exegesis {
      Use escape hatch for intentional capture when needed.
    }

    example {
      macro define_counter() -> Ast {
        return '{
          #unhygienic(counter): Int32 = 0
          function increment() -> Int32 {
            counter = counter + 1
            return counter
          }
        }
      }

      !define_counter()
      a: Int32 = increment()  -- 1
      b: Int32 = increment()  -- 2
    }
  }
}

-- ============================================================================
-- COMPILE-TIME COMPUTATION
-- ============================================================================

compile_time {
  exegesis {
    Macros can perform arbitrary computation at compile-time.
    Results are embedded in generated code.
  }

  examples {
    -- Compile-time factorial
    macro factorial(n: Int32) -> Ast {
      if n <= 1 {
        return '{ 1 }
      } else {
        prev: Ast = factorial(n - 1)
        return '{ $n * $prev }
      }
    }

    result: Int32 = !factorial(5)  -- Expands to 5 * 4 * 3 * 2 * 1 = 120 (computed at compile-time)

    -- Compile-time string processing
    macro make_getter(field_name: String) -> Ast {
      getter_name: String = "get_" + field_name
      return '{
        function #ident($getter_name)() -> Int32 {
          return self.#ident($field_name)
        }
      }
    }

    !make_getter("age")  -- Generates get_age() function

    -- Loop unrolling
    macro unroll(count: Int32, body: Ast) -> Ast {
      statements: Vector<Ast> = []
      for i in 0..count {
        statements.push(substitute(body, 'i, 'i))
      }
      return '{ #splice(statements) }
    }

    !unroll(4, '{ process(i) })
    -- Expands to: process(0); process(1); process(2); process(3)
  }
}

-- ============================================================================
-- RECURSIVE MACROS
-- ============================================================================

recursive_macros {
  exegesis {
    Macros can be recursive with termination guarantees.
    The compiler enforces a maximum expansion depth.
  }

  examples {
    macro repeat(n: Int32, body: Ast) -> Ast {
      if n <= 0 {
        return '{ }  -- Empty
      } else {
        rest: Ast = repeat(n - 1, body)
        return '{
          $body
          $rest
        }
      }
    }

    !repeat(3, '{ print("Hello") })
    -- Expands to 3 print statements

    macro build_chain(ops: Array<Ast>) -> Ast {
      if ops.is_empty() {
        return '{ x }  -- Identity
      } else {
        first: Ast = ops[0]
        rest: Ast = build_chain(ops[1..])
        return '{ $first($rest) }
      }
    }

    pipeline: Ast = !build_chain(['parse, 'validate, 'transform])
    -- Generates: parse(validate(transform(x)))
  }

  constraint termination {
    max_depth: 256

    exegesis {
      The compiler limits macro expansion depth to 256 by default.
      This prevents infinite recursion during compilation.
    }
  }
}

-- ============================================================================
-- MLIR LOWERING
-- ============================================================================

mlir_lowering macro {
  expansion_phase {
    exegesis {
      Macro expansion occurs during compilation before MLIR lowering.
      All macros are fully expanded to regular DOL code.
    }

    example {
      dol_code: """
        macro double(x: Ast) -> Ast {
          return '{ $x * 2 }
        }

        result: Int32 = !double('{ 5 + 3 })
      """

      after_expansion: """
        result: Int32 = (5 + 3) * 2
      """

      mlir_ir: """
        %c5 = arith.constant 5 : i32
        %c3 = arith.constant 3 : i32
        %c2 = arith.constant 2 : i32
        %sum = arith.addi %c5, %c3 : i32
        %result = arith.muli %sum, %c2 : i32
      """
    }
  }

  debug_info {
    exegesis {
      Macro expansions preserve debug information:
      - Original macro name and arguments
      - Expansion location
      - Hygiene context

      This enables debugging through macro expansions.
    }
  }
}

-- ============================================================================
-- EXAMPLES
-- ============================================================================

exegesis examples {
  example control_flow_macro {
    description: "Custom control flow constructs"

    code: """
      macro unless(condition: Ast, body: Ast) -> Ast {
        return '{ if not $condition { $body } }
      }

      macro when(condition: Ast, body: Ast) -> Ast {
        return '{ if $condition { $body } }
      }

      macro times(count: Ast, body: Ast) -> Ast {
        return '{
          for _i in 0..$count {
            $body
          }
        }
      }

      !unless('{ list.is_empty() }, '{ process(list) })
      !times('5, '{ print("Hello") })
    """
  }

  example dsl_embedding {
    description: "Domain-specific language via macros"

    code: """
      macro sql {
        (SELECT $fields FROM $table WHERE $condition) => '{
          query_builder()
            .select($fields)
            .from($table)
            .where($condition)
            .execute()
        }
      }

      results = !sql(SELECT '[name, age] FROM '"users" WHERE '{ age > 18 })
    """
  }

  example derive_macro {
    description: "Auto-implementing traits"

    code: """
      macro derive_debug(type_def: Ast) -> Ast {
        type_name: String = extract_name(type_def)
        fields: Array<Ast> = extract_fields(type_def)

        field_prints: Vector<Ast> = []
        for field in fields {
          field_name: String = stringify(field)
          field_prints.push('{
            print($field_name + ": " + self.$field.to_string())
          })
        }

        return '{
          $type_def

          impl Debug for #ident($type_name) {
            function debug_print(self) -> Void {
              print($type_name + " {")
              #splice(field_prints)
              print("}")
            }
          }
        }
      }

      !derive_debug('{
        struct Person {
          name: String,
          age: Int32
        }
      })
    """
  }
}

-- ============================================================================
-- CONSTRAINTS
-- ============================================================================

constraint macro_validity {
  rule type_safety {
    exegesis {
      Macro results must be type-safe DOL code.
      Type errors in expanded code are reported at expansion site.
    }
  }

  rule hygiene {
    exegesis {
      Macros must be hygienic by default.
      Intentional capture requires explicit escape hatch.
    }
  }

  rule termination {
    exegesis {
      Recursive macros must terminate within expansion limit.
    }
  }

  rule determinism {
    exegesis {
      Macro expansion must be deterministic.
      Same inputs always produce same outputs.
    }
  }
}

-- ============================================================================
-- RELATED CONSTRUCTS
-- ============================================================================

related_constructs {
  quote: "The ' operator creates AST that macros transform"
  eval: "The ! operator executes macro results"
  reflect: "The ? operator provides type info for macros"

  exegesis {
    Macros combine quote, eval, and compile-time computation:

    - Quote captures code patterns
    - Compile-time logic transforms patterns
    - Eval expands result into final code

    This enables powerful, zero-cost abstractions.
  }
}

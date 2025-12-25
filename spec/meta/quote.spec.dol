-- ============================================================================
-- DOL 2.0 META-PROGRAMMING: QUOTE OPERATOR SPECIFICATION
-- ============================================================================
-- Module: spec.meta.quote
-- Version: 1.0.0
-- Purpose: Complete specification of the quote operator (') for AST capture
-- ============================================================================

module spec.meta.quote @ 1.0.0

exegesis {
  The quote operator ' captures DOL code as an Abstract Syntax Tree (AST)
  without evaluating it. This enables:

  - Code-as-data manipulation
  - Compile-time metaprogramming
  - Template construction
  - DSL embedding
  - Macro definition

  Quote preserves full type information and lexical context, enabling
  hygienic macro expansion and type-safe code generation.
}

-- ============================================================================
-- AST TYPE DEFINITION
-- ============================================================================

gene Ast {
  type: {
    node_type: AstNodeType,
    type_info: TypeInfo,
    source_location: SourceLocation,
    children: Vector<Ast>,
    metadata: Map<String, Any>
  }

  exegesis {
    Ast represents a quoted code fragment with full semantic information.

    Properties:
    - node_type: The syntactic category (expression, statement, declaration)
    - type_info: Full type information from type checker
    - source_location: Original source file, line, column for debugging
    - children: Child AST nodes in the tree structure
    - metadata: Additional compiler information (hygiene marks, scope refs)
  }
}

enum AstNodeType {
  -- Expressions
  Literal,
  Identifier,
  BinaryOp,
  UnaryOp,
  FunctionCall,
  FieldAccess,
  IndexAccess,
  Lambda,
  Block,

  -- Statements
  Assignment,
  Declaration,
  IfStatement,
  WhileLoop,
  ForLoop,
  Return,
  Break,
  Continue,

  -- Declarations
  FunctionDecl,
  TypeDecl,
  GeneDecl,
  TraitDecl,

  -- Meta
  QuoteHole,
  SplicePoint
}

-- ============================================================================
-- QUOTE OPERATOR SYNTAX
-- ============================================================================

operator quote {
  syntax: '{ <expression> }
  syntax: '{ <statement>* }
  syntax: '{ $<identifier> }

  precedence: PREFIX
  associativity: N/A

  exegesis {
    Quote operator has three forms:

    1. Expression quote: '{ x + y }
       Captures a single expression as AST

    2. Statement block quote: '{ stmt1; stmt2; ... }
       Captures multiple statements as AST

    3. Quasi-quote with holes: '{ result = $expr + 1 }
       Creates a template with placeholders ($expr) that can be filled
       later during macro expansion
  }
}

-- ============================================================================
-- HYGIENE RULES
-- ============================================================================

hygiene quote {
  rule prevent_capture {
    exegesis {
      Variables introduced by quoted code must not capture variables
      in the expansion context unless explicitly intended.

      Implementation:
      - Each quoted identifier is tagged with its lexical scope ID
      - During expansion, scope tags are checked to prevent accidental capture
      - Only identifiers from the same or parent scopes are resolved
    }

    example {
      macro make_setter(field: Ast) -> Ast {
        return '{
          function set_field(value: Int32) -> Void {
            self.$field = value
          }
        }
      }

      value: Int32 = 100
      setter: Ast = make_setter('age)
      -- Expanded 'value' parameter doesn't capture outer 'value'
    }
  }

  rule preserve_bindings {
    exegesis {
      Free variables in quoted code retain their bindings from the
      quote site, not the expansion site.
    }

    example {
      constant: Int32 = 42

      macro use_constant() -> Ast {
        return '{ result = constant + 10 }
      }

      function other_scope() -> Int32 {
        constant: Int32 = 999
        !use_constant()  -- Still uses 42, not 999
        return result  -- 52
      }
    }
  }

  rule hygiene_marks {
    exegesis {
      Each quoted AST node is marked with:
      - Quote generation ID (unique per quote operation)
      - Lexical scope reference
      - Hygiene color (for multi-level expansion)
    }
  }
}

-- ============================================================================
-- TYPE PRESERVATION
-- ============================================================================

type_preservation quote {
  rule full_type_info {
    exegesis {
      Quoted code preserves complete type information from the type
      checking phase. This enables type-safe code generation.
    }

    example {
      x: Int32 = 10
      y: Int32 = 20
      ast: Ast = '{ x + y }

      assert(ast.type_info.result_type == TypeInfo.of<Int32>())
    }
  }

  rule polymorphic_quote {
    exegesis {
      Quoted code can be polymorphic, capturing type parameters
      and constraints for later instantiation.
    }

    example {
      generic_max: Ast = '{
        function max<T>(a: T, b: T) -> T where T: Comparable {
          if a > b { return a } else { return b }
        }
      }
    }
  }
}

-- ============================================================================
-- SCOPING SEMANTICS
-- ============================================================================

scoping quote {
  rule lexical_capture {
    exegesis {
      Quote captures the lexical environment at the quote site.
      All free variables are resolved in the quote site's scope.
    }

    example {
      outer: Int32 = 100

      function create_closure() -> Ast {
        inner: Int32 = 200
        return '{ outer + inner }
      }

      ast: Ast = create_closure()
    }
  }

  rule scope_isolation {
    exegesis {
      Quoted blocks create isolated scopes that don't leak
      bindings to the surrounding code at quote time.
    }

    example {
      ast: Ast = '{
        local: Int32 = 42
        local + 10
      }

      result: Int32 = !ast  -- 52
    }
  }

  rule quasi_quote_scoping {
    exegesis {
      Quasi-quote holes ($identifier) create insertion points where
      external AST fragments can be spliced in during expansion.
    }

    example {
      macro build_expr(operand: Ast) -> Ast {
        offset: Int32 = 10
        return '{ $operand + offset }
      }

      user_expr: Ast = '{ x * 2 }
      result_ast: Ast = build_expr(user_expr)
    }
  }
}

-- ============================================================================
-- MLIR LOWERING
-- ============================================================================

mlir_lowering quote {
  representation {
    exegesis {
      Quoted ASTs are represented at compile-time as MLIR operations
      in the 'meta' dialect.
    }

    compile_time: """
      !meta.ast<
        node: !meta.binary_op<add>,
        type: i32,
        left: !meta.identifier<"x", type: i32>,
        right: !meta.identifier<"y", type: i32>
      >
    """
  }

  examples {
    example simple_expression {
      dol_code: """
        ast: Ast = '{ x + 42 }
      """

      mlir_ir: """
        %ast = meta.quote {
          %x = meta.identifier "x" : i32
          %c42 = meta.literal 42 : i32
          %add = meta.binary_op "add" (%x, %c42) : (i32, i32) -> i32
          meta.yield %add : !meta.ast<i32>
        } : !meta.ast<i32>
      """
    }

    example quasi_quote {
      dol_code: """
        macro double_and_add(expr: Ast) -> Ast {
          return '{ ($expr * 2) + 1 }
        }
      """

      mlir_ir: """
        %template = meta.quote {
          %hole = meta.hole "expr" : !meta.ast<i32>
          %c2 = meta.literal 2 : i32
          %mul = meta.binary_op "mul" (%hole, %c2) : (i32, i32) -> i32
          %c1 = meta.literal 1 : i32
          %add = meta.binary_op "add" (%mul, %c1) : (i32, i32) -> i32
          meta.yield %add : !meta.ast<i32>
        } : !meta.template<i32>

        %filled = meta.instantiate %template, %expr_arg
          : (!meta.template<i32>, !meta.ast<i32>) -> !meta.ast<i32>
      """
    }
  }
}

-- ============================================================================
-- CONSTRAINTS
-- ============================================================================

constraint quote_validity {
  rule well_formed {
    exegesis {
      Quoted code must be syntactically valid DOL code.
      Parse errors in quotes are compile-time errors.
    }
  }

  rule type_checked {
    exegesis {
      Quoted code must pass type checking at quote time.
      Exception: Quasi-quote holes defer type checking until instantiation.
    }
  }

  rule scope_valid {
    exegesis {
      All free variables in quoted code must be in scope at the quote site.
    }
  }
}

-- ============================================================================
-- RELATED CONSTRUCTS
-- ============================================================================

related_constructs {
  eval: "The ! operator evaluates quoted AST in a context"
  macro: "The # operator enables compile-time code generation using quotes"
  reflect: "The ? operator provides type introspection complementing quote"

  exegesis {
    Quote forms the foundation of DOL's metaprogramming triad:

    - Quote (') captures code as data
    - Eval (!) executes code from data
    - Macro (#) combines quote/eval for code generation
    - Reflect (?) inspects types at compile-time
  }
}

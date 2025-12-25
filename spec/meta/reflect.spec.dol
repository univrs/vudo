-- ============================================================================
-- DOL 2.0 META-PROGRAMMING: REFLECT OPERATOR SPECIFICATION
-- ============================================================================
-- Module: spec.meta.reflect
-- Version: 1.0.0
-- Purpose: Complete specification of the reflect operator (?) for introspection
-- ============================================================================

module spec.meta.reflect @ 1.0.0

exegesis {
  The reflect operator ? provides compile-time type introspection,
  enabling programs to examine and reason about types.

  Reflect enables:
  - Type introspection at compile-time
  - Generic programming with type constraints
  - Automatic trait implementations
  - Serialization/deserialization generation
  - Debug and display formatting

  Reflection in DOL is primarily compile-time, enabling zero-cost
  abstractions while providing powerful metaprogramming capabilities.
}

-- ============================================================================
-- REFLECT OPERATOR SYNTAX
-- ============================================================================

operator reflect {
  syntax: ?<type>
  syntax: ?<expression>
  syntax: ?typeof(<expression>)

  precedence: PREFIX
  associativity: N/A

  exegesis {
    Reflect operator has three forms:

    1. Type reflection: ?Int32
       Returns TypeInfo for the named type

    2. Expression reflection: ?value
       Returns TypeInfo for the expression's type

    3. Typeof reflection: ?typeof(expr)
       Explicit form for expression type reflection
  }

  type_signature: """
    reflect<T>() -> TypeInfo
    reflect<T>(value: T) -> TypeInfo
    typeof<T>(expr: T) -> TypeInfo
  """
}

-- ============================================================================
-- TYPE INFO STRUCTURE
-- ============================================================================

gene TypeInfo {
  type: {
    name: String,
    kind: TypeKind,
    size: Int64,
    alignment: Int64,
    fields: Vector<FieldInfo>,
    methods: Vector<MethodInfo>,
    traits: Vector<TraitInfo>,
    type_params: Vector<TypeParamInfo>,
    constraints: Vector<ConstraintInfo>
  }

  exegesis {
    TypeInfo provides complete type metadata:
    - name: Fully qualified type name
    - kind: Category (primitive, struct, enum, trait, etc.)
    - size: Size in bytes
    - alignment: Memory alignment requirement
    - fields: Struct/gene field information
    - methods: Associated functions
    - traits: Implemented traits
    - type_params: Generic parameters
    - constraints: Type constraints
  }
}

enum TypeKind {
  Primitive,
  Struct,
  Enum,
  Gene,
  Trait,
  Function,
  Array,
  Slice,
  Pointer,
  Optional,
  Result,
  Tuple,
  Never
}

struct FieldInfo {
  name: String,
  type_info: TypeInfo,
  offset: Int64,
  visibility: Visibility,
  attributes: Map<String, Any>
}

struct MethodInfo {
  name: String,
  signature: FunctionType,
  visibility: Visibility,
  is_static: Bool,
  attributes: Map<String, Any>
}

struct TraitInfo {
  name: String,
  methods: Vector<MethodInfo>,
  associated_types: Vector<TypeInfo>
}

-- ============================================================================
-- TYPE INTROSPECTION
-- ============================================================================

introspection {
  rule type_query {
    exegesis {
      Query type properties at compile-time.
    }

    examples {
      -- Get type info
      info: TypeInfo = ?Int32
      print(info.name)  -- "Int32"
      print(info.size)  -- 4

      -- Check type kind
      if ?String.kind == TypeKind.Struct {
        print("String is a struct")
      }

      -- Get field info
      person_info: TypeInfo = ?Person
      for field in person_info.fields {
        print(field.name + ": " + field.type_info.name)
      }
    }
  }

  rule trait_checking {
    exegesis {
      Check trait implementations at compile-time.
    }

    examples {
      -- Check if type implements trait
      if ?Int32.implements<Comparable>() {
        print("Int32 is comparable")
      }

      -- Get all implemented traits
      for trait in ?String.traits {
        print("String implements " + trait.name)
      }

      -- Constrain generics
      function sort<T>(items: Array<T>) -> Array<T>
        where ?T.implements<Comparable>()
      {
        -- sorting implementation
      }
    }
  }

  rule method_inspection {
    exegesis {
      Inspect methods at compile-time.
    }

    examples {
      -- Get all methods
      for method in ?Person.methods {
        print(method.name + ": " + method.signature.to_string())
      }

      -- Check for specific method
      if ?Person.has_method("to_string") {
        print("Person has to_string method")
      }

      -- Get method signature
      method: Optional<MethodInfo> = ?Person.get_method("greet")
      if method.is_some() {
        print("greet takes " + method.unwrap().signature.params.length().to_string() + " params")
      }
    }
  }
}

-- ============================================================================
-- COMPILE-TIME REFLECTION
-- ============================================================================

compile_time_reflection {
  exegesis {
    Most reflection operations execute at compile-time,
    with results embedded in generated code.
  }

  examples {
    -- Compile-time type size
    const SIZE: Int64 = ?MyStruct.size  -- Computed at compile-time

    -- Static assertions using reflection
    static_assert(?Vector<Int32>.is_generic())
    static_assert(?Int32.implements<Add>())

    -- Conditional compilation based on type properties
    macro generate_serializer<T>() -> Ast {
      if ?T.kind == TypeKind.Struct {
        return generate_struct_serializer<T>()
      } else if ?T.kind == TypeKind.Enum {
        return generate_enum_serializer<T>()
      } else {
        compile_error("Cannot generate serializer for " + ?T.name)
      }
    }
  }
}

-- ============================================================================
-- FIELD ENUMERATION
-- ============================================================================

field_enumeration {
  exegesis {
    Iterate over type fields at compile-time for code generation.
  }

  examples {
    macro for_each_field<T>(body: Ast) -> Ast {
      statements: Vector<Ast> = []
      for field in ?T.fields {
        field_name: String = field.name
        field_type: TypeInfo = field.type_info
        stmt: Ast = substitute(body, {
          'FIELD_NAME: field_name,
          'FIELD_TYPE: field_type.name
        })
        statements.push(stmt)
      }
      return '{ #splice(statements) }
    }

    -- Generate debug printer
    macro derive_debug<T>() -> Ast {
      return '{
        impl Debug for T {
          function debug_print(self) -> Void {
            print(?T.name + " {")
            #for_each_field<T>('{
              print("  " + FIELD_NAME + ": " + self.FIELD_NAME.to_string())
            })
            print("}")
          }
        }
      }
    }

    -- Generate equality check
    macro derive_eq<T>() -> Ast {
      return '{
        impl Eq for T {
          function eq(self, other: T) -> Bool {
            #for_each_field<T>('{
              if self.FIELD_NAME != other.FIELD_NAME {
                return false
              }
            })
            return true
          }
        }
      }
    }
  }
}

-- ============================================================================
-- GENERIC REFLECTION
-- ============================================================================

generic_reflection {
  exegesis {
    Reflect on generic types and type parameters.
  }

  examples {
    -- Inspect generic type
    generic_info: TypeInfo = ?Vector<Int32>
    print(generic_info.name)  -- "Vector<Int32>"
    print(generic_info.type_params[0].name)  -- "Int32"

    -- Check if type is generic
    if ?Vector<String>.is_generic() {
      print("Vector is a generic type")
    }

    -- Get base generic type
    base: TypeInfo = ?Vector<Int32>.generic_base()
    print(base.name)  -- "Vector<T>"

    -- Reflect on type parameters
    macro inspect_generics<T>() -> Void {
      for param in ?T.type_params {
        print("Type param: " + param.name)
        if param.has_constraint() {
          print("  Constraint: " + param.constraint.to_string())
        }
      }
    }
  }
}

-- ============================================================================
-- RUNTIME REFLECTION (LIMITED)
-- ============================================================================

runtime_reflection {
  exegesis {
    Limited runtime reflection for specific use cases.
    Most reflection is compile-time for zero overhead.
  }

  examples {
    -- Runtime type check
    function process(value: Any) -> Void {
      type_info: TypeInfo = value.get_type()

      match type_info.kind {
        TypeKind.Primitive => print("Got primitive"),
        TypeKind.Struct => print("Got struct: " + type_info.name),
        TypeKind.Enum => print("Got enum"),
        _ => print("Got other")
      }
    }

    -- Dynamic dispatch based on type
    function serialize(value: Any) -> String {
      type_info: TypeInfo = value.get_type()

      if type_info.implements_runtime<Serializable>() {
        return value.as<Serializable>().serialize()
      } else {
        return default_serialize(value, type_info)
      }
    }
  }

  limitation {
    exegesis {
      Runtime reflection requires:
      - Type metadata embedded in binary
      - Slightly larger binary size
      - Small runtime overhead for type queries

      Compile-time reflection has zero overhead.
    }
  }
}

-- ============================================================================
-- MLIR LOWERING
-- ============================================================================

mlir_lowering reflect {
  compile_time {
    exegesis {
      Compile-time reflection is fully resolved during compilation.
      Type information is computed and embedded as constants.
    }

    example {
      dol_code: """
        const SIZE: Int64 = ?Int32.size
        const NAME: String = ?Int32.name
      """

      mlir_ir: """
        %size = arith.constant 4 : i64
        %name = llvm.mlir.constant("Int32") : !llvm.ptr<i8>
      """
    }
  }

  runtime {
    exegesis {
      Runtime reflection uses type metadata tables.
    }

    example {
      dol_code: """
        function get_name(value: Any) -> String {
          return value.get_type().name
        }
      """

      mlir_ir: """
        func.func @get_name(%value: !dol.any) -> !llvm.ptr<i8> {
          // Get type ID from value
          %type_id = dol.any.type_id %value : i64

          // Look up type info in metadata table
          %type_info_ptr = llvm.getelementptr @type_metadata_table[%type_id]
            : !llvm.ptr<!dol.type_info>

          // Get name field
          %name_ptr = llvm.getelementptr %type_info_ptr[0, 0]
            : !llvm.ptr<!llvm.ptr<i8>>
          %name = llvm.load %name_ptr : !llvm.ptr<i8>

          return %name : !llvm.ptr<i8>
        }
      """
    }
  }

  metadata_table {
    exegesis {
      Type metadata is stored in a global table for runtime access.
    }

    structure: """
      struct TypeMetadata {
        name: *const u8,
        kind: u8,
        size: u64,
        alignment: u64,
        fields: *const FieldMetadata,
        num_fields: u32,
        traits: *const TraitMetadata,
        num_traits: u32
      }

      @type_metadata_table = global [...] !dol.type_metadata
    """
  }
}

-- ============================================================================
-- EXAMPLES
-- ============================================================================

exegesis examples {
  example type_description {
    description: "Generate human-readable type description"

    code: """
      function describe<T>() -> String {
        info: TypeInfo = ?T
        result: String = info.name + " ("

        match info.kind {
          TypeKind.Primitive => result += "primitive, ",
          TypeKind.Struct => result += "struct with " + info.fields.length().to_string() + " fields, ",
          TypeKind.Enum => result += "enum, ",
          _ => result += "other, "
        }

        result += info.size.to_string() + " bytes)"
        return result
      }

      print(describe<Int32>())   -- "Int32 (primitive, 4 bytes)"
      print(describe<Person>())  -- "Person (struct with 2 fields, 16 bytes)"
    """
  }

  example auto_serialization {
    description: "Automatic JSON serialization via reflection"

    code: """
      macro derive_json<T>() -> Ast {
        return '{
          impl JsonSerializable for T {
            function to_json(self) -> String {
              result: String = "{"
              first: Bool = true

              #for_each_field<T>('{
                if not first { result += ", " }
                first = false
                result += "\"" + FIELD_NAME + "\": "
                result += json_encode(self.FIELD_NAME)
              })

              result += "}"
              return result
            }
          }
        }
      }

      struct User {
        name: String,
        age: Int32,
        email: String
      }

      !derive_json<User>()

      user: User = User { name: "Alice", age: 30, email: "alice@example.com" }
      json: String = user.to_json()
      -- {"name": "Alice", "age": 30, "email": "alice@example.com"}
    """
  }

  example trait_dispatch {
    description: "Dynamic trait-based dispatch"

    code: """
      function print_if_printable<T>(value: T) -> Void {
        if ?T.implements<Display>() {
          print(value.to_display_string())
        } else if ?T.implements<Debug>() {
          print(value.debug_string())
        } else {
          print("<" + ?T.name + " (no display)>")
        }
      }

      print_if_printable(42)        -- "42"
      print_if_printable("hello")   -- "hello"
      print_if_printable(OpaqueType{})  -- "<OpaqueType (no display)>"
    """
  }
}

-- ============================================================================
-- CONSTRAINTS
-- ============================================================================

constraint reflect_validity {
  rule compile_time_preferred {
    exegesis {
      Compile-time reflection should be preferred for zero overhead.
      Runtime reflection adds overhead and should be used sparingly.
    }
  }

  rule type_safety {
    exegesis {
      Reflection operations preserve type safety.
      Field access via reflection is type-checked.
    }
  }

  rule privacy_respect {
    exegesis {
      Reflection respects visibility modifiers.
      Private fields are not accessible via public reflection APIs.
    }
  }
}

-- ============================================================================
-- RELATED CONSTRUCTS
-- ============================================================================

related_constructs {
  quote: "The ' operator captures code that can use reflected types"
  eval: "The ! operator can work with reflected type information"
  macro: "The # operator uses reflection for code generation"

  exegesis {
    Reflect completes DOL's metaprogramming system:

    - Quote captures code as data
    - Eval executes code from data
    - Macro transforms code using patterns
    - Reflect provides type information for all three

    Together they enable powerful, type-safe metaprogramming.
  }
}

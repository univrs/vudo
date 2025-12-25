-- ============================================================================
-- DOL 2.0 PRIMITIVE TYPES SPECIFICATION
-- ============================================================================
-- Module: spec.types.primitives
-- Version: 1.0.0
-- Purpose: Complete specification of DOL primitive types with MLIR lowering
-- ============================================================================

module spec.types.primitives @ 1.0.0

exegesis {
  This specification defines all primitive types in DOL 2.0.

  Primitive types form the foundation of DOL's type system, providing
  direct mappings to MLIR's builtin types for efficient code generation
  and optimization.

  Each type specification includes:
  - MLIR lowering target
  - Size and alignment guarantees
  - Memory layout characteristics
  - Literal syntax
  - Usage examples
  - Constraints and valid ranges

  All primitive types are copy-by-value and stored on the stack unless
  explicitly heap-allocated.
}

-- ============================================================================
-- SIGNED INTEGER TYPES
-- ============================================================================

type Int8 {
  mlir_type: i8

  layout {
    size_bytes: 1
    alignment: 1
    signedness: signed
    range: -128 to 127
    bit_width: 8
  }

  literal_syntax {
    decimal: 42i8, -127i8
    hex: 0x7Fi8, 0xFFi8
    binary: 0b01010101i8
    underscore_separator: 1_2_7i8
  }

  exegesis {
    Int8 represents an 8-bit signed two's complement integer.

    Range: -128 to 127 (inclusive)
    Storage: Single byte on stack
    MLIR: Lowers to i8 (8-bit signless integer, treated as signed)

    Use Int8 for:
    - Compact storage of small integer values
    - Byte-level protocol encoding
    - ASCII character codes (when treated as signed)
    - Delta encoding of small differences

    Constraints:
    - Overflow behavior is defined (wrapping semantics)
    - Division by zero triggers runtime panic
    - Bit operations preserve two's complement representation
  }

  examples {
    -- Basic usage
    age_delta: Int8 = -5i8
    temperature_offset: Int8 = 23i8

    -- Arithmetic (wrapping on overflow)
    result: Int8 = 120i8 + 20i8  -- Wraps to -116i8

    -- Conversion
    wide: Int32 = age_delta as Int32  -- Sign-extending cast

    -- Bounds checking
    function validate_small_int(value: Int8) -> Bool {
      return value >= -100i8 and value <= 100i8
    }
  }

  mlir_lowering {
    dol_code: "x: Int8 = 42i8"

    mlir_ir: """
      %x = arith.constant 42 : i8
    """
  }
}

type Int16 {
  mlir_type: i16

  layout {
    size_bytes: 2
    alignment: 2
    signedness: signed
    range: -32768 to 32767
    bit_width: 16
  }

  literal_syntax {
    decimal: 1000i16, -32768i16
    hex: 0x7FFFi16, 0x8000i16
    binary: 0b0111111111111111i16
    underscore_separator: 32_767i16
  }

  exegesis {
    Int16 represents a 16-bit signed two's complement integer.

    Range: -32,768 to 32,767 (inclusive)
    Storage: 2 bytes on stack, aligned to 2-byte boundary
    MLIR: Lowers to i16

    Use Int16 for:
    - Audio samples (PCM 16-bit)
    - Small counters and indices
    - Network protocol fields
    - Embedded systems with memory constraints

    Constraints:
    - Wrapping arithmetic on overflow
    - Division by zero is runtime error
    - Alignment required for efficient access on most architectures
  }

  examples {
    -- Audio sample
    sample: Int16 = -1024i16

    -- Port number (signed interpretation rare but valid)
    offset: Int16 = 8080i16

    -- Array indexing (when range known to be small)
    index: Int16 = 127i16
    positions: Array<Int16, 100> = Array.filled(0i16)

    -- Arithmetic with overflow checking
    function safe_add(a: Int16, b: Int16) -> Optional<Int16> {
      result: Int32 = (a as Int32) + (b as Int32)
      if result < -32768 or result > 32767 {
        return None
      }
      return Some(result as Int16)
    }
  }

  mlir_lowering {
    dol_code: "y: Int16 = y + 1i16"

    mlir_ir: """
      %one = arith.constant 1 : i16
      %result = arith.addi %y, %one : i16
    """
  }
}

type Int32 {
  mlir_type: i32

  layout {
    size_bytes: 4
    alignment: 4
    signedness: signed
    range: -2147483648 to 2147483647
    bit_width: 32
  }

  literal_syntax {
    decimal: 42, -2147483648, 1000000
    decimal_explicit: 42i32, -2147483648i32
    hex: 0x7FFFFFFF, 0x80000000
    binary: 0b01010101010101010101010101010101
    underscore_separator: 2_147_483_647

    note: "Int32 is the default integer type, suffix optional"
  }

  exegesis {
    Int32 represents a 32-bit signed two's complement integer.

    Range: -2,147,483,648 to 2,147,483,647 (inclusive)
    Storage: 4 bytes on stack, aligned to 4-byte boundary
    MLIR: Lowers to i32
    DEFAULT: Int32 is the default type for integer literals

    Use Int32 for:
    - General-purpose integer arithmetic
    - Loop counters and array indices
    - File sizes and offsets (for files < 2GB)
    - Process IDs, timestamps (Unix epoch seconds)
    - Most application-level integer values

    Constraints:
    - Wrapping semantics on overflow (can be checked explicitly)
    - Division by zero causes panic
    - Default type for integer literals without suffix
    - Most efficient integer size on 32-bit and 64-bit architectures
  }

  examples {
    -- Default integer literal type
    count: Int32 = 1000
    offset: Int32 = -500

    -- Explicit suffix (optional for Int32)
    capacity: Int32 = 1000i32

    -- Common use cases
    function factorial(n: Int32) -> Int32 {
      if n <= 1 {
        return 1
      }
      return n * factorial(n - 1)
    }

    -- Loop iteration
    sum: Int32 = 0
    for i in 1..100 {
      sum = sum + i
    }

    -- Checked arithmetic
    function checked_multiply(a: Int32, b: Int32) -> Optional<Int32> {
      result: Int64 = (a as Int64) * (b as Int64)
      if result < -2147483648 or result > 2147483647 {
        return None
      }
      return Some(result as Int32)
    }
  }

  mlir_lowering {
    dol_code: "result: Int32 = a * b + c"

    mlir_ir: """
      %mul = arith.muli %a, %b : i32
      %result = arith.addi %mul, %c : i32
    """
  }
}

type Int64 {
  mlir_type: i64

  layout {
    size_bytes: 8
    alignment: 8
    signedness: signed
    range: -9223372036854775808 to 9223372036854775807
    bit_width: 64
  }

  literal_syntax {
    decimal: 42i64, -9223372036854775808i64
    hex: 0x7FFFFFFFFFFFFFFFi64
    binary: 0b0111111111111111111111111111111111111111111111111111111111111111i64
    underscore_separator: 9_223_372_036_854_775_807i64
  }

  exegesis {
    Int64 represents a 64-bit signed two's complement integer.

    Range: -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807
    Storage: 8 bytes on stack, aligned to 8-byte boundary
    MLIR: Lowers to i64

    Use Int64 for:
    - High-precision timestamps (nanoseconds since epoch)
    - Large file sizes and offsets
    - Unique identifiers and database keys
    - High-precision counters
    - Scientific computing requiring large integer range
    - Financial calculations (when combined with fixed-point arithmetic)

    Constraints:
    - Wrapping semantics on overflow
    - Division by zero causes panic
    - Native size on 64-bit architectures
    - May require two registers on 32-bit systems
  }

  examples {
    -- Timestamps
    nanoseconds_since_epoch: Int64 = 1703174400000000000i64

    -- Large counters
    total_bytes_processed: Int64 = 0i64

    -- Unique IDs
    user_id: Int64 = 1234567890123456i64

    -- Time calculations
    function milliseconds_to_nanoseconds(ms: Int64) -> Int64 {
      return ms * 1_000_000i64
    }

    -- Large number arithmetic
    function fibonacci(n: Int32) -> Int64 {
      if n <= 1 {
        return n as Int64
      }
      a: Int64 = 0i64
      b: Int64 = 1i64
      for _ in 2..=n {
        temp: Int64 = a + b
        a = b
        b = temp
      }
      return b
    }
  }

  mlir_lowering {
    dol_code: "timestamp: Int64 = current_time() + 1000i64"

    mlir_ir: """
      %time = call @current_time() : () -> i64
      %offset = arith.constant 1000 : i64
      %timestamp = arith.addi %time, %offset : i64
    """
  }
}

-- ============================================================================
-- UNSIGNED INTEGER TYPES
-- ============================================================================

type UInt8 {
  mlir_type: ui8

  layout {
    size_bytes: 1
    alignment: 1
    signedness: unsigned
    range: 0 to 255
    bit_width: 8
  }

  literal_syntax {
    decimal: 42u8, 255u8
    hex: 0xFFu8
    binary: 0b11111111u8
    underscore_separator: 2_5_5u8
  }

  exegesis {
    UInt8 represents an 8-bit unsigned integer.

    Range: 0 to 255 (inclusive)
    Storage: Single byte on stack
    MLIR: Lowers to ui8 (unsigned integer interpretation)

    Use UInt8 for:
    - Raw byte data and buffers
    - RGB color components (0-255)
    - ASCII character codes
    - Network protocol bytes
    - Bitmap and binary data
    - Age values, percentages (when 0-100 range)

    Constraints:
    - Wrapping arithmetic on overflow (255 + 1 = 0)
    - Division by zero causes panic
    - No negative values (compilation error for negative literals)
    - Bitwise operations preserve unsigned interpretation
  }

  examples {
    -- Raw bytes
    byte_buffer: Array<UInt8, 1024> = Array.new()

    -- RGB color
    red: UInt8 = 255u8
    green: UInt8 = 128u8
    blue: UInt8 = 0u8

    -- Network packet
    type IpV4Header is {
      version_ihl: UInt8,
      dscp_ecn: UInt8,
      total_length: UInt16,
      -- ... more fields
    }

    -- Character encoding
    function is_ascii_printable(c: UInt8) -> Bool {
      return c >= 32u8 and c <= 126u8
    }

    -- Byte manipulation
    function swap_nibbles(byte: UInt8) -> UInt8 {
      high: UInt8 = (byte >> 4) & 0x0Fu8
      low: UInt8 = (byte << 4) & 0xF0u8
      return high | low
    }
  }

  mlir_lowering {
    dol_code: "b: UInt8 = 0xFFu8"

    mlir_ir: """
      %b = arith.constant 255 : ui8
    """
  }
}

type UInt16 {
  mlir_type: ui16

  layout {
    size_bytes: 2
    alignment: 2
    signedness: unsigned
    range: 0 to 65535
    bit_width: 16
  }

  literal_syntax {
    decimal: 1000u16, 65535u16
    hex: 0xFFFFu16
    binary: 0b1111111111111111u16
    underscore_separator: 65_535u16
  }

  exegesis {
    UInt16 represents a 16-bit unsigned integer.

    Range: 0 to 65,535 (inclusive)
    Storage: 2 bytes on stack, aligned to 2-byte boundary
    MLIR: Lowers to ui16

    Use UInt16 for:
    - Network port numbers (0-65535)
    - Unicode code points (BMP only, 0x0000-0xFFFF)
    - Small unsigned counters
    - Protocol length fields
    - Hardware registers

    Constraints:
    - Wrapping arithmetic on overflow
    - Division by zero causes panic
    - Alignment important for network byte order conversions
  }

  examples {
    -- Network port
    port: UInt16 = 8080u16

    -- Unicode BMP character
    unicode_char: UInt16 = 0x03B1u16  -- Greek alpha

    -- Length field
    packet_length: UInt16 = 1500u16

    -- Checksum calculation
    function compute_checksum(data: Slice<UInt8>) -> UInt16 {
      sum: UInt32 = 0u32
      for byte in data {
        sum = sum + (byte as UInt32)
      }
      -- Fold 32-bit sum to 16 bits
      while sum >> 16 > 0u32 {
        sum = (sum & 0xFFFFu32) + (sum >> 16)
      }
      return not (sum as UInt16)
    }

    -- Byte order conversion
    function to_big_endian(value: UInt16) -> UInt16 {
      return ((value & 0xFFu16) << 8) | ((value >> 8) & 0xFFu16)
    }
  }

  mlir_lowering {
    dol_code: "port: UInt16 = port + 1u16"

    mlir_ir: """
      %one = arith.constant 1 : ui16
      %result = arith.addi %port, %one : ui16
    """
  }
}

type UInt32 {
  mlir_type: ui32

  layout {
    size_bytes: 4
    alignment: 4
    signedness: unsigned
    range: 0 to 4294967295
    bit_width: 32
  }

  literal_syntax {
    decimal: 42u32, 4294967295u32
    hex: 0xFFFFFFFFu32
    binary: 0b11111111111111111111111111111111u32
    underscore_separator: 4_294_967_295u32
  }

  exegesis {
    UInt32 represents a 32-bit unsigned integer.

    Range: 0 to 4,294,967,295 (inclusive)
    Storage: 4 bytes on stack, aligned to 4-byte boundary
    MLIR: Lowers to ui32

    Use UInt32 for:
    - Hash values and checksums (CRC32, etc.)
    - IPv4 addresses (as 32-bit value)
    - File sizes (up to 4GB)
    - Unsigned counters and indices
    - Bit flags and masks
    - Memory addresses (32-bit systems)
    - RGBA colors (packed format)

    Constraints:
    - Wrapping arithmetic on overflow
    - Division by zero causes panic
    - Efficient on all modern architectures
    - Common size for hash functions
  }

  examples {
    -- IPv4 address
    ipv4: UInt32 = 0xC0A80001u32  -- 192.168.0.1

    -- Hash value
    hash: UInt32 = 0x5F3759DFu32

    -- RGBA color (packed)
    color: UInt32 = 0xFF8040FFu32  -- R=255, G=128, B=64, A=255

    -- Bit flags
    const READ_PERMISSION: UInt32 = 0x01u32
    const WRITE_PERMISSION: UInt32 = 0x02u32
    const EXECUTE_PERMISSION: UInt32 = 0x04u32

    permissions: UInt32 = READ_PERMISSION | WRITE_PERMISSION

    -- CRC32 implementation
    function crc32(data: Slice<UInt8>) -> UInt32 {
      crc: UInt32 = 0xFFFFFFFFu32
      for byte in data {
        crc = crc xor (byte as UInt32)
        for _ in 0..8 {
          mask: UInt32 = if (crc & 1u32) != 0u32 { 0xEDB88320u32 } else { 0u32 }
          crc = (crc >> 1) xor mask
        }
      }
      return not crc
    }

    -- Extract RGBA components
    function extract_rgba(color: UInt32) -> Tuple<UInt8, UInt8, UInt8, UInt8> {
      r: UInt8 = ((color >> 24) & 0xFFu32) as UInt8
      g: UInt8 = ((color >> 16) & 0xFFu32) as UInt8
      b: UInt8 = ((color >> 8) & 0xFFu32) as UInt8
      a: UInt8 = (color & 0xFFu32) as UInt8
      return (r, g, b, a)
    }
  }

  mlir_lowering {
    dol_code: "mask: UInt32 = flags & 0xFFu32"

    mlir_ir: """
      %mask_val = arith.constant 255 : ui32
      %mask = arith.andi %flags, %mask_val : ui32
    """
  }
}

type UInt64 {
  mlir_type: ui64

  layout {
    size_bytes: 8
    alignment: 8
    signedness: unsigned
    range: 0 to 18446744073709551615
    bit_width: 64
  }

  literal_syntax {
    decimal: 42u64, 18446744073709551615u64
    hex: 0xFFFFFFFFFFFFFFFFu64
    binary: 0b1111111111111111111111111111111111111111111111111111111111111111u64
    underscore_separator: 18_446_744_073_709_551_615u64
  }

  exegesis {
    UInt64 represents a 64-bit unsigned integer.

    Range: 0 to 18,446,744,073,709,551,615 (inclusive)
    Storage: 8 bytes on stack, aligned to 8-byte boundary
    MLIR: Lowers to ui64

    Use UInt64 for:
    - High-precision timestamps (microseconds, nanoseconds)
    - Very large file sizes and offsets
    - Unique identifiers (snowflake IDs, UUIDs low/high parts)
    - Memory addresses (64-bit systems)
    - Large counters (network bytes, database records)
    - Cryptographic operations
    - Bit manipulation requiring 64 bits

    Constraints:
    - Wrapping arithmetic on overflow
    - Division by zero causes panic
    - Native size on 64-bit architectures
    - Preferred for sizes and counts on modern systems
  }

  examples {
    -- Large file size
    file_size: UInt64 = 5_000_000_000u64  -- 5GB

    -- High-resolution timestamp
    nanos: UInt64 = 1703174400123456789u64

    -- Memory address
    address: UInt64 = 0x00007FFF5FBFFFFu64

    -- Snowflake ID
    type SnowflakeId is {
      timestamp: UInt64,  -- 41 bits
      worker_id: UInt64,  -- 10 bits
      sequence: UInt64    -- 12 bits
    }

    function generate_snowflake(worker: UInt64, sequence: UInt64) -> UInt64 {
      timestamp: UInt64 = get_milliseconds_since_epoch()
      return (timestamp << 22) | (worker << 12) | sequence
    }

    -- Large counter
    total_requests: UInt64 = 0u64

    function increment_counter(counter: UInt64) -> UInt64 {
      return counter + 1u64
    }

    -- Bit manipulation
    function count_ones(value: UInt64) -> UInt32 {
      count: UInt32 = 0u32
      temp: UInt64 = value
      while temp != 0u64 {
        count = count + 1u32
        temp = temp & (temp - 1u64)  -- Clear lowest set bit
      }
      return count
    }
  }

  mlir_lowering {
    dol_code: "counter: UInt64 = counter + 1u64"

    mlir_ir: """
      %one = arith.constant 1 : ui64
      %counter = arith.addi %counter, %one : ui64
    """
  }
}

-- ============================================================================
-- FLOATING POINT TYPES
-- ============================================================================

type Float32 {
  mlir_type: f32

  layout {
    size_bytes: 4
    alignment: 4
    format: ieee754_binary32
    precision: 24_bits_significand
    exponent_range: -126 to 127
    decimal_digits: approximately_7
  }

  literal_syntax {
    decimal: 3.14f32, -0.5f32, 1.0f32
    scientific: 1.23e-4f32, 6.022e23f32
    special: inf, -inf, nan
    underscore_separator: 3.141_592_653f32
  }

  exegesis {
    Float32 represents a 32-bit IEEE 754 binary floating-point number.

    Format: 1 sign bit, 8 exponent bits, 23 mantissa bits
    Range: ±1.175494e-38 to ±3.402823e+38
    Precision: ~7 decimal digits
    Storage: 4 bytes on stack, aligned to 4-byte boundary
    MLIR: Lowers to f32

    Use Float32 for:
    - Graphics and game development (coordinates, vectors)
    - Audio processing (samples, DSP)
    - Machine learning (neural network weights)
    - Physics simulations (when precision allows)
    - GPU computing (native float size)
    - 3D transformations and matrices

    Constraints:
    - NaN != NaN (IEEE 754 semantics)
    - Infinity is a valid value
    - Denormalized numbers supported
    - Loss of precision in some arithmetic operations
    - Not all decimal fractions representable exactly

    Special values:
    - inf: Positive infinity
    - -inf: Negative infinity
    - nan: Not a Number (invalid operations)
    - 0.0 and -0.0 are distinct but compare equal
  }

  examples {
    -- Basic usage
    pi: Float32 = 3.14159f32
    half: Float32 = 0.5f32

    -- Scientific notation
    avogadro: Float32 = 6.022e23f32
    electron_mass: Float32 = 9.109e-31f32

    -- 3D vector
    type Vec3 is {
      x: Float32,
      y: Float32,
      z: Float32
    }

    function dot_product(a: Vec3, b: Vec3) -> Float32 {
      return a.x * b.x + a.y * b.y + a.z * b.z
    }

    function magnitude(v: Vec3) -> Float32 {
      return sqrt(v.x * v.x + v.y * v.y + v.z * v.z)
    }

    -- Special value handling
    function safe_divide(a: Float32, b: Float32) -> Optional<Float32> {
      result: Float32 = a / b
      if result.is_nan() or result.is_infinite() {
        return None
      }
      return Some(result)
    }

    -- Comparison with tolerance
    function approximately_equal(a: Float32, b: Float32, epsilon: Float32) -> Bool {
      return abs(a - b) < epsilon
    }
  }

  mlir_lowering {
    dol_code: "result: Float32 = a * b + c"

    mlir_ir: """
      %mul = arith.mulf %a, %b : f32
      %result = arith.addf %mul, %c : f32
    """
  }
}

type Float64 {
  mlir_type: f64

  layout {
    size_bytes: 8
    alignment: 8
    format: ieee754_binary64
    precision: 53_bits_significand
    exponent_range: -1022 to 1023
    decimal_digits: approximately_15_to_17
  }

  literal_syntax {
    decimal: 3.14, -0.5, 1.0, 2.718281828459045
    decimal_explicit: 3.14f64, -0.5f64
    scientific: 1.23e-4, 6.022e23, 1.0e-100f64
    special: inf, -inf, nan
    underscore_separator: 3.141_592_653_589_793

    note: "Float64 is the default floating-point type, suffix optional"
  }

  exegesis {
    Float64 represents a 64-bit IEEE 754 binary floating-point number.

    Format: 1 sign bit, 11 exponent bits, 52 mantissa bits
    Range: ±2.225074e-308 to ±1.797693e+308
    Precision: ~15-17 decimal digits
    Storage: 8 bytes on stack, aligned to 8-byte boundary
    MLIR: Lowers to f64
    DEFAULT: Float64 is the default type for floating-point literals

    Use Float64 for:
    - Scientific computing and numerical analysis
    - Financial calculations requiring precision
    - Coordinate systems requiring high accuracy
    - Statistical computations
    - General-purpose floating-point arithmetic
    - Physics simulations requiring precision

    Constraints:
    - NaN != NaN (IEEE 754 semantics)
    - Infinity is a valid value
    - Denormalized numbers supported
    - Still subject to rounding errors (not arbitrary precision)
    - Comparison should use epsilon for equality tests

    Special values:
    - inf: Positive infinity
    - -inf: Negative infinity
    - nan: Not a Number
    - 0.0 and -0.0 are distinct but compare equal
  }

  examples {
    -- Default floating-point literal type
    e: Float64 = 2.718281828459045
    golden_ratio: Float64 = 1.618033988749895

    -- Scientific notation
    planck: Float64 = 6.62607015e-34
    speed_of_light: Float64 = 299792458.0

    -- High-precision calculations
    function compute_stddev(values: Slice<Float64>) -> Float64 {
      mean: Float64 = values |> sum() / (values.length as Float64)

      variance: Float64 = 0.0
      for value in values {
        diff: Float64 = value - mean
        variance = variance + diff * diff
      }
      variance = variance / (values.length as Float64)

      return sqrt(variance)
    }

    -- Financial calculations (fixed-point alternative recommended)
    function compound_interest(
      principal: Float64,
      rate: Float64,
      time: Float64,
      n: Float64
    ) -> Float64 {
      return principal * pow(1.0 + rate / n, n * time)
    }

    -- Numerical methods
    function bisection_root(
      f: Function<Float64, Float64>,
      a: Float64,
      b: Float64,
      tolerance: Float64
    ) -> Optional<Float64> {
      if f(a) * f(b) >= 0.0 {
        return None
      }

      mid: Float64 = a
      while (b - a) >= tolerance {
        mid = (a + b) / 2.0

        if f(mid) == 0.0 {
          break
        }

        if f(mid) * f(a) < 0.0 {
          b = mid
        } else {
          a = mid
        }
      }

      return Some(mid)
    }

    -- Safe comparison
    const EPSILON: Float64 = 1e-10

    function float_equal(a: Float64, b: Float64) -> Bool {
      return abs(a - b) < EPSILON
    }
  }

  mlir_lowering {
    dol_code: "area: Float64 = radius * radius * 3.14159"

    mlir_ir: """
      %r_squared = arith.mulf %radius, %radius : f64
      %pi = arith.constant 3.14159 : f64
      %area = arith.mulf %r_squared, %pi : f64
    """
  }
}

-- ============================================================================
-- BOOLEAN AND VOID TYPES
-- ============================================================================

type Bool {
  mlir_type: i1

  layout {
    size_bytes: 1
    alignment: 1
    representation: i1_zero_or_one
    bit_width: 1_logical_8_physical
  }

  literal_syntax {
    values: true, false
  }

  exegesis {
    Bool represents a Boolean truth value: true or false.

    Values: true (1) or false (0)
    Storage: Logically 1 bit, physically 1 byte in memory
    MLIR: Lowers to i1 (1-bit signless integer)

    Use Bool for:
    - Conditional expressions and control flow
    - Flags and state indicators
    - Logical operations (and, or, not)
    - Function return values for predicates
    - Bit flags (though UInt8/UInt32 preferred for packed flags)

    Constraints:
    - Only two valid values: true and false
    - Short-circuit evaluation in logical operators
    - Zero-cost conversion to/from i1 in MLIR
    - Stored as byte in memory, but treated as single bit logically

    Logical operators:
    - and: Logical AND (short-circuits)
    - or: Logical OR (short-circuits)
    - not: Logical NOT
    - ==, !=: Equality comparison
  }

  examples {
    -- Basic usage
    is_valid: Bool = true
    is_empty: Bool = false

    -- Conditional logic
    function can_process(enabled: Bool, has_data: Bool) -> Bool {
      return enabled and has_data
    }

    -- Predicate functions
    function is_even(n: Int32) -> Bool {
      return n % 2 == 0
    }

    function is_positive(x: Float64) -> Bool {
      return x > 0.0
    }

    -- Short-circuit evaluation
    function safe_access(items: Slice<Int32>, index: UInt64) -> Bool {
      -- Second condition only evaluated if first is true
      return index < items.length and items[index] > 0
    }

    -- Logical combinations
    function validate_input(
      value: Int32,
      min: Int32,
      max: Int32,
      allow_zero: Bool
    ) -> Bool {
      in_range: Bool = value >= min and value <= max
      valid_zero: Bool = allow_zero or value != 0
      return in_range and valid_zero
    }

    -- Pattern matching
    function describe(flag: Bool) -> String {
      match flag {
        true { return "enabled" }
        false { return "disabled" }
      }
    }

    -- Conversion from comparison
    result: Bool = x > y
    equal: Bool = a == b
    not_equal: Bool = a != b
  }

  mlir_lowering {
    dol_code: "result: Bool = a > 0 and b < 100"

    mlir_ir: """
      %zero = arith.constant 0 : i32
      %hundred = arith.constant 100 : i32
      %cond1 = arith.cmpi sgt, %a, %zero : i32
      %cond2 = arith.cmpi slt, %b, %hundred : i32
      %result = arith.andi %cond1, %cond2 : i1
    """
  }
}

type Void {
  mlir_type: none

  layout {
    size_bytes: 0
    alignment: 1
    representation: unit_type_no_runtime_value
  }

  literal_syntax {
    values: none
    note: "Void has no literal value; used only in type position"
  }

  exegesis {
    Void represents the absence of a value (unit type).

    Size: Zero bytes (no runtime representation)
    MLIR: Lowers to none (MLIR's unit type)

    Use Void for:
    - Functions that perform side effects but return no value
    - Indicating absence of meaningful return value
    - Type-level programming where "no value" is needed
    - Trait methods that don't return data

    Constraints:
    - Cannot be instantiated or stored in variables
    - Only valid as function return type or in type expressions
    - Zero runtime cost (optimized away)
    - Not the same as Optional<T> (which has runtime representation)

    Relationship to other types:
    - Void: No value ever (type-level absence)
    - Optional<T>: Maybe a value (runtime absence)
    - Result<T, E>: Either value or error (runtime alternatives)
  }

  examples {
    -- Functions returning Void
    function print_message(msg: String) -> Void {
      io.print(msg)
      -- Implicit return (no value)
    }

    function increment_counter(counter: UInt64) -> Void {
      counter = counter + 1u64
    }

    -- Side-effect functions
    function log_event(event: String) -> Void {
      timestamp: Int64 = get_current_time()
      write_log(timestamp, event)
    }

    -- Trait with Void method
    trait Drawable {
      requires draw: Function<Self, Void>
    }

    type Circle is {
      radius: Float64
    }

    implement Drawable for Circle {
      function draw(self: Circle) -> Void {
        render_circle(self.radius)
      }
    }

    -- Void in generic contexts
    function apply<T>(f: Function<T, Void>, value: T) -> Void {
      f(value)
    }

    -- Contrast with Optional
    function find(items: Slice<Int32>, target: Int32) -> Optional<UInt64> {
      -- Returns Some(index) or None
    }

    function print_all(items: Slice<Int32>) -> Void {
      -- Returns nothing (Void)
    }
  }

  mlir_lowering {
    dol_code: """
      function notify() -> Void {
        send_message("hello")
      }
    """

    mlir_ir: """
      func.func @notify() {
        %msg = llvm.mlir.addressof @str_hello : !llvm.ptr
        call @send_message(%msg) : (!llvm.ptr) -> ()
        return
      }
    """
  }
}

-- ============================================================================
-- STRING TYPE
-- ============================================================================

type String {
  mlir_type: !dol.string

  layout {
    size_bytes: 16_or_24_depending_on_representation
    alignment: 8
    representation: fat_pointer_ptr_length_capacity
    encoding: utf8
    storage: heap_allocated
  }

  literal_syntax {
    basic: "hello", "world"
    escaped: "line1\nline2", "tab\there", "quote\"inside"
    raw: r"C:\path\to\file", r"regex\d+"
    multiline: """
      This is a
      multiline string
      with preserved formatting
    """

    escape_sequences: {
      newline: "\n"
      tab: "\t"
      carriage_return: "\r"
      quote: "\""
      backslash: "\\"
      unicode: "\u{1F600}"  -- Emoji
      hex: "\x48"  -- 'H'
    }
  }

  exegesis {
    String represents a UTF-8 encoded, heap-allocated, growable string.

    Encoding: UTF-8 (variable-length, 1-4 bytes per code point)
    Storage: Heap-allocated with fat pointer (ptr, length, capacity)
    Size: 16-24 bytes on stack (pointer + metadata)
    MLIR: Lowers to !dol.string (custom DOL dialect type)

    Structure (conceptual):
    - ptr: Pointer to heap-allocated UTF-8 bytes
    - length: Number of bytes (NOT characters)
    - capacity: Allocated capacity (for growth)

    Use String for:
    - Text processing and manipulation
    - User input and output
    - File paths (though Path type preferred)
    - JSON, XML, and other text formats
    - Logging and error messages
    - Configuration values

    Constraints:
    - Always valid UTF-8 (construction validates)
    - Heap-allocated (not suitable for embedded/real-time without allocator)
    - Length is byte count, not character count
    - Indexed by byte position, not character position
    - Immutable slices via string views
    - Clone required for independent copies

    Operations:
    - Concatenation: + operator or concat()
    - Slicing: substring operations
    - Iteration: by characters (code points) or bytes
    - Comparison: lexicographic ordering
    - Pattern matching: contains, starts_with, ends_with
    - Conversion: to/from bytes, integers, floats
  }

  examples {
    -- Basic strings
    greeting: String = "Hello, world!"
    empty: String = ""

    -- Escape sequences
    multiline: String = "Line 1\nLine 2\nLine 3"
    path: String = "C:\\Users\\name\\file.txt"
    raw_path: String = r"C:\Users\name\file.txt"

    -- Unicode support
    emoji: String = "Hello 🌍"
    greek: String = "Ελληνικά"
    chinese: String = "中文"

    -- String operations
    function greet(name: String) -> String {
      return "Hello, " + name + "!"
    }

    -- Length and indexing (byte-based)
    function get_byte_length(s: String) -> UInt64 {
      return s.len()  -- Byte count, not character count
    }

    -- Character iteration
    function count_characters(s: String) -> UInt64 {
      count: UInt64 = 0u64
      for _ in s.chars() {
        count = count + 1u64
      }
      return count
    }

    -- String comparison
    function is_prefix(text: String, prefix: String) -> Bool {
      return text.starts_with(prefix)
    }

    -- Pattern matching
    function contains_word(text: String, word: String) -> Bool {
      return text.contains(word)
    }

    -- String building
    function build_csv(fields: Slice<String>) -> String {
      result: String = ""
      for i, field in fields.enumerate() {
        if i > 0u64 {
          result = result + ","
        }
        result = result + field
      }
      return result
    }

    -- Conversion
    function parse_integer(s: String) -> Optional<Int32> {
      return s.parse::<Int32>()
    }

    function int_to_string(n: Int32) -> String {
      return n.to_string()
    }

    -- String interpolation (if supported)
    name: String = "Alice"
    age: Int32 = 30
    message: String = "Name: {name}, Age: {age}"

    -- Slicing
    function first_word(text: String) -> Optional<String> {
      space_index: Optional<UInt64> = text.find(' ')
      match space_index {
        Some(idx) { return Some(text.substring(0u64, idx)) }
        None { return Some(text) }
      }
    }
  }

  mlir_lowering {
    dol_code: """
      message: String = "Hello, " + name
    """

    mlir_ir: """
      %hello = dol.string.literal "Hello, " : !dol.string
      %message = dol.string.concat %hello, %name : !dol.string
    """

    note: """
      The !dol.string type is a custom MLIR dialect type that encapsulates:
      - Heap allocation management
      - UTF-8 validation
      - Reference counting or ownership semantics
      - String operations (concat, slice, etc.)

      Lowering to LLVM IR expands to:
      - Struct with ptr, length, capacity fields
      - Calls to runtime string library
      - Memory allocation/deallocation
    """
  }
}

-- ============================================================================
-- TYPE RELATIONSHIPS AND CONVERSIONS
-- ============================================================================

exegesis {
  TYPE HIERARCHY
  ==============

  Primitive types form a flat hierarchy with explicit conversions:

  Integer conversions:
  - Widening conversions are safe (Int8 -> Int16 -> Int32 -> Int64)
  - Narrowing conversions require explicit cast and may truncate
  - Signed <-> Unsigned conversions require explicit cast

  Floating-point conversions:
  - Float32 -> Float64 is safe (widening)
  - Float64 -> Float32 may lose precision (narrowing)

  Integer <-> Float conversions:
  - Int/UInt -> Float requires explicit cast
  - Float -> Int/UInt requires explicit cast and truncates (rounds toward zero)

  Bool conversions:
  - No implicit conversions to/from integers
  - Explicit: 0 -> false, non-zero -> true (when cast)
  - Explicit: false -> 0, true -> 1 (when cast)

  String conversions:
  - All primitive types can convert to String via to_string()
  - String can parse to primitives via parse::<T>() (returns Optional)


  LITERAL SUFFIXES
  ================

  Integer suffixes:
    i8, i16, i32, i64    -- Signed integers
    u8, u16, u32, u64    -- Unsigned integers
    (no suffix)          -- Defaults to Int32

  Float suffixes:
    f32                  -- 32-bit float
    f64 or (no suffix)   -- 64-bit float (default)


  NUMERIC PROMOTIONS
  ==================

  Binary operations require matching types:

  Valid:
    a: Int32 = 5
    b: Int32 = 10
    c: Int32 = a + b  -- Same types

  Invalid:
    a: Int32 = 5
    b: Int64 = 10
    c = a + b  -- ERROR: Type mismatch

  Correct:
    a: Int32 = 5
    b: Int64 = 10
    c: Int64 = (a as Int64) + b  -- Explicit cast


  OVERFLOW BEHAVIOR
  =================

  All integer arithmetic uses wrapping semantics by default:

    max: Int8 = 127i8
    overflow: Int8 = max + 1i8  -- Wraps to -128i8

  For checked arithmetic, use explicit functions:

    result: Optional<Int32> = checked_add(a, b)
    result: Int32 = saturating_add(a, b)  -- Clamps to min/max


  SPECIAL FLOAT VALUES
  ====================

  IEEE 754 special values are supported:

    positive_inf: Float64 = inf
    negative_inf: Float64 = -inf
    not_a_number: Float64 = nan

  Comparisons:
    - NaN != NaN (always true)
    - Any comparison with NaN returns false (except !=)
    - inf > any finite number
    - -inf < any finite number

  Testing:
    is_nan: Bool = x.is_nan()
    is_infinite: Bool = x.is_infinite()
    is_finite: Bool = x.is_finite()
}

-- ============================================================================
-- MEMORY LAYOUT GUARANTEES
-- ============================================================================

exegesis {
  MEMORY LAYOUT
  =============

  All primitive types have defined size and alignment:

  Type      | Size | Align | Notes
  ----------|------|-------|----------------------------------
  Int8      | 1    | 1     | Two's complement
  Int16     | 2    | 2     | Two's complement
  Int32     | 4    | 4     | Two's complement
  Int64     | 8    | 8     | Two's complement
  UInt8     | 1    | 1     | Unsigned
  UInt16    | 2    | 2     | Unsigned
  UInt32    | 4    | 4     | Unsigned
  UInt64    | 8    | 8     | Unsigned
  Float32   | 4    | 4     | IEEE 754 binary32
  Float64   | 8    | 8     | IEEE 754 binary64
  Bool      | 1    | 1     | 0 = false, 1 = true
  Void      | 0    | 1     | No runtime representation
  String    | 16+  | 8     | Fat pointer (ptr + len + cap)


  ENDIANNESS
  ==========

  DOL follows platform endianness for in-memory representation.
  For portable binary formats, use explicit conversion functions:

    to_big_endian(value)
    to_little_endian(value)
    from_big_endian(bytes)
    from_little_endian(bytes)


  STRUCT PACKING
  ==============

  Structs containing primitive types follow standard C ABI alignment rules:

    type Point is {
      x: Float64,  -- Offset 0, size 8
      y: Float64   -- Offset 8, size 8
    }  -- Total size: 16, alignment: 8

    type Mixed is {
      a: UInt8,    -- Offset 0, size 1
      -- Padding: 3 bytes
      b: Int32,    -- Offset 4, size 4
      c: UInt8     -- Offset 8, size 1
      -- Padding: 7 bytes (for 8-byte alignment)
    }  -- Total size: 16, alignment: 8
}

-- ============================================================================
-- MLIR DIALECT INTEGRATION
-- ============================================================================

exegesis {
  MLIR TYPE MAPPING
  =================

  DOL primitive types map to MLIR builtin types and custom dialect types:

  DOL Type   | MLIR Type        | Dialect
  -----------|------------------|------------------
  Int8       | i8               | builtin
  Int16      | i16              | builtin
  Int32      | i32              | builtin
  Int64      | i64              | builtin
  UInt8      | ui8              | builtin (signless with unsigned semantics)
  UInt16     | ui16             | builtin (signless with unsigned semantics)
  UInt32     | ui32             | builtin (signless with unsigned semantics)
  UInt64     | ui64             | builtin (signless with unsigned semantics)
  Float32    | f32              | builtin
  Float64    | f64              | builtin
  Bool       | i1               | builtin
  Void       | none             | builtin
  String     | !dol.string      | dol dialect (custom)


  ARITHMETIC OPERATIONS
  =====================

  Integer operations (arith dialect):
    +  : arith.addi
    -  : arith.subi
    *  : arith.muli
    /  : arith.divsi (signed) / arith.divui (unsigned)
    %  : arith.remsi (signed) / arith.remui (unsigned)
    &  : arith.andi
    |  : arith.ori
    ^  : arith.xori
    << : arith.shli
    >> : arith.shrsi (signed) / arith.shrui (unsigned)

  Float operations (arith dialect):
    +  : arith.addf
    -  : arith.subf
    *  : arith.mulf
    /  : arith.divf
    %  : arith.remf

  Comparison operations (arith dialect):
    ==  : arith.cmpi eq (int) / arith.cmpf oeq (float)
    !=  : arith.cmpi ne (int) / arith.cmpf one (float)
    <   : arith.cmpi slt/ult (int) / arith.cmpf olt (float)
    <=  : arith.cmpi sle/ule (int) / arith.cmpf ole (float)
    >   : arith.cmpi sgt/ugt (int) / arith.cmpf ogt (float)
    >=  : arith.cmpi sge/uge (int) / arith.cmpf oge (float)


  LOWERING PIPELINE
  =================

  DOL Source Code
        |
        v
  [DOL Parser] --> AST
        |
        v
  [Type Checker] --> Typed AST
        |
        v
  [MLIR Codegen] --> High-level MLIR (dol dialect)
        |
        v
  [Dialect Lowering] --> Mid-level MLIR (arith, scf, func)
        |
        v
  [Target Lowering] --> Low-level MLIR (llvm dialect)
        |
        v
  [LLVM Backend] --> LLVM IR
        |
        v
  [LLVM Optimization + Codegen]
        |
        +---> WASM (via LLVM WASM backend)
        |
        +---> Native (x86-64, ARM64, etc.)
}

-- ============================================================================
-- END OF SPECIFICATION
-- ============================================================================

exegesis {
  This specification defines the complete set of primitive types in DOL 2.0.

  All primitive types are:
  - Value types (copied, not referenced)
  - Stack-allocated by default
  - Zero-cost abstractions (compile to native MLIR types)
  - Explicitly convertible (no implicit coercion)

  For composite types (arrays, slices, pointers, etc.), see:
    - spec/types/composite.spec.dol

  For ontological types (Gene, Trait, Constraint, etc.), see:
    - spec/types/ontological.spec.dol

  Version: 1.0.0
  Last updated: 2024
  Status: CANONICAL SPECIFICATION
}

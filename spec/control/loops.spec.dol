-- ============================================================================
-- DOL 2.0 LOOP CONSTRUCTS SPECIFICATION
-- ============================================================================
-- Module: spec.control.loops
-- Version: 1.0.0
-- Purpose: Loop constructs with MLIR SCF lowering
-- ============================================================================

module spec.control.loops @ 1.0.0

exegesis {
  DOL provides three loop constructs:
  - for: Iterating over ranges or collections
  - while: Condition-controlled iteration
  - loop: Infinite loop with explicit break

  All loops support break and continue with optional labels.
  Loops lower to MLIR SCF dialect operations (scf.for, scf.while).
}

-- ============================================================================
-- FOR LOOP
-- ============================================================================

construct for_loop {
  syntax {
    for <pattern> in <iterator> { <body> }
    for <variable> in <start>..<end> { <body> }
    for <variable> in <start>..=<end> { <body> }
  }

  exegesis {
    Iterates over ranges or anything implementing Iterable trait.

    Range forms:
    - start..end: Exclusive end (0..10 = 0,1,2,...,9)
    - start..=end: Inclusive end (0..=10 = 0,1,2,...,10)

    Loop variable is immutable within body by default.
  }

  examples {
    -- Range loop
    for i in 0..10 {
      print(i)
    }

    -- Inclusive range
    for i in 1..=100 {
      sum = sum + i
    }

    -- Collection iteration
    for item in collection {
      process(item)
    }

    -- Destructuring
    for (key, value) in map.entries() {
      print(key + ": " + value)
    }

    -- With index
    for (i, item) in collection.enumerate() {
      print(i.to_string() + ": " + item)
    }
  }

  mlir_lowering {
    dol_code: """
      sum: Int32 = 0
      for i in 0..10 {
        sum = sum + i
      }
    """

    mlir_ir: """
      %c0 = arith.constant 0 : index
      %c10 = arith.constant 10 : index
      %c1 = arith.constant 1 : index
      %init_sum = arith.constant 0 : i32

      %final_sum = scf.for %i = %c0 to %c10 step %c1
          iter_args(%sum = %init_sum) -> i32 {
        %i_i32 = arith.index_cast %i : index to i32
        %new_sum = arith.addi %sum, %i_i32 : i32
        scf.yield %new_sum : i32
      }
    """
  }
}

-- ============================================================================
-- WHILE LOOP
-- ============================================================================

construct while_loop {
  syntax {
    while <condition> { <body> }
  }

  exegesis {
    Executes body while condition is true.
    Condition is re-evaluated before each iteration.
    If condition is initially false, body never executes.
  }

  examples {
    -- Simple while
    while count > 0 {
      process()
      count = count - 1
    }

    -- With complex condition
    while buffer.has_data() and not cancelled {
      chunk = buffer.read(1024)
      send(chunk)
    }
  }

  mlir_lowering {
    dol_code: """
      while x > 0 {
        x = x - 1
      }
    """

    mlir_ir: """
      %c0 = arith.constant 0 : i32
      %c1 = arith.constant 1 : i32

      scf.while (%x_val = %x) : (i32) -> i32 {
        %cond = arith.cmpi sgt, %x_val, %c0 : i32
        scf.condition(%cond) %x_val : i32
      } do {
      ^bb0(%arg: i32):
        %new_x = arith.subi %arg, %c1 : i32
        scf.yield %new_x : i32
      }
    """
  }
}

-- ============================================================================
-- LOOP (INFINITE)
-- ============================================================================

construct loop_infinite {
  syntax {
    loop { <body> }
  }

  exegesis {
    Infinite loop - runs until break.
    Useful for event loops, servers, state machines.
    Must contain break or return to terminate.
  }

  examples {
    -- Event loop
    loop {
      event = poll_events()
      match event {
        Event.Quit => break,
        Event.Key(k) => handle_key(k),
        Event.Mouse(m) => handle_mouse(m),
      }
    }

    -- Retry with backoff
    loop {
      result = try_connect()
      if result.is_ok() {
        break
      }
      delay = delay * 2
      sleep(delay)
    }
  }

  mlir_lowering {
    note: """
      loop lowers to scf.while with always-true condition.
      break lowers to scf.condition(false).
    """
  }
}

-- ============================================================================
-- BREAK AND CONTINUE
-- ============================================================================

construct break_continue {
  syntax {
    break [<label>]
    continue [<label>]
  }

  exegesis {
    break: Exit the innermost (or labeled) loop
    continue: Skip to next iteration of innermost (or labeled) loop

    Labels enable breaking out of nested loops:
    'outer: for i in 0..10 {
      for j in 0..10 {
        if condition { break 'outer }
      }
    }
  }

  examples {
    -- Simple break
    for i in 0..100 {
      if found(i) {
        result = i
        break
      }
    }

    -- Labeled break
    'outer: for x in 0..10 {
      for y in 0..10 {
        if matrix[x][y] == target {
          found_x = x
          found_y = y
          break 'outer
        }
      }
    }

    -- Continue
    for item in items {
      if item.skip {
        continue
      }
      process(item)
    }
  }

  mlir_lowering {
    note: """
      break lowers to early exit from scf.for/while.
      continue lowers to scf.yield with current values.
      Labeled breaks use block arguments to track exit points.
    """
  }
}

-- ============================================================================
-- LOOP EXPRESSIONS
-- ============================================================================

construct loop_expressions {
  exegesis {
    Loops can be expressions when they always produce a value via break.

    loop {
      if condition { break value }
    }

    The type of the loop expression is the type of break values.
    All break statements must provide compatible types.
  }

  examples {
    result: Int32 = loop {
      candidate = generate()
      if is_valid(candidate) {
        break candidate
      }
    }

    -- For loops with collection result
    squares: Array<Int32> = for i in 0..10 {
      yield i * i
    }
  }
}

;; Counter Spirit - Stateful counter module
;;
;; Demonstrates mutable global state in WASM.
;; Provides init, increment, decrement, and get operations.

(module
  ;; Mutable global variable to hold the counter value
  ;; Initialized to 0
  (global $counter (mut i32) (i32.const 0))

  ;; Initialize the counter to 0
  ;; Resets the counter state
  (func $init (export "init")
    i32.const 0
    global.set $counter
  )

  ;; Increment the counter by 1
  ;; Returns the new counter value
  (func $increment (export "increment") (result i32)
    ;; Get current value, add 1, store back
    global.get $counter
    i32.const 1
    i32.add
    global.set $counter
    ;; Return the new value
    global.get $counter
  )

  ;; Decrement the counter by 1
  ;; Returns the new counter value
  (func $decrement (export "decrement") (result i32)
    ;; Get current value, subtract 1, store back
    global.get $counter
    i32.const 1
    i32.sub
    global.set $counter
    ;; Return the new value
    global.get $counter
  )

  ;; Get the current counter value without modifying it
  (func $get (export "get") (result i32)
    global.get $counter
  )
)

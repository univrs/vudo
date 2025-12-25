;; Hello World Spirit - Basic greeting Spirit
;;
;; A minimal WASM module demonstrating basic structure.
;; Exports a single function that returns a success code.

(module
  ;; Define the greet function that returns i32
  ;; Returns 42 as a success indicator
  (func $greet (export "greet") (result i32)
    ;; Push the constant 42 onto the stack
    i32.const 42
  )
)

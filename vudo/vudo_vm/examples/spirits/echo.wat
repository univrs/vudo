;; Echo Spirit - Simple input/output module
;;
;; Demonstrates parameter passing and basic arithmetic.
;; Provides echo and double operations.

(module
  ;; Echo function - returns the input value unchanged
  ;; This demonstrates basic parameter passing
  (func $echo (export "echo") (param $input i32) (result i32)
    local.get $input
  )

  ;; Double function - returns input * 2
  ;; Demonstrates arithmetic operations
  (func $double (export "double") (param $input i32) (result i32)
    local.get $input
    i32.const 2
    i32.mul
  )

  ;; Triple function - returns input * 3
  ;; Additional arithmetic demonstration
  (func $triple (export "triple") (param $input i32) (result i32)
    local.get $input
    i32.const 3
    i32.mul
  )

  ;; Add two numbers
  ;; Demonstrates multiple parameters
  (func $add (export "add") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )
)

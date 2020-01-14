(module
  (global $zero (import "test" "zero") i32)
  (memory 2)
  (table $tab 2 funcref)
  (data (i32.const 0) "test")
  (data (i32.const 65534) "span")
  (elem (i32.const 0) $fib)
  (global $fib7 (mut i32) (global.get $zero))
  (global $one i32 (i32.const 1))

  (type $fib_type (func (param $f i32) (result i32)))

  (func $fib (param $f i32) (result i32)
    (if (result i32)
        (i32.lt_s
            (local.get $f)
            (i32.const 2)
        )
        (then local.get $f)
        (else 
          (i32.add 
            (call $fib (i32.sub (local.get $f) (i32.const 1)))
            (call_indirect (type $fib_type) (i32.sub (local.get $f) (i32.const 2)) (i32.const 0))
          )
        )
    )
  )

  (func $init_fib7
    (global.set $fib7
      (call $fib (i32.const 7))
    )
  )

  (start $init_fib7)

  (export "fib" (func $fib))
  (export "fib7" (global $fib7))
  (export "zero" (global $zero))
  (export "one" (global $one))
)

(module
  (memory 2)
  (table 2 funcref)
  (data (i32.const 0) "test")
  (data (i32.const 65534) "span")
  (elem (i32.const 0) $fib)

  (func $fib (param $f i32) (result i32)
    (if (result i32)
        (i32.lt_s
            (local.get $f)
            (i32.const 2)
        )
        (then local.get $f)
        (else (i32.add (call $fib (i32.sub (local.get $f) (i32.const 1))) (call $fib (i32.sub (local.get $f) (i32.const 2)))))
    )
  )
  (export "fib" (func $fib))
)

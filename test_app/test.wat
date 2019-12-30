(module
  (func $fib (param $f i32) (result i32)
    (if (result i32)
        (i32.lt_s
            (get_local $f)
            (i32.const 2)
        )
        (then get_local $f)
        (else (i32.add (call $fib (i32.sub (get_local $f) (i32.const 1))) (call $fib (i32.sub (get_local $f) (i32.const 2)))))
    )
  )
  (export "fib" (func $fib))
)

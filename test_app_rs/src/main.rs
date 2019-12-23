fn test_fib(n: i32) -> i32 {
    // This is deliberately rubbish
    if n < 2 {
        return 1;
    } else {
        return test_fib(n - 1) + test_fib(n - 2);
    }
}

fn main() {
    for a in 0..17 {
        println!("bogan {:?}", test_fib(a));
    }
}

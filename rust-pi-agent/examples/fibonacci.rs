fn fibonacci(n: u32) -> u64 {
    if n == 0 {
        return 0;
    }

    if n == 1 {
        return 1;
    }

    fibonacci(n - 1) + fibonacci(n - 2)
}

fn main() {
    println!("{}", fibonacci(5));
}

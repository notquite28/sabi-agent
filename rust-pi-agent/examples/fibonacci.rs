fn fibonacci(n: u32) -> u64 {
    if n == 0 {
        return 0;
    }

    let mut previous: u64 = 0;
    let mut current: u64 = 1;

    for _ in 1..n {
        let next = previous + current;
        previous = current;
        current = next;
    }

    current
}

fn main() {
    println!("{}", fibonacci(10));
}

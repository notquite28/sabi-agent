#!/usr/bin/env python3
"""Print a Fibonacci number using a simple recursive implementation."""


def fibonacci(n: int) -> int:
    """Return the nth Fibonacci number."""
    if n < 0:
        raise ValueError("n must be non-negative")
    if n == 0:
        return 0
    if n == 1:
        return 1
    return fibonacci(n - 1) + fibonacci(n - 2)


if __name__ == "__main__":
    print(fibonacci(5))

#!/usr/bin/env python3
"""Binary search example in Python."""

from collections.abc import Sequence


def binary_search(items: Sequence[int], target: int) -> int:
    """Return the index of target in sorted items, or -1 if not found."""
    left = 0
    right = len(items) - 1

    while left <= right:
        mid = left + (right - left) // 2

        if items[mid] == target:
            return mid
        if items[mid] < target:
            left = mid + 1
        else:
            right = mid - 1

    return -1


if __name__ == "__main__":
    numbers = [1, 3, 5, 7, 9, 11, 13]
    target = 7
    print(binary_search(numbers, target))

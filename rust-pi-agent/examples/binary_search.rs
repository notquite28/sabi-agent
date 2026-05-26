fn binary_search(items: &[i32], target: i32) -> Option<usize> {
    let mut left = 0;
    let mut right = items.len();

    while left < right {
        let mid = left + (right - left) / 2;

        if items[mid] == target {
            return Some(mid);
        }

        if items[mid] < target {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    None
}

fn main() {
    let numbers = [1, 3, 5, 7, 9, 11, 13];
    let target = 7;

    match binary_search(&numbers, target) {
        Some(index) => println!("{}", index),
        None => println!("not found"),
    }
}

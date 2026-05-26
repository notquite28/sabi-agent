function binarySearch(items: number[], target: number): number {
  let left = 0;
  let right = items.length - 1;

  while (left <= right) {
    const mid = left + Math.floor((right - left) / 2);

    if (items[mid] === target) {
      return mid;
    }

    if (items[mid] < target) {
      left = mid + 1;
    } else {
      right = mid - 1;
    }
  }

  return -1;
}

const numbers = [1, 3, 5, 7, 9, 11, 13];
const target = 7;
console.log(binarySearch(numbers, target));

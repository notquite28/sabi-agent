public class BinarySearch {
    public static int binarySearch(int[] items, int target) {
        int left = 0;
        int right = items.length - 1;

        while (left <= right) {
            int mid = left + (right - left) / 2;

            if (items[mid] == target) {
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

    public static void main(String[] args) {
        int[] numbers = {1, 3, 5, 7, 9, 11, 13};
        int target = 7;

        System.out.println(binarySearch(numbers, target));
    }
}

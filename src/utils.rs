unsafe fn find_if_ne<T: PartialEq>(first_target: *mut T, last: *mut T) -> *mut T {
    let mut first = first_target;
    while first != last {
        if first != first_target {
            return first;
        }
        first = first.add(1);
    }
    return last;
}
unsafe fn partition<T: PartialEq>(first_target: *mut T, last: *mut T) -> *mut T {
    let mut first = find_if_ne(first_target, last);
    if first == last {
        return first;
    }

    let mut i = first.add(1);
    while i != last {
        if *first_target == *i {
            std::ptr::swap(first, i);
            first = first.add(1);
        }
        i = i.add(1);
    }
    first
}
pub fn cluster<T: PartialEq>(slice: &mut [T]) {
    let len = slice.len();
    if len <= 1 {
        return;
    }

    unsafe {
        let mut it = slice.as_mut_ptr();
        let last = it.add(len);
        while it != last {
            it = partition(it, last);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition() {
        let mut slice: [i32; 6] = [1, 0, 3, 4, 3, 5];
        for i in 0..slice.len() {
            unsafe {
                let start = slice.as_mut_ptr().add(i);
                let end = slice.as_mut_ptr().add(slice.len());
                let it = partition(start, end);
                assert_ne!(
                    start,
                    it,
                    "slice {:?}, index {}",
                    slice,
                    it.offset_from(start) as usize + i
                );
            }
        }
    }

    #[test]
    fn test_cluster() {
        // Test case 1: Empty slice
        let mut slice: [i32; 0] = [];
        cluster(&mut slice);
        assert_eq!(slice, []);

        // Test case 2: Slice with one element
        let mut slice = [1];
        cluster(&mut slice);
        assert_eq!(slice, [1]);

        // Test case 3: Slice with multiple elements, no duplicates
        let mut slice = [1, 2, 3, 4, 5];
        cluster(&mut slice);
        assert_eq!(slice, [1, 2, 3, 4, 5]);

        // Test case 4: Slice with multiple elements, duplicates
        let mut slice = [1, 2, 3, 4, 2, 3, 4, 4, 4, 3];
        cluster(&mut slice);
        assert_eq!(slice, [1, 2, 2, 4, 4, 4, 4, 3, 3, 3]);

        // Test case 5: Slice with multiple elements, all duplicates
        let mut slice = [1, 1, 1, 1, 1];
        cluster(&mut slice);
        assert_eq!(slice, [1, 1, 1, 1, 1]);
    }
}

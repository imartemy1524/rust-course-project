#[inline]
pub(crate) fn mergesort<T: Ord + Clone>(array: &mut [T]) {
    let mut vec: Vec<T> = Vec::with_capacity(array.len());
    mergesort_local(array, &mut vec);
}
fn mergesort_local<T: Ord + Clone>(array: &mut [T], new_array: &mut Vec<T>) {
    let length = array.len();
    if length <= 1 {
        return;
    }
    if length == 2 {
        if array[0] > array[1] {
            array.swap(0, 1);
        }
        return;
    }

    let mid = length / 2;
    mergesort_local(&mut array[0..mid], new_array);
    mergesort_local(&mut array[mid..length], new_array);
    new_array.clear();

    let mut pointer_a = 0;
    let mut pointer_b = mid;
    while pointer_b < length && pointer_a < mid {
        if array[pointer_a] > array[pointer_b] {
            new_array.push(array[pointer_b].clone());
            pointer_b += 1;
        } else {
            new_array.push(array[pointer_a].clone());
            pointer_a += 1;
        }
    }
    while pointer_a < mid {
        new_array.push(array[pointer_a].clone());
        pointer_a += 1;
    }
    while pointer_b < length {
        new_array.push(array[pointer_b].clone());
        pointer_b += 1;
    }
    for i in 0..length {
        let g = new_array[i].clone();
        array[i] = g;
    }
}

pub fn allocate<T: Copy>(value: &T) -> *mut T {
    let box_ptr = Box::new(*value);
    Box::into_raw(box_ptr)
}
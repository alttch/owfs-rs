use std::os::raw::c_char;

extern "C" {
    pub(crate) fn OW_init(params: *const c_char) -> isize;
}

extern "C" {
    pub(crate) fn OW_finish();
}

extern "C" {
    pub(crate) fn OW_get(
        path: *const c_char,
        buffer: *const *mut c_char,
        buffer_length: *mut usize,
    ) -> isize;
}

extern "C" {
    pub(crate) fn OW_put(path: *const c_char, buffer: *const c_char, buffer_length: usize)
        -> isize;
}

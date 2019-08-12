pub fn u32_to_be_bytes(num: u32) -> [u8; 4] {
    #[allow(unsafe_code)]
    unsafe {
        std::mem::transmute::<u32, [u8; 4]>(num.to_be())
    }
}
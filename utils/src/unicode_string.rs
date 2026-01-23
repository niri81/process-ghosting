use ntapi::winapi::shared::ntdef::UNICODE_STRING;

#[must_use]
pub fn to_unicode_string(s: &str) -> UNICODE_STRING {
    let mut utf16: Vec<u16> = s.encode_utf16().collect();
    let len = u16::try_from(utf16.len() * 2)
        .expect("Expected to be able to convert unicode string length to u16");
    utf16.push(0); // Null terminator

    UNICODE_STRING {
        Length: len,
        MaximumLength: len + 2,
        Buffer: utf16.as_mut_ptr(),
    }
}

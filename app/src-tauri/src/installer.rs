const CHINESE_LANGUAGE: &str = "zh";
const ENGLISH_LANGUAGE: &str = "en";

pub fn selected_language() -> Option<&'static str> {
    selected_language_value()
        .as_deref()
        .and_then(normalize_language)
}

fn normalize_language(value: &str) -> Option<&'static str> {
    match value.trim() {
        CHINESE_LANGUAGE => Some(CHINESE_LANGUAGE),
        ENGLISH_LANGUAGE => Some(ENGLISH_LANGUAGE),
        _ => None,
    }
}

#[cfg(windows)]
fn selected_language_value() -> Option<String> {
    use windows::{
        core::w,
        Win32::{
            Foundation::ERROR_SUCCESS,
            System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_SZ},
        },
    };

    let mut byte_len = 0_u32;
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!(r"Software\NeoPad\NeoPad"),
            w!("InstallLanguage"),
            RRF_RT_REG_SZ,
            None,
            None,
            Some(&mut byte_len),
        )
    };
    if status != ERROR_SUCCESS || byte_len < 2 {
        return None;
    }

    let mut buffer = vec![0_u16; byte_len.div_ceil(2) as usize];
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!(r"Software\NeoPad\NeoPad"),
            w!("InstallLanguage"),
            RRF_RT_REG_SZ,
            None,
            Some(buffer.as_mut_ptr().cast()),
            Some(&mut byte_len),
        )
    };
    if status != ERROR_SUCCESS {
        return None;
    }

    let value_len = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());
    String::from_utf16(&buffer[..value_len]).ok()
}

#[cfg(not(windows))]
fn selected_language_value() -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::normalize_language;

    #[test]
    fn installer_language_accepts_only_supported_values() {
        assert_eq!(normalize_language("zh"), Some("zh"));
        assert_eq!(normalize_language(" en "), Some("en"));
        assert_eq!(normalize_language("fr"), None);
        assert_eq!(normalize_language(""), None);
    }
}

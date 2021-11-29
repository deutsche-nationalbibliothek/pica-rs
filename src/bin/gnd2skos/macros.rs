#[macro_export]
macro_rules! push_value {
    ($label:expr, $field:expr, $prefix:expr, $suffix:expr) => {
        if let Some(value) = $field {
            $label.push_str($prefix);
            // SAFETY: It's not necessary, because we are working on
            // `StringRecord`s.
            $label.push_str(unsafe { value.to_str_unchecked() });
            $label.push_str($suffix);
        }
    };
    ($label:expr, $field:expr, $prefix:expr) => {
        if let Some(value) = $field {
            $label.push_str($prefix);
            // SAFETY: It's not necessary, because we are working on
            // `StringRecord`s.
            $label.push_str(unsafe { value.to_str_unchecked() });
        }
    };
    ($label:expr, $field:expr) => {
        if let Some(value) = $field {
            // SAFETY: It's not necessary, because we are working on
            // `StringRecord`s.
            $label.push_str(unsafe { value.to_str_unchecked() });
        }
    };
}

#[macro_export]
macro_rules! push_list {
    ($label:expr, $values:expr, $sep:expr, $prefix:expr, $suffix:expr) => {
        if !$values.is_empty() {
            $label.push_str($prefix);
            // SAFETY: It's not necessary, because we are working on
            // `StringRecord`s.
            $label.push_str(unsafe { bstr::join($sep, $values).to_str_unchecked() });
            $label.push_str($suffix);
        }
    };
    ($label:expr, $values:expr, $sep:expr, $prefix:expr) => {
        if !$values.is_empty() {
            $label.push_str($prefix);
            // SAFETY: It's not necessary, because we are working on
            // `StringRecord`s.
            $label.push_str(unsafe { bstr::join($sep, $values).to_str_unchecked() });
        }
    };
}

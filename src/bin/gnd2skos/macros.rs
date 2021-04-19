#[macro_export]
macro_rules! push_value {
    ($label:expr, $field:expr, $prefix:expr, $suffix:expr) => {
        if let Some(value) = $field {
            $label.push_str($prefix);
            $label.push_str(value.to_str().unwrap());
            $label.push_str($suffix);
        }
    };
    ($label:expr, $field:expr, $prefix:expr) => {
        if let Some(value) = $field {
            $label.push_str($prefix);
            $label.push_str(value.to_str().unwrap());
        }
    };
    ($label:expr, $field:expr) => {
        if let Some(value) = $field {
            $label.push_str(value.to_str().unwrap());
        }
    };
}

#[macro_export]
macro_rules! push_list {
    ($label:expr, $values:expr, $sep:expr, $prefix:expr, $suffix:expr) => {
        if !$values.is_empty() {
            $label.push_str($prefix);
            $label.push_str(bstr::join($sep, $values).to_str().unwrap());
            $label.push_str($suffix);
        }
    };
    ($label:expr, $values:expr, $sep:expr, $prefix:expr) => {
        if !$values.is_empty() {
            $label.push_str($prefix);
            $label.push_str(bstr::join($sep, $values).to_str().unwrap());
        }
    };
}

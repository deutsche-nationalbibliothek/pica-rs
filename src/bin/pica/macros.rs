#[macro_export]
macro_rules! skip_invalid_flag {
    ($skip_invalid: expr, $local:expr, $global:expr) => {
        if $skip_invalid {
            true
        } else if let Some(ref config) = $local {
            config.skip_invalid.unwrap_or_default()
        } else if let Some(ref config) = $global {
            config.skip_invalid.unwrap_or_default()
        } else {
            false
        }
    };
}

#[macro_export]
macro_rules! gzip_flag {
    ($gzip: expr, $local:expr) => {
        if $gzip {
            true
        } else if let Some(ref config) = $local {
            config.gzip.unwrap_or_default()
        } else {
            false
        }
    };
}

#[macro_export]
macro_rules! template_opt {
    ($args: expr, $local:expr, $default:expr) => {
        if $args.is_present("template") {
            $args.value_of("template").unwrap().to_string()
        } else if let Some(ref config) = $local {
            config
                .template
                .as_ref()
                .map(|x| x.to_owned())
                .unwrap_or($default.to_string())
        } else {
            $default.to_string()
        }
    };
}

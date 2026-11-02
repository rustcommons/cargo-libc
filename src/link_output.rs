//! Final-link output classification, independent of Cargo's subcommand or file
//! suffix. Intermediate libraries can feed executables and DSOs in one build.
use std::ffi::{OsStr, OsString};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Executable,
    Shared,
    Relocatable,
}

/// Read driver and forwarded linker controls without altering caller arguments.
/// This is output selection only, not proof of safe cross-DSO unwinding.
pub(crate) fn classify(args: &[OsString]) -> Result<Kind, String> {
    let mut kind = Kind::Executable;
    let mut selected = false;
    let mut driver_value = false;
    let mut linker_value = false;
    let mut forwarded = false;
    let mut operands_only = false;
    for arg in args {
        if driver_value {
            driver_value = false;
        } else if forwarded {
            operand(arg, &mut linker_value, &mut kind, &mut selected)?;
            forwarded = false;
        } else if operands_only {
            continue;
        } else if arg == "--" {
            operands_only = true;
        } else if arg == "-Xlinker" {
            forwarded = true;
        } else if arg.as_encoded_bytes().starts_with(b"-Wl,") {
            for value in crate::clock::split_forwarded(arg)? {
                operand(&value, &mut linker_value, &mut kind, &mut selected)?;
            }
        } else if arg
            .to_str()
            .is_some_and(|s| crate::cc::SEPARATE_VALUE.contains(&s))
        {
            driver_value = true;
        } else if matches!(arg.to_str(), Some("-shared" | "-r")) {
            // A driver mode is not an operand of a preceding forwarded linker
            // option: Clang assembles the completed command separately.
            let mut driver_mode_value = false;
            operand(arg, &mut driver_mode_value, &mut kind, &mut selected)?;
        } else {
            operand(arg, &mut linker_value, &mut kind, &mut selected)?;
        }
    }
    if driver_value || linker_value || forwarded {
        return Err("missing driver/linker operand while classifying final output".into());
    }
    Ok(kind)
}

fn operand(
    arg: &OsStr,
    value: &mut bool,
    kind: &mut Kind,
    selected: &mut bool,
) -> Result<(), String> {
    if *value {
        *value = false;
        return Ok(());
    }
    let bytes = arg.as_encoded_bytes();
    let choice = match bytes {
        b"-shared" | b"--shared" | b"-Bshareable" => Some(Kind::Shared),
        b"-r" | b"--relocatable" | b"-Ur" => Some(Kind::Relocatable),
        _ => None,
    };
    if let Some(choice) = choice {
        if *selected && *kind != choice {
            return Err("conflicting shared and relocatable final-output controls".into());
        }
        *kind = choice;
        *selected = true;
    } else if crate::clock_named::takes_value(bytes) {
        *value = true;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[test]
    fn final_mode_comes_from_controls_not_suffix_or_cargo_command() {
        for values in [
            vec!["test-harness.o", "-pie", "-o", "test.so"],
            vec!["doctest.o", "-o", "rust_out"],
            vec!["cmake-probe.o", "-o", "cmTC_123"],
            vec![
                "app.o",
                "-Wl,--push-state,--no-as-needed",
                "-lgcc_s",
                "-Wl,--pop-state",
            ],
        ] {
            assert_eq!(classify(&args(&values)).unwrap(), Kind::Executable);
        }
        for values in [
            vec!["-Wl,-u", "-shared", "symbol", "library.o"],
            vec!["rust-cdylib.o", "-shared", "-o", "module"],
            vec!["cmake.o", "-Wl,-shared,-soname,libc.so", "-o", "library"],
            vec!["library.o", "-Xlinker", "--shared", "-o", "output"],
            vec!["library.o", "-Wl,-Bshareable"],
        ] {
            assert_eq!(classify(&args(&values)).unwrap(), Kind::Shared);
        }
        for values in [
            vec!["part.o", "-r"],
            vec!["part.o", "-Wl,--relocatable"],
            vec!["part.o", "-Xlinker", "-r"],
        ] {
            assert_eq!(classify(&args(&values)).unwrap(), Kind::Relocatable);
        }
    }

    #[test]
    fn output_names_and_forwarded_option_values_do_not_select_a_mode() {
        for values in [
            vec!["a.o", "-o", "-shared"],
            vec!["a.o", "-L", "-shared"],
            vec!["a.o", "-Wl,-o,-shared"],
            vec!["a.o", "-Wl,-soname", "-Xlinker", "-shared"],
            vec!["a.o", "-Xlinker", "-u", "-Xlinker", "-r"],
            vec!["a.o", "--", "-shared"],
        ] {
            assert_eq!(classify(&args(&values)).unwrap(), Kind::Executable);
        }
        assert_eq!(
            classify(&args(&["a.o", "-Wl,-soname,-shared", "-shared"])).unwrap(),
            Kind::Shared
        );
    }

    #[test]
    fn ambiguous_or_incomplete_controls_refuse() {
        for values in [
            vec!["a.o", "-shared", "-Wl,-r"],
            vec!["a.o", "-Xlinker"],
            vec!["a.o", "-o"],
            vec!["a.o", "-Wl,-soname"],
        ] {
            assert!(classify(&args(&values)).is_err(), "{values:?}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn unrelated_non_utf8_inputs_keep_their_bytes() {
        use std::os::unix::ffi::OsStringExt;
        let args = vec![
            OsString::from_vec(b"source\xff.o".to_vec()),
            "-shared".into(),
        ];
        let before = args.clone();
        assert_eq!(classify(&args).unwrap(), Kind::Shared);
        assert_eq!(args, before);
    }
}

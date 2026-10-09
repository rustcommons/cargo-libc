use crate::{
    elf::{self, ElfInfo},
    version::Version,
};
use std::{fs::File, io::Read, path::Path};

const MAX_ELF_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Debug)]
pub struct Assessment {
    pub errors: Vec<String>,
    pub notes: Vec<String>,
    pub highest_required: Option<Version>,
}

pub fn read_elf(path: &Path) -> Result<ElfInfo, String> {
    let file = File::open(path).map_err(|e| format!("cannot open {}: {e}", path.display()))?;
    if !file.metadata().map_err(|e| e.to_string())?.is_file() {
        return Err("ELF input must be a regular file".into());
    }
    let mut bytes = Vec::new();
    file.take(MAX_ELF_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_ELF_BYTES {
        return Err("ELF input exceeds the prototype's 256 MiB limit".into());
    }
    elf::inspect(&bytes)
}

pub fn assess(info: &ElfInfo, ceiling: Option<Version>) -> Assessment {
    let mut result = Assessment {
        errors: Vec::new(),
        notes: Vec::new(),
        highest_required: None,
    };
    for need in &info.versions {
        let Some(suffix) = need.name.strip_prefix("GLIBC_") else {
            continue;
        };
        match suffix.parse::<Version>() {
            Ok(version) if !need.weak => {
                result.highest_required =
                    Some(result.highest_required.map_or(version, |v| v.max(version)));
                if ceiling.is_some_and(|limit| version > limit) {
                    result.errors.push(format!(
                        "{} requires {} above the requested ceiling {}",
                        need.library,
                        need.name,
                        ceiling.unwrap()
                    ));
                }
            }
            Ok(version) => {
                if ceiling.is_some_and(|limit| version > limit) {
                    result.notes.push(format!(
                        "{} has version-level WEAK requirement {}; reported separately",
                        need.library, need.name
                    ));
                }
            }
            Err(_) if need.weak => result.notes.push(format!(
                "{} has nonnumeric version-level WEAK requirement {}",
                need.library, need.name
            )),
            Err(_) if ceiling.is_some() => result.errors.push(format!(
                "{} requires {}; a numeric glibc ceiling cannot validate this requirement",
                need.library, need.name
            )),
            Err(_) => result.notes.push(format!(
                "{} has nonnumeric glibc requirement {}",
                need.library, need.name
            )),
        }
    }
    result
}

pub fn quote_json(value: &str) -> String {
    let mut result = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c <= '\u{1f}' => result.push_str(&format!("\\u{:04x}", c as u32)),
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

fn json_strings(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|s| quote_json(s))
            .collect::<Vec<_>>()
            .join(",")
    )
}

pub fn render(
    path: &Path,
    info: &ElfInfo,
    assessment: &Assessment,
    ceiling: Option<Version>,
    json: bool,
) -> String {
    let check = if ceiling.is_none() {
        "not_requested"
    } else if assessment.errors.is_empty() {
        "pass"
    } else {
        "fail"
    };
    if json {
        let versions = info
            .versions
            .iter()
            .map(|n| {
                format!(
                    "{{\"library\":{},\"name\":{},\"version_weak\":{}}}",
                    quote_json(&n.library),
                    quote_json(&n.name),
                    n.weak
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        return format!(
            concat!(
                "{{\"file\":{},\"machine\":{},\"interpreter\":{},",
                "\"needed\":{},\"rpath\":{},\"runpath\":{},\"version_requirements\":[{}],",
                "\"max_glibc\":{},\"highest_required_numeric_glibc\":{},",
                "\"direct_glibc_requirement_check\":{},\"errors\":{},\"notes\":{},",
                "\"transitive_dependencies_checked\":false,\"runtime_tested\":false}}"
            ),
            quote_json(&path.to_string_lossy()),
            info.machine,
            info.interpreter
                .as_ref()
                .map_or("null".into(), |s| quote_json(s)),
            json_strings(&info.needed),
            json_strings(&info.rpath),
            json_strings(&info.runpath),
            versions,
            ceiling.map_or("null".into(), |v| quote_json(&v.to_string())),
            assessment
                .highest_required
                .map_or("null".into(), |v| quote_json(&v.to_string())),
            quote_json(check),
            json_strings(&assessment.errors),
            json_strings(&assessment.notes)
        );
    }
    let mut lines = vec![
        format!("ELF: {}", path.display()),
        format!("Interpreter: {:?}", info.interpreter),
        format!("Needed libraries: {:?}", info.needed),
        format!("RPATH: {:?}", info.rpath),
        format!("RUNPATH: {:?}", info.runpath),
        "Version requirements:".into(),
    ];
    for need in &info.versions {
        lines.push(format!(
            "  {:?}: {:?} [{}]",
            need.library,
            need.name,
            if need.weak {
                "version-level WEAK"
            } else {
                "required"
            }
        ));
    }
    lines.push(format!(
        "Highest required numeric GLIBC version: {}",
        assessment
            .highest_required
            .map_or("none".into(), |v| v.to_string())
    ));
    lines.push(format!("Direct glibc requirement check: {check}"));
    for error in &assessment.errors {
        lines.push(format!("  FAIL: {error}"));
    }
    for note in &assessment.notes {
        lines.push(format!("  NOTE: {note}"));
    }
    lines.push("Scope: direct ELF version requirements only. Dependent libraries, CPU/kernel requirements, dynamically loaded plugins, and runtime behavior are not checked.".into());
    lines.join("\n")
}

pub fn inspect_file(path: &Path, ceiling: Option<Version>, json: bool) -> Result<i32, String> {
    let info = read_elf(path)?;
    let assessment = assess(&info, ceiling);
    println!("{}", render(path, &info, &assessment, ceiling, json));
    Ok(if assessment.errors.is_empty() { 0 } else { 2 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elf::VersionNeed;

    fn fixture(needs: &[(&str, bool)]) -> ElfInfo {
        ElfInfo {
            machine: 62,
            interpreter: None,
            needed: vec!["libc.so.6".into()],
            rpath: vec![],
            runpath: vec![],
            versions: needs
                .iter()
                .map(|(name, weak)| VersionNeed {
                    library: "libc.so.6".into(),
                    name: name.to_string(),
                    weak: *weak,
                })
                .collect(),
        }
    }

    #[test]
    fn newer_weak_requirement_does_not_raise_required_floor() {
        let result = assess(
            &fixture(&[("GLIBC_2.17", false), ("GLIBC_2.34", true)]),
            Some("2.22".parse().unwrap()),
        );
        assert!(result.errors.is_empty());
        assert_eq!(result.highest_required, Some("2.17".parse().unwrap()));
        assert_eq!(result.notes.len(), 1);
    }

    #[test]
    fn rejects_newer_required_and_nonnumeric_abi_requirements() {
        for name in ["GLIBC_2.34", "GLIBC_PRIVATE", "GLIBC_ABI_DT_RELR"] {
            let result = assess(&fixture(&[(name, false)]), Some("2.22".parse().unwrap()));
            assert_eq!(result.errors.len(), 1, "{name}");
        }
    }

    #[test]
    fn does_not_treat_libstdcxx_versions_as_glibc_versions() {
        let result = assess(
            &fixture(&[("GLIBCXX_3.4.30", false), ("GCC_4.2.0", false)]),
            Some("2.22".parse().unwrap()),
        );
        assert!(result.errors.is_empty());
        assert!(result.highest_required.is_none());
    }

    #[test]
    fn escapes_json_strings() {
        assert_eq!(quote_json("a\"b\\c\n\u{0}"), "\"a\\\"b\\\\c\\n\\u0000\"");
    }
}

use crate::{
    elf::{self, ElfInfo},
    version::Version,
};
use linux_abi_scan::{Report, ScanTarget, Severity};
use linux_targets::{Request, Selection};
use std::{fs::File, io::Read, path::Path};

const MAX_ELF_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Debug)]
pub struct Assessment {
    pub errors: Vec<String>,
    pub notes: Vec<String>,
    pub highest_required: Option<Version>,
}

/// The tier 1 scan requested on `inspect` (`--distro`, `--kernel`, `--scan`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScanOptions {
    pub distro: Option<String>,
    pub kernel: Option<String>,
    /// Warnings fail too.
    pub strict: bool,
    /// Show informational findings in the text report.
    pub verbose: bool,
}

/// A finished scan and the options it ran with.
pub struct Scan<'a> {
    pub options: &'a ScanOptions,
    pub report: Report,
}

impl Scan<'_> {
    pub fn passed(&self) -> bool {
        self.report.exit_code(self.options.strict) == 0
    }
}

pub fn read_elf(path: &Path) -> Result<ElfInfo, String> {
    elf::inspect(&read_bytes(path)?)
}

pub fn read_bytes(path: &Path) -> Result<Vec<u8>, String> {
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
    Ok(bytes)
}

/// Resolves the scan's floors with `linux_targets` (so a preset means the
/// same here as in the seccomp simulation) and scans `bytes`. Returns the
/// glibc ceiling for the direct check, which a `--distro` preset sets.
/// Resolves the scan's floors with `linux_targets`, so a preset means the
/// same here as in the seccomp simulation. Returns the glibc ceiling for the
/// direct check: a `--distro` preset sets it, otherwise `--max-glibc` stays.
pub fn floors(
    ceiling: Option<Version>,
    options: &ScanOptions,
) -> Result<(Option<Version>, ScanTarget), String> {
    let glibc = ceiling.map(|v| v.to_string());
    let selection = Selection::resolve(&Request {
        distro: options.distro.as_deref(),
        glibc: glibc.as_deref(),
        kernel: options.kernel.as_deref(),
        target: None,
    })?;
    let ceiling = match (&selection.distro, selection.glibc) {
        (Some(_), Some(floor)) => Some(
            floor
                .to_string()
                .parse::<Version>()?
                .validate_glibc_target()?,
        ),
        _ => ceiling,
    };
    Ok((ceiling, ScanTarget::from_selection(&selection)))
}

/// Runs the tier 1 scan on `bytes`; see [`floors`].
pub fn scan<'a>(
    bytes: &[u8],
    ceiling: Option<Version>,
    options: &'a ScanOptions,
) -> Result<(Option<Version>, Scan<'a>), String> {
    let (ceiling, target) = floors(ceiling, options)?;
    let report = linux_abi_scan::scan(bytes, &target).map_err(|e| format!("tier 1 scan: {e}"))?;
    Ok((ceiling, Scan { options, report }))
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

fn scan_json(scan: &Scan) -> String {
    let findings = scan
        .report
        .findings
        .iter()
        .map(|f| {
            format!(
                concat!(
                    "{{\"severity\":{},\"confidence\":{},\"check\":{},",
                    "\"subject\":{},\"address\":{},\"detail\":{}}}"
                ),
                quote_json(f.severity.as_str()),
                quote_json(f.confidence.as_str()),
                quote_json(f.check),
                quote_json(&f.subject),
                f.address
                    .map_or("null".into(), |a| quote_json(&format!("{a:#x}"))),
                quote_json(&f.detail)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{\"tier\":1,\"target\":{},\"strict\":{},\"result\":{},",
            "\"errors\":{},\"warnings\":{},\"findings\":[{}]}}"
        ),
        quote_json(&scan.report.target),
        scan.options.strict,
        quote_json(if scan.passed() { "pass" } else { "fail" }),
        scan.report.count(Severity::Error),
        scan.report.count(Severity::Warning),
        findings
    )
}

pub fn render(
    path: &Path,
    info: &ElfInfo,
    assessment: &Assessment,
    ceiling: Option<Version>,
    scan: Option<&Scan>,
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
                "\"scan\":{},",
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
            json_strings(&assessment.notes),
            scan.map_or("null".into(), scan_json)
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
    match scan {
        None => lines.push("Scope: direct ELF version requirements only. Dependent libraries, CPU/kernel requirements, dynamically loaded plugins, and runtime behavior are not checked. For the tier 1 kernel scan, run cargo libc inspect with --distro or --kernel.".into()),
        Some(scan) => {
            lines.push(format!(
                "Tier 1 scan (linux-abi-scan{}): {}",
                if scan.options.strict { ", strict" } else { "" },
                if scan.passed() { "pass" } else { "fail" }
            ));
            lines.extend(
                scan.report
                    .to_text(scan.options.verbose)
                    .lines()
                    .map(|line| format!("  {line}")),
            );
            lines.push("Scope: this file only. Dependent libraries, dynamically loaded plugins, syscalls made inside glibc, and runtime behavior are not checked.".into());
        }
    }
    lines.join("\n")
}

pub fn inspect_file(
    path: &Path,
    ceiling: Option<Version>,
    scan_options: Option<&ScanOptions>,
    json: bool,
) -> Result<i32, String> {
    let bytes = read_bytes(path)?;
    let info = elf::inspect(&bytes)?;
    let (ceiling, scan) = match scan_options {
        Some(options) => {
            let (ceiling, scan) = scan(&bytes, ceiling, options)?;
            (ceiling, Some(scan))
        }
        None => (ceiling, None),
    };
    let assessment = assess(&info, ceiling);
    println!(
        "{}",
        render(path, &info, &assessment, ceiling, scan.as_ref(), json)
    );
    let passed = assessment.errors.is_empty() && scan.as_ref().is_none_or(Scan::passed);
    Ok(if passed { 0 } else { 2 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::elf::VersionNeed;
    use linux_abi_scan::{Confidence, Finding};

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
    fn scan_floors_come_from_the_preset_or_the_flags() {
        let sles = ScanOptions {
            distro: Some("sles-12-sp5".into()),
            ..Default::default()
        };
        let (ceiling, target) = floors(None, &sles).unwrap();
        assert_eq!(ceiling, Some("2.22".parse().unwrap()));
        assert_eq!(target.glibc.unwrap().to_string(), "2.22");
        assert!(target.kernel.is_some());
        assert!(target.label.contains("SUSE"));

        let kernel_only = ScanOptions {
            kernel: Some("3.10".into()),
            ..Default::default()
        };
        let patch = "2.17.1".parse().unwrap();
        let (ceiling, target) = floors(Some(patch), &kernel_only).unwrap();
        assert_eq!(ceiling, Some(patch), "--max-glibc is kept as given");
        assert!(target.kernel.is_some());

        let (_, target) = floors(Some(patch), &ScanOptions::default()).unwrap();
        assert!(
            target.kernel.is_none(),
            "--scan alone does not simulate a kernel"
        );

        let unknown = ScanOptions {
            distro: Some("no-such-distro".into()),
            ..Default::default()
        };
        assert!(floors(None, &unknown).is_err());
    }

    #[test]
    fn scan_results_render_in_text_and_json() {
        let options = ScanOptions {
            strict: true,
            ..Default::default()
        };
        let scan = Scan {
            options: &options,
            report: Report {
                target: "glibc 2.17, kernel 3.10".into(),
                findings: vec![Finding {
                    severity: Severity::Warning,
                    confidence: Confidence::Heuristic,
                    check: "syscall-call",
                    subject: "memfd_create".into(),
                    detail: "added in \"3.17\"".into(),
                    address: Some(0x1234),
                }],
            },
        };
        assert!(!scan.passed(), "strict makes a warning fail");
        let info = fixture(&[("GLIBC_2.17", false)]);
        let ceiling = Some("2.17".parse().unwrap());
        let assessment = assess(&info, ceiling);
        let path = Path::new("app");
        let json = render(path, &info, &assessment, ceiling, Some(&scan), true);
        assert!(json.contains(concat!(
            "\"scan\":{\"tier\":1,\"target\":\"glibc 2.17, kernel 3.10\",",
            "\"strict\":true,\"result\":\"fail\",\"errors\":0,\"warnings\":1,",
            "\"findings\":[{\"severity\":\"warning\",\"confidence\":\"heuristic\",",
            "\"check\":\"syscall-call\",\"subject\":\"memfd_create\",",
            "\"address\":\"0x1234\",\"detail\":\"added in \\\"3.17\\\"\"}]}"
        )));
        let text = render(path, &info, &assessment, ceiling, Some(&scan), false);
        assert!(text.contains("Tier 1 scan (linux-abi-scan, strict): fail"));
        assert!(text.contains("memfd_create at 0x1234"));
        let plain = render(path, &info, &assessment, ceiling, None, true);
        assert!(plain.contains("\"scan\":null"));
    }

    #[test]
    fn escapes_json_strings() {
        assert_eq!(quote_json("a\"b\\c\n\u{0}"), "\"a\\\"b\\\\c\\n\\u0000\"");
    }
}

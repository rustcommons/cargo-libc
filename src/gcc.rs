//! Static, two-file libgcc interface comparison. No loader or target execution.
use crate::{elf, report, version::Version};
use elf::exact::Symbol;
use std::{collections::BTreeMap, path::Path};

const OWNER: &str = "libgcc_s.so.1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    Match,
    Missing,
    Unresolved,
    OptionalAbsent,
    Excluded,
}
impl Status {
    fn name(self) -> &'static str {
        match self {
            Self::Match => "match",
            Self::Missing => "missing",
            Self::Unresolved => "unresolved",
            Self::OptionalAbsent => "optional_absent",
            Self::Excluded => "not_compared",
        }
    }
}

#[derive(Clone, Debug)]
struct Row {
    name: String,
    version: Option<String>,
    owner: Option<String>,
    required: bool,
    weak_version: bool,
    binding: Option<u8>,
    kind: Option<u8>,
    hidden: bool,
    status: Status,
    reason: &'static str,
    definition: Option<Symbol>,
}

pub(crate) struct Comparison {
    artifact_identity: String,
    target_identity: String,
    status: Status,
    rows: Vec<Row>,
    nodes: Vec<Row>,
    findings: Vec<String>,
    target_metadata: String,
    pub target_glibc_failed: bool,
    enumeration_complete: bool,
}

fn q(s: &str) -> String {
    report::quote_json(s)
}
fn opt(s: &Option<String>) -> String {
    s.as_ref().map_or("null".into(), |s| q(s))
}
fn binding(b: u8) -> &'static str {
    match b {
        0 => "LOCAL",
        1 => "GLOBAL",
        2 => "WEAK",
        10 => "GNU_UNIQUE",
        _ => "unsupported",
    }
}
fn kind(k: u8) -> &'static str {
    match k {
        0 => "NOTYPE",
        1 => "OBJECT",
        2 => "FUNC",
        6 => "TLS",
        10 => "IFUNC",
        _ => "unsupported",
    }
}
fn visibility(v: u8) -> &'static str {
    match v {
        0 => "DEFAULT",
        1 => "INTERNAL",
        2 => "HIDDEN",
        3 => "PROTECTED",
        _ => "unsupported",
    }
}

fn identity(path: &Path, bytes: &[u8]) -> String {
    let resolved = std::fs::canonicalize(path).ok();
    let locator = path.to_string_lossy();
    let resolved_display = resolved.as_ref().map(|p| p.to_string_lossy());
    let header = if bytes.len() >= 20 && &bytes[..4] == b"\x7fELF" {
        format!(
            "{{\"class\":{},\"data\":{},\"type_raw_le\":{},\"machine_raw_le\":{}}}",
            bytes[4],
            bytes[5],
            u16::from_le_bytes([bytes[16], bytes[17]]),
            u16::from_le_bytes([bytes[18], bytes[19]])
        )
    } else {
        "null".into()
    };
    format!("{{\"locator\":{},\"resolved_locator\":{},\"path_display_lossy\":{},\"sha256\":{},\"bytes\":{},\"elf_header\":{}}}", q(&locator), resolved_display.as_ref().map_or("null".into(), |s| q(s)), matches!(locator, std::borrow::Cow::Owned(_)) || resolved_display.is_some_and(|s| matches!(s, std::borrow::Cow::Owned(_))), q(&libc_abi_tools::content_sha256(bytes)), bytes.len(), header)
}

fn definition_json(s: &Symbol) -> String {
    format!("{{\"name\":{},\"version\":{},\"default_version\":{},\"binding\":{},\"visibility\":{},\"type\":{}}}", q(&s.name), opt(&s.version), !s.version_hidden, q(binding(s.binding)), q(visibility(s.visibility)), q(kind(s.kind)))
}

impl Row {
    fn json(&self) -> String {
        format!(concat!("{{\"name\":{},\"version\":{},\"recorded_version_owner\":{},\"required\":{},",
            "\"version_need_weak\":{},\"symbol_binding\":{},\"symbol_type\":{},\"versym_hidden\":{},",
            "\"status\":{},\"reason_code\":{},\"matching_definition\":{}}}"),
            q(&self.name), opt(&self.version), opt(&self.owner), self.required, self.weak_version,
            self.binding.map_or("null".into(), |b| q(binding(b))), self.kind.map_or("null".into(), |k| q(kind(k))), self.hidden,
            q(self.status.name()), q(self.reason), self.definition.as_ref().map_or("null".into(), definition_json))
    }
}

impl Comparison {
    fn finding(&mut self, code: &str, detail: &str) {
        self.findings
            .push(format!("{{\"code\":{},\"detail\":{}}}", q(code), q(detail)));
    }
    pub(crate) fn failed(&self) -> bool {
        self.status == Status::Missing || self.target_glibc_failed
    }
    pub(crate) fn unresolved(&self) -> bool {
        self.status == Status::Unresolved
    }
    pub(crate) fn json(&self) -> String {
        let join = |rows: Vec<&Row>| {
            rows.into_iter()
                .map(Row::json)
                .collect::<Vec<_>>()
                .join(",")
        };
        let counts = |rows: Vec<&Row>| {
            let count = |required, status| {
                rows.iter()
                    .filter(|r| r.required == required && r.status == status)
                    .count()
            };
            format!("{{\"required_matched\":{},\"required_missing\":{},\"required_unresolved\":{},\"optional_matched\":{},\"optional_absent\":{}}}", count(true, Status::Match), count(true, Status::Missing), count(true, Status::Unresolved), count(false, Status::Match), count(false, Status::OptionalAbsent))
        };
        format!(concat!("{{\"schema_version\":1,\"status\":{},\"scope\":\"direct_recorded_libgcc_requirements\",",
            "\"artifact_identity\":{},\"target_identity\":{},\"enumeration_complete\":{},",
            "\"version_requirements\":[{}],\"requirements\":[{}],\"other_owner_imports\":[{}],\"unowned_imports\":[{}],",
            "\"counts\":{{\"version_nodes\":{},\"symbol_pairs\":{}}},\"target_runtime_metadata\":{},\"findings\":[{}],",
            "\"all_application_imports_checked\":false,\"selected_by_target_loader_verified\":false,",
            "\"transitive_dependencies_checked\":false,\"plugins_checked\":false,\"runtime_tested\":false,",
            "\"cpu_kernel_compatibility_checked\":false,\"runtime_implementation_authenticated\":false}}"),
            q(self.status.name()), self.artifact_identity, self.target_identity, self.enumeration_complete,
            join(self.nodes.iter().collect()), join(self.rows.iter().filter(|r| r.owner.as_deref() == Some(OWNER)).collect()),
            join(self.rows.iter().filter(|r| r.owner.as_deref().is_some_and(|o| o != OWNER)).collect()),
            join(self.rows.iter().filter(|r| r.owner.is_none()).collect()),
            counts(self.nodes.iter().collect()), counts(self.rows.iter().filter(|r| r.owner.as_deref() == Some(OWNER)).collect()),
            self.target_metadata, self.findings.join(","))
    }
    pub(crate) fn text(&self) -> String {
        let matched = self
            .rows
            .iter()
            .filter(|r| {
                r.owner.as_deref() == Some(OWNER) && r.required && r.status == Status::Match
            })
            .count();
        let mut lines = vec![format!(
            "GCC ABI comparison: {} ({matched} mandatory symbol/version pairs matched)",
            self.status.name()
        )];
        for r in self.nodes.iter().chain(
            self.rows
                .iter()
                .filter(|r| r.owner.as_deref() == Some(OWNER) || r.status == Status::Unresolved),
        ) {
            lines.push(format!(
                "  {}: {}{} [{}; {}]",
                r.status.name(),
                q(&r.name),
                r.version
                    .as_ref()
                    .map_or(String::new(), |v| format!("@{}", q(v))),
                if r.required { "required" } else { "optional" },
                r.reason
            ));
        }
        for finding in &self.findings {
            lines.push(format!("  Finding: {finding}"));
        }
        lines.push(format!("  Artifact identity: {}", self.artifact_identity));
        lines.push(format!("  Target identity: {}", self.target_identity));
        lines.push(format!(
            "  Target runtime metadata (separate direct GLIBC assessment): {}",
            self.target_metadata
        ));
        lines.push("Scope: exact recorded direct interfaces in these two files only. Target loader selection, real runtime implementation, dependent libraries, plugins, CPU/kernel compatibility and execution are NOT verified. A match is not a safe-to-run certificate.".into());
        lines.join("\n")
    }
}

fn compare_symbols(
    result: &mut Comparison,
    info: &elf::ElfInfo,
    imports: &[Symbol],
    definitions: &[Symbol],
    target: &elf::ElfInfo,
) {
    for node in info.versions.iter().filter(|v| v.library == OWNER) {
        let found = target.definitions.contains(&node.name);
        result.nodes.push(Row {
            name: node.name.clone(),
            version: None,
            owner: Some(node.library.clone()),
            required: !node.weak,
            weak_version: node.weak,
            binding: None,
            kind: None,
            hidden: false,
            status: if found {
                Status::Match
            } else if node.weak {
                Status::OptionalAbsent
            } else {
                Status::Missing
            },
            reason: if found {
                "version_node_present"
            } else {
                "version_node_absent"
            },
            definition: None,
        });
    }
    let mut exports: BTreeMap<&str, Vec<&Symbol>> = BTreeMap::new();
    for s in definitions.iter().filter(|s| {
        s.defined
            && s.lookup_visible
            && (s.value != 0 || s.section == 0xfff1)
            && matches!(s.binding, 1 | 2 | 10)
            && matches!(s.visibility, 0 | 3)
            && s.owner.is_none()
    }) {
        exports.entry(&s.name).or_default().push(s);
    }
    for s in imports.iter().filter(|s| !s.defined || s.owner.is_some()) {
        if s.binding == 0 && s.owner.is_none() {
            continue;
        }
        let required = s.binding != 2;
        let mut row = Row {
            name: s.name.clone(),
            version: s.version.clone(),
            owner: s.owner.clone(),
            required,
            weak_version: s.version_weak,
            binding: Some(s.binding),
            kind: Some(s.kind),
            hidden: s.version_hidden,
            status: Status::Unresolved,
            reason: "unowned_import",
            definition: None,
        };
        match s.owner.as_deref() {
            Some(owner) if owner != OWNER => {
                row.status = Status::Excluded;
                row.reason = "other_recorded_owner";
            }
            None if !required => {
                row.status = Status::Excluded;
                row.reason = "unowned_weak_hook";
            }
            None => {}
            Some(_)
                if s.defined
                    || !matches!(s.binding, 1 | 2)
                    || !matches!(s.kind, 2 | 10)
                    || !matches!(s.visibility, 0 | 3) =>
            {
                row.reason = "unsupported_import_representation";
            }
            Some(_) => {
                let candidates = exports.get(s.name.as_str()).map_or(&[][..], Vec::as_slice);
                let exact: Vec<_> = candidates
                    .iter()
                    .copied()
                    .filter(|d| d.version == s.version)
                    .collect();
                let functions: Vec<_> = exact
                    .iter()
                    .copied()
                    .filter(|d| {
                        matches!(d.kind, 2 | 10)
                            && matches!(d.binding, 1 | 2)
                            && !matches!(d.section, 0xffff | 0xfff2)
                    })
                    .collect();
                if functions.len() == 1 && exact.len() == 1 {
                    row.status = Status::Match;
                    row.reason = "exact_pair_present";
                    row.definition = Some(functions[0].clone());
                } else if !exact.is_empty() {
                    row.reason = "unsupported_or_ambiguous_definition";
                } else if candidates.iter().any(|d| d.version.is_none()) {
                    row.reason = "unversioned_fallback_not_modeled";
                } else {
                    row.status = if required {
                        Status::Missing
                    } else {
                        Status::OptionalAbsent
                    };
                    row.reason = "exact_pair_absent";
                }
            }
        }
        result.rows.push(row);
    }
    let relevant = result.nodes.iter().chain(
        result
            .rows
            .iter()
            .filter(|r| r.owner.as_deref() == Some(OWNER)),
    );
    let relevant: Vec<_> = relevant.collect();
    result.status = if relevant
        .iter()
        .any(|r| r.required && r.status == Status::Missing)
    {
        Status::Missing
    } else if relevant.is_empty()
        || result
            .rows
            .iter()
            .any(|r| r.required && r.status == Status::Unresolved)
    {
        Status::Unresolved
    } else {
        Status::Match
    };
    if relevant.is_empty() {
        result.finding(
            "no_direct_attributed_requirements",
            "No recorded owner-attributed libgcc interface requirements; not a vacuous match",
        );
    }
}

pub(crate) fn compare(
    path: &Path,
    bytes: &[u8],
    info: &elf::ElfInfo,
    target_path: &Path,
    ceiling: Option<Version>,
) -> Comparison {
    let mut result = Comparison {
        artifact_identity: identity(path, bytes),
        target_identity: "null".into(),
        status: Status::Unresolved,
        rows: vec![],
        nodes: vec![],
        findings: vec![],
        target_metadata: "null".into(),
        target_glibc_failed: false,
        enumeration_complete: false,
    };
    let imports = elf::exact::inspect(bytes);
    let target_bytes = match report::read_bytes_limited(target_path, 64 * 1024 * 1024) {
        Ok(b) => b,
        Err(e) => {
            result.finding("target_unavailable", &e);
            return result;
        }
    };
    result.target_identity = identity(target_path, &target_bytes);
    // Known incompatible ELF identity is distinct from unreadable syntax.
    if target_bytes.len() >= 20
        && &target_bytes[..4] == b"\x7fELF"
        && (target_bytes[4] != 2
            || target_bytes[5] != 1
            || target_bytes[18..20] != 62_u16.to_le_bytes())
    {
        result.status = Status::Missing;
        result.finding(
            "architecture_mismatch",
            "Target must be ELF64 little-endian x86-64",
        );
        return result;
    }
    let target = match elf::inspect(&target_bytes) {
        Ok(t) => t,
        Err(e) => {
            result.finding("malformed_target", &e);
            return result;
        }
    };
    let assessment = report::assess(&target, ceiling);
    result.target_glibc_failed = !assessment.errors.is_empty();
    result.target_metadata = report::render(target_path, &target, &assessment, ceiling, None, true);
    if target.soname.as_deref() != Some(OWNER) || target_bytes[16..18] != 3_u16.to_le_bytes() {
        result.status = Status::Missing;
        result.finding(
            "soname_or_type_mismatch",
            "Target must be ET_DYN with exact SONAME libgcc_s.so.1",
        );
        return result;
    }
    let imports = match imports {
        Ok(s) => s,
        Err(e) => {
            result.finding("artifact_enumeration_unresolved", &e);
            return result;
        }
    };
    let definitions = match elf::exact::inspect(&target_bytes) {
        Ok(s) => s,
        Err(e) => {
            result.finding("target_enumeration_unresolved", &e);
            return result;
        }
    };
    if imports.len() + definitions.len() > 32_768 {
        result.finding("comparison_budget_exceeded", "Combined dynamic symbol inventory exceeds 32768 entries; no partial comparison reported");
        return result;
    }
    result.enumeration_complete = true;
    compare_symbols(&mut result, info, &imports, &definitions, &target);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use elf::{ElfInfo, VersionNeed};
    fn info() -> ElfInfo {
        ElfInfo {
            machine: 62,
            soname: Some(OWNER.into()),
            interpreter: None,
            needed: vec![OWNER.into()],
            rpath: vec![],
            runpath: vec![],
            versions: vec![VersionNeed {
                library: OWNER.into(),
                name: "GCC_4.2.0".into(),
                weak: false,
            }],
            definitions: vec!["GCC_4.2.0".into()],
        }
    }
    fn symbol(defined: bool) -> Symbol {
        Symbol {
            name: "_Unwind_GetIPInfo".into(),
            binding: 1,
            kind: 2,
            visibility: 0,
            defined,
            value: 4096,
            section: if defined { 1 } else { 0 },
            version_index: 2,
            version: Some("GCC_4.2.0".into()),
            owner: if defined { None } else { Some(OWNER.into()) },
            version_weak: false,
            version_hidden: false,
            lookup_visible: defined,
        }
    }
    fn run(
        a: &ElfInfo,
        imports: &[Symbol],
        definitions: &[Symbol],
        target: &ElfInfo,
    ) -> Comparison {
        let mut c = Comparison {
            artifact_identity: "null".into(),
            target_identity: "null".into(),
            status: Status::Unresolved,
            rows: vec![],
            nodes: vec![],
            findings: vec![],
            target_metadata: "null".into(),
            target_glibc_failed: false,
            enumeration_complete: true,
        };
        compare_symbols(&mut c, a, imports, definitions, target);
        c
    }
    #[test]
    fn exact_pair_and_node_match_without_requiring_default_version() {
        let mut def = symbol(true);
        def.version_hidden = true;
        let c = run(&info(), &[symbol(false)], &[def], &info());
        assert_eq!(c.status, Status::Match);
        assert!(c.json().contains("\"default_version\":false"));
    }
    #[test]
    fn mandatory_node_survives_weak_or_absent_symbol_references() {
        let mut target = info();
        target.definitions.clear();
        let mut weak = symbol(false);
        weak.binding = 2;
        for imports in [vec![weak], vec![]] {
            let c = run(&info(), &imports, &[], &target);
            assert_eq!(c.status, Status::Missing);
            assert_eq!(c.nodes[0].status, Status::Missing);
        }
        let c = run(&info(), &[], &[], &info());
        assert_eq!(c.status, Status::Match);
    }
    #[test]
    fn weak_symbol_absence_is_optional_but_weak_node_does_not_weaken_global_symbol() {
        let mut weak = symbol(false);
        weak.binding = 2;
        let c = run(&info(), &[weak], &[], &info());
        assert_eq!(c.status, Status::Match);
        assert_eq!(c.rows[0].status, Status::OptionalAbsent);
        let mut app = info();
        app.versions[0].weak = true;
        let mut strong = symbol(false);
        strong.version_weak = true;
        let c = run(&app, &[strong], &[], &info());
        assert_eq!(c.status, Status::Missing);
        assert!(c.rows[0].required);
    }
    #[test]
    fn name_node_and_version_each_matter() {
        for change in 0..3 {
            let mut def = symbol(true);
            match change {
                0 => def.name = "other".into(),
                1 => def.version = Some("GCC_99.0".into()),
                _ => def.lookup_visible = false,
            }
            assert_eq!(
                run(&info(), &[symbol(false)], &[def], &info()).status,
                Status::Missing
            );
        }
    }
    #[test]
    fn visibility_binding_type_and_unversioned_fallback_are_not_false_matches() {
        for change in 0..5 {
            let mut def = symbol(true);
            match change {
                0 => def.visibility = 2,
                1 => def.binding = 0,
                2 => def.kind = 1,
                3 => def.version = None,
                _ => def.owner = Some("other.so".into()),
            }
            let c = run(&info(), &[symbol(false)], &[def], &info());
            assert_ne!(c.status, Status::Match);
        }
        let mut def = symbol(true);
        def.binding = 2;
        def.visibility = 3;
        def.kind = 10;
        assert_eq!(
            run(&info(), &[symbol(false)], &[def], &info()).status,
            Status::Match
        );
    }
    #[test]
    fn owner_is_not_guessed_from_name_or_namespace() {
        let mut app = info();
        app.versions.clear();
        let mut import = symbol(false);
        import.owner = Some("libother.so".into());
        let c = run(&app, &[import], &[symbol(true)], &info());
        assert_eq!(c.status, Status::Unresolved);
        assert_eq!(c.rows[0].status, Status::Excluded);
    }
    #[test]
    fn unowned_strong_import_prevents_attribution_but_weak_hooks_are_excluded() {
        for binding in [1, 2] {
            let mut hook = symbol(false);
            hook.name = "unowned".into();
            hook.version = None;
            hook.owner = None;
            hook.binding = binding;
            let c = run(&info(), &[symbol(false), hook], &[symbol(true)], &info());
            assert_eq!(
                c.status,
                if binding == 1 {
                    Status::Unresolved
                } else {
                    Status::Match
                }
            );
        }
    }
    #[test]
    fn canonical_plt_and_unsupported_requirements_are_not_omitted() {
        for change in 0..3 {
            let mut import = symbol(false);
            match change {
                0 => import.defined = true,
                1 => import.kind = 6,
                _ => import.binding = 0,
            }
            let c = run(&info(), &[import], &[symbol(true)], &info());
            assert_eq!(c.status, Status::Unresolved);
            assert_eq!(c.rows[0].reason, "unsupported_import_representation");
        }
    }
    #[test]
    fn known_missing_interface_beats_unowned_attribution_after_complete_enumeration() {
        let mut hook = symbol(false);
        hook.owner = None;
        let c = run(&info(), &[symbol(false), hook], &[], &info());
        assert_eq!(c.status, Status::Missing);
    }
}

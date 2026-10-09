//! Read the dynamic dependency metadata of Linux x86-64 ELF files.
//!
//! This deliberately uses program headers, not section headers: stripping the
//! section table must not hide a binary's recorded version requirements. It does
//! not execute the file, resolve its dependent libraries, or certify runtime
//! compatibility. `weak` describes VER_FLG_WEAK on a *version requirement*, not
//! the binding of any individual symbol.
//!
//! Supported input is ELF64 little-endian ET_EXEC/ET_DYN for x86-64. Extended
//! program-header numbering and tables crossing PT_LOAD boundaries are rejected.
//! Inspection limits bound work and allocations for deliberately malformed files.
//!
//! Structure definitions: glibc elf/elf.h; version-chain semantics: Linux
//! Foundation LSB Core Generic, "Symbol Versioning", sections 11.7.4 and 11.7.5.

use std::collections::BTreeSet;

const ELF_HEADER_SIZE: u64 = 64;
const PROGRAM_HEADER_SIZE: u64 = 56;
const DYNAMIC_ENTRY_SIZE: u64 = 16;
const VERSION_ENTRY_SIZE: u64 = 16;

const PT_LOAD: u32 = 1;
const PT_DYNAMIC: u32 = 2;
const PT_INTERP: u32 = 3;
const DT_NULL: u64 = 0;
const DT_NEEDED: u64 = 1;
const DT_STRTAB: u64 = 5;
const DT_STRSZ: u64 = 10;
const DT_RPATH: u64 = 15;
const DT_RUNPATH: u64 = 29;
const DT_VERNEED: u64 = 0x6fff_fffe;
const DT_VERNEEDNUM: u64 = 0x6fff_ffff;
const VER_FLG_WEAK: u16 = 2;

const MAX_PROGRAM_HEADERS: u64 = 1_024;
const MAX_METADATA_RECORDS: usize = 65_536;
const MAX_STRING_LENGTH: usize = 65_536;
const MAX_OUTPUT_STRING_BYTES: usize = 16 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ElfInfo {
    pub machine: u16,
    pub interpreter: Option<String>,
    /// DT_NEEDED values, in their original order.
    pub needed: Vec<String>,
    /// Raw DT_RPATH values; do not expand tokens or split search paths here.
    pub rpath: Vec<String>,
    /// Raw DT_RUNPATH values; do not expand tokens or split search paths here.
    pub runpath: Vec<String>,
    pub versions: Vec<VersionNeed>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VersionNeed {
    pub library: String,
    pub name: String,
    pub weak: bool,
}

#[derive(Clone, Copy, Debug)]
struct Segment {
    offset: u64,
    vaddr: u64,
    filesz: u64,
}

struct Reader<'a> {
    bytes: &'a [u8],
}

impl<'a> Reader<'a> {
    fn take(&self, offset: u64, length: u64) -> Result<&'a [u8], String> {
        let end = offset
            .checked_add(length)
            .ok_or("ELF file offset overflow")?;
        let start = usize::try_from(offset).map_err(|_| "ELF offset exceeds address space")?;
        let end = usize::try_from(end).map_err(|_| "ELF range exceeds address space")?;
        self.bytes.get(start..end).ok_or_else(|| {
            format!("truncated ELF: file range {offset:#x} + {length:#x} is unavailable")
        })
    }

    fn u16(&self, offset: u64) -> Result<u16, String> {
        let b = self.take(offset, 2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    fn u32(&self, offset: u64) -> Result<u32, String> {
        let b = self.take(offset, 4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    fn u64(&self, offset: u64) -> Result<u64, String> {
        let b = self.take(offset, 8)?;
        Ok(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }
}

struct Budget {
    records: usize,
    string_bytes: usize,
}

impl Budget {
    fn record(&mut self) -> Result<(), String> {
        if self.records == 0 {
            return Err("ELF metadata record count exceeds inspection limit".into());
        }
        self.records -= 1;
        Ok(())
    }

    fn charge_string(&mut self, length: usize) -> Result<(), String> {
        self.string_bytes = self
            .string_bytes
            .checked_sub(length)
            .ok_or("ELF metadata strings exceed inspection limit")?;
        Ok(())
    }

    fn string(&mut self, bytes: &[u8], kind: &str, allow_empty: bool) -> Result<String, String> {
        let end = bytes
            .iter()
            .take(MAX_STRING_LENGTH + 1)
            .position(|byte| *byte == 0)
            .ok_or_else(|| {
                format!("{kind} is not NUL terminated within the string inspection limit")
            })?;
        if !allow_empty && end == 0 {
            return Err(format!("{kind} cannot be empty"));
        }
        self.charge_string(end)?;
        let value = std::str::from_utf8(&bytes[..end])
            .map_err(|_| format!("{kind} is not UTF-8; this prototype cannot represent it"))?;
        Ok(value.to_owned())
    }
}

fn add(value: u64, delta: u64, kind: &str) -> Result<u64, String> {
    value
        .checked_add(delta)
        .ok_or_else(|| format!("{kind} address overflow"))
}

/// Translate an address only if every byte is present in a single file-backed
/// PT_LOAD region. The zero-filled part of p_memsz is not file data.
fn file_offset(loads: &[Segment], address: u64, length: u64) -> Result<u64, String> {
    let end = add(address, length, "virtual")?;
    let mut result = None;
    for load in loads {
        let load_end = add(load.vaddr, load.filesz, "PT_LOAD")?;
        if address >= load.vaddr && end <= load_end {
            let offset = add(load.offset, address - load.vaddr, "PT_LOAD file")?;
            if let Some(previous) = result {
                if previous != offset {
                    return Err("ambiguous overlapping PT_LOAD address mapping".into());
                }
            }
            result = Some(offset);
        }
    }
    result.ok_or_else(|| {
        format!("ELF address {address:#x} + {length:#x} is not file-backed by one PT_LOAD")
    })
}

/// Reject partial overlaps with conflicting file mappings too, so resolving a
/// large table cannot overlook an overlapping segment covering only part of it.
fn check_load_mappings(loads: &mut [Segment]) -> Result<(), String> {
    loads.sort_unstable_by_key(|load| load.vaddr);
    let mut previous: Option<Segment> = None;
    for &load in loads.iter().filter(|load| load.filesz != 0) {
        if let Some(prior) = previous {
            let prior_end = add(prior.vaddr, prior.filesz, "PT_LOAD")?;
            if load.vaddr < prior_end {
                let expected = add(prior.offset, load.vaddr - prior.vaddr, "PT_LOAD file")?;
                if expected != load.offset {
                    return Err("ambiguous overlapping PT_LOAD address mapping".into());
                }
                if add(load.vaddr, load.filesz, "PT_LOAD")? <= prior_end {
                    continue;
                }
            }
        }
        previous = Some(load);
    }
    Ok(())
}

fn set_once(slot: &mut Option<u64>, value: u64, name: &str) -> Result<(), String> {
    if slot.replace(value).is_some() {
        return Err(format!("duplicate {name} entry"));
    }
    Ok(())
}

fn table_string(
    strings: &[u8],
    offset: u64,
    kind: &str,
    allow_empty: bool,
    budget: &mut Budget,
) -> Result<String, String> {
    let start = usize::try_from(offset).map_err(|_| format!("{kind} string offset overflow"))?;
    let remaining = strings
        .get(start..)
        .ok_or_else(|| format!("{kind} offset lies outside DT_STRTAB/DT_STRSZ"))?;
    budget.string(remaining, kind, allow_empty)
}

/// Mark a 16-byte version record. Nonzero forward offsets plus this overlap
/// check prevent a malformed chain from reusing headers as auxiliary entries.
fn claim_record(seen: &mut BTreeSet<u64>, address: u64) -> Result<(), String> {
    let end = add(address, VERSION_ENTRY_SIZE, "version record")?;
    if let Some(&prior) = seen.range(..=address).next_back() {
        if add(prior, VERSION_ENTRY_SIZE, "version record")? > address {
            return Err("overlapping or cyclic ELF version records".into());
        }
    }
    if let Some(&next) = seen.range(address..).next() {
        if next < end {
            return Err("overlapping or cyclic ELF version records".into());
        }
    }
    seen.insert(address);
    Ok(())
}

fn next_record(current: u64, delta: u32, last: bool, kind: &str) -> Result<u64, String> {
    if last {
        if delta != 0 {
            return Err(format!("{kind} chain exceeds its declared count"));
        }
        return Ok(current);
    }
    if u64::from(delta) < VERSION_ENTRY_SIZE {
        return Err(format!(
            "{kind} chain ends early or overlaps its current record"
        ));
    }
    add(current, u64::from(delta), kind)
}

fn read_versions(
    reader: &Reader<'_>,
    loads: &[Segment],
    strings: &[u8],
    address: u64,
    count: u64,
    budget: &mut Budget,
) -> Result<Vec<VersionNeed>, String> {
    if count == 0 || count > MAX_METADATA_RECORDS as u64 {
        return Err("invalid DT_VERNEEDNUM or version count exceeds inspection limit".into());
    }
    let mut versions = Vec::new();
    let mut seen = BTreeSet::new();
    let mut current = address;
    for index in 0..count {
        budget.record()?;
        claim_record(&mut seen, current)?;
        let offset = file_offset(loads, current, VERSION_ENTRY_SIZE)?;
        let record = Reader {
            bytes: reader.take(offset, VERSION_ENTRY_SIZE)?,
        };
        if record.u16(0)? != 1 {
            return Err("unsupported ELF Verneed structure version".into());
        }
        let aux_count = u64::from(record.u16(2)?);
        let library = table_string(
            strings,
            u64::from(record.u32(4)?),
            "version dependency library",
            false,
            budget,
        )?;
        let aux_delta = u64::from(record.u32(8)?);
        let next_delta = record.u32(12)?;
        if aux_count == 0 || aux_delta < VERSION_ENTRY_SIZE {
            return Err("Verneed has no auxiliary entries or an invalid vn_aux offset".into());
        }
        let mut aux_address = add(current, aux_delta, "Vernaux")?;
        for aux_index in 0..aux_count {
            budget.record()?;
            claim_record(&mut seen, aux_address)?;
            let offset = file_offset(loads, aux_address, VERSION_ENTRY_SIZE)?;
            let aux = Reader {
                bytes: reader.take(offset, VERSION_ENTRY_SIZE)?,
            };
            let flags = aux.u16(4)?;
            if flags & !3 != 0 {
                return Err(format!(
                    "unsupported ELF version requirement flags {flags:#x}"
                ));
            }
            let name = table_string(
                strings,
                u64::from(aux.u32(8)?),
                "version requirement name",
                false,
                budget,
            )?;
            budget.charge_string(library.len())?;
            versions.push(VersionNeed {
                library: library.clone(),
                name,
                weak: flags & VER_FLG_WEAK != 0,
            });
            aux_address = next_record(
                aux_address,
                aux.u32(12)?,
                aux_index + 1 == aux_count,
                "Vernaux",
            )?;
        }
        current = next_record(current, next_delta, index + 1 == count, "Verneed")?;
    }
    Ok(versions)
}

pub fn inspect(bytes: &[u8]) -> Result<ElfInfo, String> {
    let reader = Reader { bytes };
    let header = reader.take(0, ELF_HEADER_SIZE)?;
    if &header[..4] != b"\x7fELF" {
        return Err("input is not an ELF file".into());
    }
    if header[4] != 2 || header[5] != 1 {
        return Err("only ELF64 little-endian files are supported".into());
    }
    if header[6] != 1 || reader.u32(20)? != 1 {
        return Err("unsupported ELF format version".into());
    }
    if header[7] != 0 && header[7] != 3 {
        return Err("only System V / GNU Linux ELF OS ABI values are supported".into());
    }
    let file_type = reader.u16(16)?;
    if file_type != 2 && file_type != 3 {
        return Err(
            "only ELF executables and shared objects (ET_EXEC/ET_DYN) are supported".into(),
        );
    }
    let machine = reader.u16(18)?;
    if machine != 62 {
        return Err(format!(
            "only x86-64 ELF is supported (e_machine is {machine})"
        ));
    }
    if reader.u16(52)? != ELF_HEADER_SIZE as u16 {
        return Err("unexpected ELF header size".into());
    }
    let phoff = reader.u64(32)?;
    let phsize = u64::from(reader.u16(54)?);
    let phcount = u64::from(reader.u16(56)?);
    if phcount == 0xffff {
        return Err("extended ELF program-header numbering is not supported".into());
    }
    if phcount == 0 || phcount > MAX_PROGRAM_HEADERS {
        return Err("ELF program-header count is zero or exceeds inspection limit".into());
    }
    if phsize != PROGRAM_HEADER_SIZE || phoff < ELF_HEADER_SIZE {
        return Err("invalid ELF program-header entry size or table offset".into());
    }
    // phcount is bounded before multiplication, and `take` checks the sum.
    reader.take(phoff, phcount * phsize)?;

    let mut loads = Vec::new();
    let mut dynamic = None;
    let mut interpreter = None;
    let mut budget = Budget {
        records: MAX_METADATA_RECORDS,
        string_bytes: MAX_OUTPUT_STRING_BYTES,
    };
    for index in 0..phcount {
        let offset = add(phoff, index * phsize, "program header")?;
        let ph = Reader {
            bytes: reader.take(offset, phsize)?,
        };
        let kind = ph.u32(0)?;
        let segment = Segment {
            offset: ph.u64(8)?,
            vaddr: ph.u64(16)?,
            filesz: ph.u64(32)?,
        };
        let memsz = ph.u64(40)?;
        if segment.filesz != 0 {
            reader.take(segment.offset, segment.filesz)?;
        }
        match kind {
            PT_LOAD => {
                if segment.filesz > memsz {
                    return Err("PT_LOAD file size exceeds memory size".into());
                }
                add(segment.vaddr, memsz, "PT_LOAD memory")?;
                loads.push(segment);
            }
            PT_DYNAMIC => {
                if dynamic.replace(segment).is_some() {
                    return Err("duplicate PT_DYNAMIC segments".into());
                }
            }
            PT_INTERP => {
                if interpreter.is_some() {
                    return Err("duplicate PT_INTERP segments".into());
                }
                let data = reader.take(segment.offset, segment.filesz)?;
                let value = budget.string(data, "ELF interpreter", false)?;
                if data[value.len() + 1..].iter().any(|byte| *byte != 0) {
                    return Err("nonzero bytes follow the ELF interpreter terminator".into());
                }
                interpreter = Some(value);
            }
            _ => {}
        }
    }
    if loads.is_empty() {
        return Err("ELF executable/shared object has no PT_LOAD segments".into());
    }
    check_load_mappings(&mut loads)?;
    let mut info = ElfInfo {
        machine,
        interpreter,
        needed: Vec::new(),
        rpath: Vec::new(),
        runpath: Vec::new(),
        versions: Vec::new(),
    };
    let Some(dynamic) = dynamic else {
        return Ok(info);
    };
    if dynamic.filesz == 0 || dynamic.filesz % DYNAMIC_ENTRY_SIZE != 0 {
        return Err("PT_DYNAMIC size is not a nonempty array of ELF64 dynamic entries".into());
    }
    if file_offset(&loads, dynamic.vaddr, dynamic.filesz)? != dynamic.offset {
        return Err("PT_DYNAMIC file offset disagrees with its PT_LOAD address mapping".into());
    }
    let entries = dynamic.filesz / DYNAMIC_ENTRY_SIZE;
    if entries > MAX_METADATA_RECORDS as u64 {
        return Err("PT_DYNAMIC entry count exceeds inspection limit".into());
    }
    let mut strtab = None;
    let mut strsz = None;
    let mut verneed = None;
    let mut verneednum = None;
    let mut needed = Vec::new();
    let mut rpath = None;
    let mut runpath = None;
    let mut terminated = false;
    for index in 0..entries {
        budget.record()?;
        let offset = add(dynamic.offset, index * DYNAMIC_ENTRY_SIZE, "dynamic entry")?;
        let tag = reader.u64(offset)?;
        let value = reader.u64(add(offset, 8, "dynamic entry")?)?;
        match tag {
            DT_NULL => {
                terminated = true;
                break;
            }
            DT_NEEDED => needed.push(value),
            DT_STRTAB => set_once(&mut strtab, value, "DT_STRTAB")?,
            DT_STRSZ => set_once(&mut strsz, value, "DT_STRSZ")?,
            DT_RPATH => set_once(&mut rpath, value, "DT_RPATH")?,
            DT_RUNPATH => set_once(&mut runpath, value, "DT_RUNPATH")?,
            DT_VERNEED => set_once(&mut verneed, value, "DT_VERNEED")?,
            DT_VERNEEDNUM => set_once(&mut verneednum, value, "DT_VERNEEDNUM")?,
            _ => {}
        }
    }
    if !terminated {
        return Err("PT_DYNAMIC has no DT_NULL terminator".into());
    }
    let strings = match (strtab, strsz) {
        (None, None) => &[][..],
        (Some(address), Some(length)) if length != 0 => {
            let offset = file_offset(&loads, address, length)?;
            let data = reader.take(offset, length)?;
            if data.first() != Some(&0) || data.last() != Some(&0) {
                return Err("DT_STRTAB must start and end with a NUL byte".into());
            }
            data
        }
        _ => return Err("DT_STRTAB and a nonzero DT_STRSZ must be supplied together".into()),
    };
    for offset in needed {
        info.needed.push(table_string(
            strings,
            offset,
            "DT_NEEDED library",
            false,
            &mut budget,
        )?);
    }
    if let Some(offset) = rpath {
        info.rpath.push(table_string(
            strings,
            offset,
            "DT_RPATH",
            true,
            &mut budget,
        )?);
    }
    if let Some(offset) = runpath {
        info.runpath.push(table_string(
            strings,
            offset,
            "DT_RUNPATH",
            true,
            &mut budget,
        )?);
    }
    match (verneed, verneednum) {
        (None, None | Some(0)) => {}
        (Some(address), Some(count)) => {
            info.versions = read_versions(&reader, &loads, strings, address, count, &mut budget)?;
        }
        _ => return Err("DT_VERNEED and DT_VERNEEDNUM must be supplied together".into()),
    }
    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: u64 = 0x40_0000;
    const DYNAMIC: usize = 0x200;
    const STRINGS: usize = 0x300;
    const VERNEED: usize = 0x400;
    const INTERP: usize = 0x500;

    fn put16(bytes: &mut [u8], offset: usize, value: u16) {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }

    fn put32(bytes: &mut [u8], offset: usize, value: u32) {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }

    fn put64(bytes: &mut [u8], offset: usize, value: u64) {
        bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }

    fn ph(bytes: &mut [u8], index: usize, kind: u32, offset: usize, size: usize) {
        let start = 64 + index * 56;
        put32(bytes, start, kind);
        put64(bytes, start + 8, offset as u64);
        put64(bytes, start + 16, BASE + offset as u64);
        put64(bytes, start + 32, size as u64);
        put64(bytes, start + 40, size as u64);
    }

    fn dyn_entry(bytes: &mut [u8], index: usize, tag: u64, value: u64) {
        put64(bytes, DYNAMIC + index * 16, tag);
        put64(bytes, DYNAMIC + index * 16 + 8, value);
    }

    fn string(bytes: &mut [u8], next: &mut usize, value: &str) -> u32 {
        let offset = *next;
        let end = offset + value.len();
        bytes[STRINGS + offset..STRINGS + end].copy_from_slice(value.as_bytes());
        bytes[STRINGS + end] = 0;
        *next = end + 1;
        offset as u32
    }

    /// A stripped PIE with two dependencies, two Verneed chains, three Vernaux
    /// records (one weak), and no section headers whatsoever.
    fn fixture() -> Vec<u8> {
        let mut bytes = vec![0_u8; 0x600];
        bytes[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
        put16(&mut bytes, 16, 3);
        put16(&mut bytes, 18, 62);
        put32(&mut bytes, 20, 1);
        put64(&mut bytes, 32, 64);
        put16(&mut bytes, 52, 64);
        put16(&mut bytes, 54, 56);
        put16(&mut bytes, 56, 3);
        let length = bytes.len();
        ph(&mut bytes, 0, PT_LOAD, 0, length);
        ph(&mut bytes, 1, PT_DYNAMIC, DYNAMIC, 9 * 16);
        let interp = b"/lib64/ld-linux-x86-64.so.2\0";
        ph(&mut bytes, 2, PT_INTERP, INTERP, interp.len());
        bytes[INTERP..INTERP + interp.len()].copy_from_slice(interp);

        let mut next = 1;
        let libc = string(&mut bytes, &mut next, "libc.so.6");
        let libgcc = string(&mut bytes, &mut next, "libgcc_s.so.1");
        let old = string(&mut bytes, &mut next, "GLIBC_2.2.5");
        let newer_weak = string(&mut bytes, &mut next, "GLIBC_2.34");
        let gcc = string(&mut bytes, &mut next, "GCC_3.0");
        let rpath = string(&mut bytes, &mut next, "/old/lib:/old/lib64");
        let runpath = string(&mut bytes, &mut next, "$ORIGIN/lib::/usr/lib64");
        dyn_entry(&mut bytes, 0, DT_STRTAB, BASE + STRINGS as u64);
        dyn_entry(&mut bytes, 1, DT_STRSZ, next as u64);
        dyn_entry(&mut bytes, 2, DT_NEEDED, u64::from(libc));
        dyn_entry(&mut bytes, 3, DT_NEEDED, u64::from(libgcc));
        dyn_entry(&mut bytes, 4, DT_VERNEED, BASE + VERNEED as u64);
        dyn_entry(&mut bytes, 5, DT_VERNEEDNUM, 2);
        dyn_entry(&mut bytes, 6, DT_RPATH, u64::from(rpath));
        dyn_entry(&mut bytes, 7, DT_RUNPATH, u64::from(runpath));
        dyn_entry(&mut bytes, 8, DT_NULL, 0);

        put16(&mut bytes, VERNEED, 1);
        put16(&mut bytes, VERNEED + 2, 2);
        put32(&mut bytes, VERNEED + 4, libc);
        put32(&mut bytes, VERNEED + 8, 16);
        put32(&mut bytes, VERNEED + 12, 48);
        put16(&mut bytes, VERNEED + 16 + 6, 2);
        put32(&mut bytes, VERNEED + 16 + 8, old);
        put32(&mut bytes, VERNEED + 16 + 12, 16);
        put16(&mut bytes, VERNEED + 32 + 4, VER_FLG_WEAK);
        put16(&mut bytes, VERNEED + 32 + 6, 3);
        put32(&mut bytes, VERNEED + 32 + 8, newer_weak);

        put16(&mut bytes, VERNEED + 48, 1);
        put16(&mut bytes, VERNEED + 48 + 2, 1);
        put32(&mut bytes, VERNEED + 48 + 4, libgcc);
        put32(&mut bytes, VERNEED + 48 + 8, 16);
        put16(&mut bytes, VERNEED + 64 + 6, 4);
        put32(&mut bytes, VERNEED + 64 + 8, gcc);
        bytes
    }

    #[test]
    fn reads_stripped_dynamic_metadata_and_distinguishes_weak_versions() {
        let info = inspect(&fixture()).unwrap();
        assert_eq!(info.machine, 62);
        assert_eq!(
            info.interpreter.as_deref(),
            Some("/lib64/ld-linux-x86-64.so.2")
        );
        assert_eq!(info.needed, ["libc.so.6", "libgcc_s.so.1"]);
        assert_eq!(info.rpath, ["/old/lib:/old/lib64"]);
        assert_eq!(info.runpath, ["$ORIGIN/lib::/usr/lib64"]);
        assert_eq!(
            info.versions,
            [
                VersionNeed {
                    library: "libc.so.6".into(),
                    name: "GLIBC_2.2.5".into(),
                    weak: false,
                },
                VersionNeed {
                    library: "libc.so.6".into(),
                    name: "GLIBC_2.34".into(),
                    weak: true,
                },
                VersionNeed {
                    library: "libgcc_s.so.1".into(),
                    name: "GCC_3.0".into(),
                    weak: false,
                },
            ]
        );
    }

    #[test]
    fn accepts_executable_without_dynamic_metadata() {
        let mut bytes = fixture();
        put16(&mut bytes, 16, 2);
        put16(&mut bytes, 56, 1);
        let info = inspect(&bytes).unwrap();
        assert!(info.interpreter.is_none());
        assert!(info.needed.is_empty());
        assert!(info.versions.is_empty());
    }

    #[test]
    fn every_truncated_prefix_returns_an_error_without_panicking() {
        let bytes = fixture();
        for length in 0..bytes.len() {
            assert!(
                inspect(&bytes[..length]).is_err(),
                "accepted prefix {length}"
            );
        }
    }

    #[test]
    fn rejects_unsupported_architecture_encoding_and_object_type() {
        for (offset, value) in [(0, 0), (4, 1), (5, 2), (6, 2), (7, 9), (16, 1), (18, 3)] {
            let mut bytes = fixture();
            bytes[offset] = value;
            assert!(inspect(&bytes).is_err(), "accepted mutation at {offset}");
        }
    }

    #[test]
    fn rejects_header_count_range_and_overflow() {
        let mut bytes = fixture();
        put16(&mut bytes, 56, 0xffff);
        assert!(inspect(&bytes).unwrap_err().contains("extended"));
        put16(&mut bytes, 56, 3);
        put64(&mut bytes, 32, u64::MAX - 16);
        assert!(inspect(&bytes).unwrap_err().contains("overflow"));
    }

    #[test]
    fn rejects_dynamic_pointer_outside_file_backed_memory() {
        let mut bytes = fixture();
        dyn_entry(&mut bytes, 0, DT_STRTAB, BASE + 0x700);
        put64(&mut bytes, 64 + 40, 0x1000); // valid BSS, not valid string-table bytes
        assert!(inspect(&bytes).unwrap_err().contains("file-backed"));
    }

    #[test]
    fn rejects_dynamic_address_offset_disagreement() {
        let mut bytes = fixture();
        put64(&mut bytes, 64 + 56 + 16, BASE + DYNAMIC as u64 + 16);
        assert!(inspect(&bytes).unwrap_err().contains("disagrees"));
    }

    #[test]
    fn rejects_unterminated_or_duplicate_dynamic_metadata() {
        let mut bytes = fixture();
        dyn_entry(&mut bytes, 8, 21, 0);
        assert!(inspect(&bytes).unwrap_err().contains("DT_NULL"));
        dyn_entry(&mut bytes, 8, DT_STRTAB, BASE + STRINGS as u64);
        assert!(inspect(&bytes).unwrap_err().contains("duplicate DT_STRTAB"));
    }

    #[test]
    fn rejects_invalid_string_bounds_and_termination() {
        let mut bytes = fixture();
        dyn_entry(&mut bytes, 2, DT_NEEDED, u64::MAX);
        assert!(inspect(&bytes).is_err());
        let mut bytes = fixture();
        bytes[INTERP + b"/lib64/ld-linux-x86-64.so.2".len()] = b'x';
        assert!(inspect(&bytes).unwrap_err().contains("NUL terminated"));
        let mut bytes = fixture();
        bytes[STRINGS] = b'x';
        assert!(inspect(&bytes).unwrap_err().contains("start and end"));
    }

    #[test]
    fn rejects_early_or_overlong_version_chains() {
        let mut bytes = fixture();
        put32(&mut bytes, VERNEED + 12, 0);
        assert!(inspect(&bytes).unwrap_err().contains("ends early"));
        let mut bytes = fixture();
        put32(&mut bytes, VERNEED + 32 + 12, 16);
        assert!(inspect(&bytes)
            .unwrap_err()
            .contains("exceeds its declared count"));
        let mut bytes = fixture();
        put32(&mut bytes, VERNEED + 16 + 12, 0);
        assert!(inspect(&bytes).unwrap_err().contains("ends early"));
    }

    #[test]
    fn rejects_overlapping_or_wrapping_version_records() {
        let mut bytes = fixture();
        put32(&mut bytes, VERNEED + 12, 16); // next Verneed aliases this one's Vernaux
        assert!(inspect(&bytes).unwrap_err().contains("overlapping"));
        let mut bytes = fixture();
        put32(&mut bytes, VERNEED + 8, u32::MAX);
        assert!(inspect(&bytes).unwrap_err().contains("file-backed"));
        let mut bytes = fixture();
        dyn_entry(&mut bytes, 4, DT_VERNEED, u64::MAX - 7);
        assert!(inspect(&bytes).unwrap_err().contains("overflow"));
    }

    #[test]
    fn rejects_huge_version_counts_and_missing_companion_tags() {
        let mut bytes = fixture();
        dyn_entry(&mut bytes, 5, DT_VERNEEDNUM, u64::MAX);
        assert!(inspect(&bytes).unwrap_err().contains("inspection limit"));
        let mut bytes = fixture();
        dyn_entry(&mut bytes, 5, 21, 0);
        assert!(inspect(&bytes).unwrap_err().contains("supplied together"));
    }

    #[test]
    fn rejects_partial_overlapping_load_mappings() {
        let mut bytes = fixture();
        put16(&mut bytes, 56, 4);
        ph(&mut bytes, 3, PT_LOAD, 0x100, 0x100);
        put64(&mut bytes, 64 + 3 * 56 + 16, BASE + 0x120);
        assert!(inspect(&bytes)
            .unwrap_err()
            .contains("ambiguous overlapping"));
    }

    #[test]
    fn accepts_redundant_load_with_identical_address_mapping() {
        let mut bytes = fixture();
        put16(&mut bytes, 56, 4);
        ph(&mut bytes, 3, PT_LOAD, STRINGS, 0x100);
        assert_eq!(inspect(&bytes).unwrap().versions.len(), 3);
    }
}

//! Optional bounded dynamic-symbol extraction. Legacy inspection never calls it.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

const MAX_SYMBOLS: u64 = 262_144;
const SYMBOL_SIZE: u64 = 24;

#[derive(Clone, Debug)]
pub(crate) struct Symbol {
    pub name: String,
    pub binding: u8,
    pub kind: u8,
    pub visibility: u8,
    pub defined: bool,
    pub value: u64,
    pub section: u16,
    pub version_index: u16,
    pub version: Option<String>,
    pub owner: Option<String>,
    pub version_weak: bool,
    pub version_hidden: bool,
    pub lookup_visible: bool,
}

#[derive(Clone)]
struct VersionIndex {
    name: String,
    owner: Option<String>,
    weak: bool,
}

fn insert_version(
    indexes: &mut BTreeMap<u16, VersionIndex>,
    index: u16,
    value: VersionIndex,
) -> Result<(), String> {
    if index < 2 || indexes.insert(index, value).is_some() {
        return Err("duplicate or reserved dynamic version index".into());
    }
    Ok(())
}

fn hash_name(name: &str, gnu: bool) -> u32 {
    if gnu {
        name.bytes().fold(5381_u32, |h, b| {
            h.wrapping_mul(33).wrapping_add(u32::from(b))
        })
    } else {
        name.bytes().fold(0_u32, |mut h, b| {
            h = (h << 4).wrapping_add(u32::from(b));
            let g = h & 0xf000_0000;
            h ^= g >> 24;
            h & !g
        })
    }
}

/// Returns an exhaustive dynsym inventory or an error, never a partial result.
/// Counts come from dynamic SysV/GNU hash tables, not section headers or EOF.
pub(crate) fn inspect(bytes: &[u8]) -> Result<Vec<Symbol>, String> {
    super::inspect(bytes)?;
    let reader = Reader { bytes };
    let mut loads = Vec::new();
    let mut dynamic = None;
    let phoff = reader.u64(32)?;
    for i in 0..u64::from(reader.u16(56)?) {
        let ph = add(phoff, i * PROGRAM_HEADER_SIZE, "program header")?;
        let segment = Segment {
            offset: reader.u64(ph + 8)?,
            vaddr: reader.u64(ph + 16)?,
            filesz: reader.u64(ph + 32)?,
        };
        match reader.u32(ph)? {
            PT_LOAD => loads.push(segment),
            PT_DYNAMIC => dynamic = Some(segment),
            _ => {}
        }
    }
    let dynamic = dynamic.ok_or("no dynamic symbol metadata")?;
    let mut tags = BTreeMap::new();
    for i in 0..dynamic.filesz / DYNAMIC_ENTRY_SIZE {
        let offset = dynamic.offset + i * DYNAMIC_ENTRY_SIZE;
        let tag = reader.u64(offset)?;
        if tag == DT_NULL {
            break;
        }
        // Only scalar tags relevant to this optional parser are unique here.
        if [
            2,
            7,
            8,
            9,
            17,
            18,
            19,
            20,
            23,
            4,
            5,
            6,
            10,
            11,
            0x6fff_fef5,
            0x6fff_fff0,
            DT_VERNEED,
            DT_VERNEEDNUM,
            DT_VERDEF,
            DT_VERDEFNUM,
        ]
        .contains(&tag)
            && tags.insert(tag, reader.u64(offset + 8)?).is_some()
        {
            return Err(format!("duplicate dynamic symbol tag {tag:#x}"));
        }
    }
    let required = |tag| {
        tags.get(&tag)
            .copied()
            .ok_or_else(|| format!("missing dynamic symbol tag {tag:#x}"))
    };
    let mapped = |address, length| file_offset(&loads, address, length);
    let strsz = required(DT_STRSZ)?;
    let strings = reader.take(mapped(required(DT_STRTAB)?, strsz)?, strsz)?;
    let mut budget = Budget {
        records: MAX_METADATA_RECORDS,
        string_bytes: MAX_OUTPUT_STRING_BYTES,
    };
    let mut indexes = BTreeMap::new();
    let mut seen_records = BTreeSet::new();
    if let Some(&address) = tags.get(&DT_VERNEED) {
        let count = required(DT_VERNEEDNUM)?;
        let mut current = address;
        for _ in 0..count {
            budget.record()?;
            claim_record(&mut seen_records, current)?;
            let off = mapped(current, 16)?;
            let owner = table_string(
                strings,
                u64::from(reader.u32(off + 4)?),
                "version owner",
                false,
                &mut budget,
            )?;
            let mut aux = add(current, u64::from(reader.u32(off + 8)?), "Vernaux")?;
            for _ in 0..reader.u16(off + 2)? {
                budget.record()?;
                claim_record(&mut seen_records, aux)?;
                let a = mapped(aux, 16)?;
                let index = reader.u16(a + 6)?;
                if index & 0x8000 != 0 {
                    return Err("unsupported high-bit Vernaux index".into());
                }
                let name = table_string(
                    strings,
                    u64::from(reader.u32(a + 8)?),
                    "version name",
                    false,
                    &mut budget,
                )?;
                if reader.u32(a)? != hash_name(&name, false) {
                    return Err("Vernaux hash disagrees with version name".into());
                }
                budget.charge_string(owner.len())?;
                insert_version(
                    &mut indexes,
                    index,
                    VersionIndex {
                        name,
                        owner: Some(owner.clone()),
                        weak: reader.u16(a + 4)? & VER_FLG_WEAK != 0,
                    },
                )?;
                aux = add(aux, u64::from(reader.u32(a + 12)?), "Vernaux")?;
            }
            current = add(current, u64::from(reader.u32(off + 12)?), "Verneed")?;
        }
    }
    if let Some(&address) = tags.get(&DT_VERDEF) {
        let count = required(DT_VERDEFNUM)?;
        let mut current = address;
        let mut claimed = BTreeSet::new();
        for _ in 0..count {
            budget.record()?;
            let off = mapped(current, VERDEF_SIZE)?;
            // Claim each byte of the small records; budget bounds the total work.
            for byte in current..add(current, VERDEF_SIZE, "Verdef")? {
                if !claimed.insert(byte) {
                    return Err("overlapping Verdef records".into());
                }
            }
            let flags = reader.u16(off + 2)?;
            if flags & !7 != 0 {
                return Err("unsupported Verdef flags".into());
            }
            let index = reader.u16(off + 4)?;
            if index & 0x8000 != 0 {
                return Err("unsupported high-bit Verdef index".into());
            }
            let aux_count = reader.u16(off + 6)?;
            let mut aux = add(current, u64::from(reader.u32(off + 12)?), "Verdaux")?;
            let mut name = None;
            for i in 0..aux_count {
                budget.record()?;
                for byte in aux..add(aux, VERDAUX_SIZE, "Verdaux")? {
                    if !claimed.insert(byte) {
                        return Err("overlapping Verdaux records".into());
                    }
                }
                let a = mapped(aux, VERDAUX_SIZE)?;
                let value = table_string(
                    strings,
                    u64::from(reader.u32(a)?),
                    "definition name",
                    false,
                    &mut budget,
                )?;
                if i == 0 {
                    if reader.u32(off + 8)? != hash_name(&value, false) {
                        return Err("Verdef hash disagrees with version name".into());
                    }
                    name = Some(value);
                }
                let next = reader.u32(a + 4)?;
                if (i + 1 == aux_count && next != 0) || (i + 1 < aux_count && next < 8) {
                    return Err("invalid Verdaux chain".into());
                }
                aux = add(aux, u64::from(next), "Verdaux")?;
            }
            if flags & VER_FLG_BASE == 0 {
                insert_version(
                    &mut indexes,
                    index,
                    VersionIndex {
                        name: name.ok_or("empty Verdef")?,
                        owner: None,
                        weak: false,
                    },
                )?;
            } else if index != 1 {
                return Err("invalid base version index".into());
            }
            current = add(current, u64::from(reader.u32(off + 16)?), "Verdef")?;
        }
    }

    let mut count = None;
    let mut sysv = None;
    if let Some(&address) = tags.get(&4) {
        let off = mapped(address, 8)?;
        let buckets = u64::from(reader.u32(off)?);
        let chains = u64::from(reader.u32(off + 4)?);
        if buckets == 0 || buckets > MAX_SYMBOLS || chains == 0 || chains > MAX_SYMBOLS {
            return Err("SysV hash count exceeds symbol inspection limit or is zero".into());
        }
        let off = mapped(address, 8 + (buckets + chains) * 4)?;
        sysv = Some((off, buckets, chains));
        count = Some(chains);
    }
    let mut gnu = None;
    let mut gnu_reachable = BTreeMap::new();
    if let Some(&address) = tags.get(&0x6fff_fef5) {
        let off = mapped(address, 16)?;
        let buckets = u64::from(reader.u32(off)?);
        let first = u64::from(reader.u32(off + 4)?);
        let bloom = u64::from(reader.u32(off + 8)?);
        let shift = reader.u32(off + 12)?;
        if buckets == 0
            || buckets > MAX_SYMBOLS
            || first == 0
            || first > MAX_SYMBOLS
            || bloom == 0
            || bloom > MAX_SYMBOLS
            || !bloom.is_power_of_two()
            || shift >= 32
        {
            return Err("invalid or over-budget GNU hash header".into());
        }
        let bucket_address = add(address, 16 + bloom * 8, "GNU buckets")?;
        let bucket_offset = mapped(bucket_address, buckets * 4)?;
        let chains = add(bucket_address, buckets * 4, "GNU chains")?;
        let mut end = first;
        for bucket in 0..buckets {
            let mut index = u64::from(reader.u32(bucket_offset + bucket * 4)?);
            if index == 0 {
                continue;
            }
            if index < first {
                return Err("GNU hash bucket precedes symoffset".into());
            }
            loop {
                if index >= MAX_SYMBOLS
                    || gnu_reachable.len() as u64 >= MAX_SYMBOLS
                    || gnu_reachable.contains_key(&index)
                {
                    return Err("GNU hash chains overlap or exceed symbol inspection limit".into());
                }
                let a = add(chains, (index - first) * 4, "GNU chain")?;
                let hash = reader.u32(mapped(a, 4)?)?;
                gnu_reachable.insert(index, (bucket, hash));
                end = end.max(index + 1);
                if hash & 1 != 0 {
                    break;
                }
                index += 1;
            }
        }
        if count.is_some_and(|c| c != end) {
            return Err("contradictory SysV/GNU symbol counts".into());
        }
        count = Some(end);
        gnu = Some((mapped(address + 16, bloom * 8)?, bloom, shift, buckets));
    }
    let count = count.ok_or("no supported hash table: dynamic symbol enumeration unresolved")?;
    if required(11)? != SYMBOL_SIZE {
        return Err("unexpected DT_SYMENT".into());
    }
    let symbols_offset = mapped(required(6)?, count * SYMBOL_SIZE)?;
    // A section count, when present, must agree with dynamic hash evidence.
    // Stripped/sectionless inputs continue to use bounded dynamic metadata.
    let shoff = reader.u64(40)?;
    let shnum = u64::from(reader.u16(60)?);
    if shoff != 0 {
        if shnum == 0 || shnum > MAX_METADATA_RECORDS as u64 || reader.u16(58)? != 64 {
            return Err(
                "unsupported section-table count/size for optional symbol cross-check".into(),
            );
        }
        reader.take(shoff, shnum * 64)?;
        let mut found = false;
        for i in 0..shnum {
            let off = shoff + i * 64;
            if reader.u32(off + 4)? == 11 {
                if found
                    || reader.u64(off + 16)? != required(6)?
                    || reader.u64(off + 24)? != symbols_offset
                    || reader.u64(off + 32)? != count * SYMBOL_SIZE
                    || reader.u64(off + 56)? != SYMBOL_SIZE
                {
                    return Err("section/dynamic symbol counts or mappings disagree".into());
                }
                found = true;
            }
        }
    } else if shnum != 0 {
        return Err("section count without section table".into());
    }
    // Dynamic relocations can refer to imported symbols omitted by a forged
    // hash bound. Never accept those out-of-inventory references as no imports.
    let mut relocation_budget = MAX_SYMBOLS;
    let mut relocation_table = |address: u64, size: u64, entry: u64| -> Result<(), String> {
        if !size.is_multiple_of(entry) || size / entry > relocation_budget {
            return Err("dynamic relocations exceed inspection limit or have invalid size".into());
        }
        relocation_budget -= size / entry;
        if size == 0 {
            return Ok(());
        }
        let off = mapped(address, size)?;
        for i in 0..size / entry {
            if reader.u64(off + i * entry + 8)? >> 32 >= count {
                return Err(
                    "relocation references a symbol outside the declared hash inventory".into(),
                );
            }
        }
        Ok(())
    };
    for (address_tag, size_tag, entry_tag, entry) in [(7, 8, 9, 24), (17, 18, 19, 16)] {
        match (tags.get(&address_tag), tags.get(&size_tag)) {
            (Some(&address), Some(&size)) => {
                if required(entry_tag)? != entry {
                    return Err("invalid dynamic relocation entry size".into());
                }
                relocation_table(address, size, entry)?;
            }
            (None, None) => {}
            _ => return Err("incomplete dynamic relocation metadata".into()),
        }
    }
    match (tags.get(&23), tags.get(&2), tags.get(&20)) {
        (Some(&address), Some(&size), Some(&kind)) if kind == 7 || kind == 17 => {
            relocation_table(address, size, if kind == 7 { 24 } else { 16 })?;
        }
        (None, None, None) => {}
        _ => return Err("incomplete or unsupported PLT relocation metadata".into()),
    }

    let versym = tags
        .get(&0x6fff_fff0)
        .map(|&address| mapped(address, count * 2))
        .transpose()?;
    if !indexes.is_empty() && versym.is_none() {
        return Err("version indexes exist without DT_VERSYM".into());
    }
    let mut symbols = Vec::new();
    for i in 0..count {
        let off = symbols_offset + i * SYMBOL_SIZE;
        let name = table_string(
            strings,
            u64::from(reader.u32(off)?),
            "dynamic symbol name",
            true,
            &mut budget,
        )?;
        let info = reader.take(off + 4, 2)?;
        let version_bits = versym
            .map(|v| reader.u16(v + i * 2))
            .transpose()?
            .unwrap_or(1);
        let index = version_bits & 0x7fff;
        let version = if index >= 2 {
            Some(
                indexes
                    .get(&index)
                    .ok_or("symbol has an unmapped version index")?,
            )
        } else {
            None
        };
        if i == 0 && (info != [0, 0] || reader.u32(off)? != 0 || reader.u16(off + 6)? != 0) {
            return Err("invalid null dynamic symbol".into());
        }
        let visibility = info[1] & 3;
        if info[1] & !3 != 0 {
            return Err("unsupported dynamic symbol visibility flags".into());
        }
        let defined = reader.u16(off + 6)? != 0;
        if !name.is_empty() {
            if let Some((bloom_off, bloom, shift, buckets)) = gnu {
                if let Some(&(bucket, hash)) = gnu_reachable.get(&i) {
                    let actual = hash_name(&name, true);
                    let word = reader.u64(bloom_off + (u64::from(actual / 64) % bloom) * 8)?;
                    if u64::from(actual) % buckets != bucket
                        || hash | 1 != actual | 1
                        || word & (1_u64 << (actual % 64)) == 0
                        || word & (1_u64 << ((actual >> shift) % 64)) == 0
                    {
                        return Err("GNU hash lookup metadata disagrees with symbol name".into());
                    }
                }
            }
        }
        if let Some(v) = version {
            budget.charge_string(v.name.len() + v.owner.as_ref().map_or(0, String::len))?;
        }
        symbols.push(Symbol {
            name,
            binding: info[0] >> 4,
            kind: info[0] & 15,
            visibility,
            defined,
            value: reader.u64(off + 8)?,
            section: reader.u16(off + 6)?,
            version_index: index,
            version: version.map(|v| v.name.clone()),
            owner: version.and_then(|v| v.owner.clone()),
            version_weak: version.is_some_and(|v| v.weak),
            version_hidden: version_bits & 0x8000 != 0,
            lookup_visible: index != 0 && gnu_reachable.contains_key(&i),
        });
    }
    if let Some((off, buckets, chains)) = sysv {
        let mut visited = BTreeSet::new();
        for bucket in 0..buckets {
            let mut index = u64::from(reader.u32(off + 8 + bucket * 4)?);
            while index != 0 {
                if index >= chains || !visited.insert(index) {
                    return Err("cyclic, overlapping or out-of-range SysV hash chain".into());
                }
                let s = &mut symbols[index as usize];
                if u64::from(hash_name(&s.name, false)) % buckets != bucket {
                    return Err("SysV bucket disagrees with symbol name".into());
                }
                if gnu.is_none() {
                    s.lookup_visible = s.version_index != 0;
                }
                index = u64::from(reader.u32(off + 8 + (buckets + index) * 4)?);
            }
        }
        for index in 0..chains {
            if u64::from(reader.u32(off + 8 + (buckets + index) * 4)?) >= chains {
                return Err("out-of-range SysV hash chain entry".into());
            }
        }
    }
    Ok(symbols.into_iter().filter(|s| !s.name.is_empty()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    const BASE: u64 = 0x400000;
    fn put16(b: &mut [u8], p: usize, v: u16) {
        b[p..p + 2].copy_from_slice(&v.to_le_bytes());
    }
    fn put32(b: &mut [u8], p: usize, v: u32) {
        b[p..p + 4].copy_from_slice(&v.to_le_bytes());
    }
    fn put64(b: &mut [u8], p: usize, v: u64) {
        b[p..p + 8].copy_from_slice(&v.to_le_bytes());
    }
    fn tag(b: &mut [u8], i: usize, t: u64, v: u64) {
        put64(b, 0x100 + i * 16, t);
        put64(b, 0x108 + i * 16, v);
    }
    fn fixture() -> Vec<u8> {
        let mut b = vec![0; 0x1000];
        b[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
        put16(&mut b, 16, 3);
        put16(&mut b, 18, 62);
        put32(&mut b, 20, 1);
        put64(&mut b, 32, 64);
        put16(&mut b, 52, 64);
        put16(&mut b, 54, 56);
        put16(&mut b, 56, 2);
        put32(&mut b, 64, PT_LOAD);
        put64(&mut b, 80, BASE);
        put64(&mut b, 96, 0x1000);
        put64(&mut b, 104, 0x1000);
        put32(&mut b, 120, PT_DYNAMIC);
        put64(&mut b, 128, 0x100);
        put64(&mut b, 136, BASE + 0x100);
        put64(&mut b, 152, 9 * 16);
        put64(&mut b, 160, 9 * 16);
        let strings = b"\0f\0libgcc_s.so.1\0GCC_4.2.0\0";
        b[0x400..0x400 + strings.len()].copy_from_slice(strings);
        for (i, (t, v)) in [
            (DT_STRTAB, BASE + 0x400),
            (DT_STRSZ, strings.len() as u64),
            (6, BASE + 0x500),
            (11, 24),
            (4, BASE + 0x700),
            (0x6fff_fff0, BASE + 0x800),
            (DT_VERNEED, BASE + 0x900),
            (DT_VERNEEDNUM, 1),
        ]
        .into_iter()
        .enumerate()
        {
            tag(&mut b, i, t, v);
        }
        // DT_VERSYM, deliberately written without any section headers.
        tag(&mut b, 5, 0x6fff_fff0, BASE + 0x800);
        put32(&mut b, 0x518, 1);
        b[0x51c] = 0x12;
        put32(&mut b, 0x700, 1);
        put32(&mut b, 0x704, 2);
        put32(&mut b, 0x708, 1);
        put16(&mut b, 0x802, 2);
        put16(&mut b, 0x900, 1);
        put16(&mut b, 0x902, 1);
        put32(&mut b, 0x904, 3);
        put32(&mut b, 0x908, 16);
        put32(&mut b, 0x910, hash_name("GCC_4.2.0", false));
        put16(&mut b, 0x916, 2);
        put32(&mut b, 0x918, 17);
        b
    }
    #[test]
    fn sectionless_sysv_symbol_version_owner_join() {
        let s = inspect(&fixture()).unwrap();
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].name, "f");
        assert_eq!(s[0].owner.as_deref(), Some("libgcc_s.so.1"));
        assert_eq!(s[0].version.as_deref(), Some("GCC_4.2.0"));
    }
    #[test]
    fn symbol_weakness_is_independent_of_version_weakness() {
        let mut b = fixture();
        b[0x51c] = 0x22;
        let s = inspect(&b).unwrap();
        assert_eq!(s[0].binding, 2);
        assert!(!s[0].version_weak);
        b[0x51c] = 0x12;
        put16(&mut b, 0x914, 2);
        let s = inspect(&b).unwrap();
        assert_eq!(s[0].binding, 1);
        assert!(s[0].version_weak);
    }
    #[test]
    fn optional_failure_does_not_regress_legacy_metadata() {
        let mut b = fixture();
        tag(&mut b, 4, 21, 0);
        assert!(super::super::inspect(&b).is_ok());
        assert!(inspect(&b).unwrap_err().contains("no supported hash"));
    }
    #[test]
    fn invalid_symbol_hash_string_and_index_refuse_without_panicking() {
        for case in 0..6 {
            let mut b = fixture();
            match case {
                0 => put32(&mut b, 0x704, u32::MAX),
                1 => put32(&mut b, 0x710, 1),
                2 => put32(&mut b, 0x518, u32::MAX),
                3 => put16(&mut b, 0x802, 99),
                4 => put32(&mut b, 0x708, 99),
                _ => tag(&mut b, 2, 6, u64::MAX - 7),
            }
            assert!(inspect(&b).is_err(), "case {case}");
        }
    }

    #[test]
    fn malformed_version_hash_and_section_count_are_unresolved() {
        let mut b = fixture();
        put32(&mut b, 0x910, 0);
        assert!(super::super::inspect(&b).is_ok());
        assert!(inspect(&b).unwrap_err().contains("hash disagrees"));
        let mut b = fixture();
        put64(&mut b, 40, 0xc00);
        put16(&mut b, 58, 64);
        put16(&mut b, 60, 1);
        put32(&mut b, 0xc04, 11);
        put64(&mut b, 0xc10, BASE + 0x500);
        put64(&mut b, 0xc18, 0x500);
        put64(&mut b, 0xc20, 72);
        put64(&mut b, 0xc38, 24);
        assert!(inspect(&b)
            .unwrap_err()
            .contains("counts or mappings disagree"));
    }

    #[test]
    fn hash_count_cannot_hide_a_relocated_import() {
        let mut b = fixture();
        put64(&mut b, 152, 12 * 16);
        put64(&mut b, 160, 12 * 16);
        tag(&mut b, 8, 7, BASE + 0xb00);
        tag(&mut b, 9, 8, 24);
        tag(&mut b, 10, 9, 24);
        tag(&mut b, 11, 0, 0);
        put64(&mut b, 0xb08, (1_u64 << 32) | 7);
        assert_eq!(inspect(&b).unwrap().len(), 1);
        put64(&mut b, 0xb08, (2_u64 << 32) | 7);
        assert!(inspect(&b)
            .unwrap_err()
            .contains("outside the declared hash inventory"));
    }

    #[test]
    fn gnu_hash_count_and_name_bloom_validation() {
        let mut b = fixture();
        tag(&mut b, 4, 0x6fff_fef5, BASE + 0x700);
        b[0x700..0x740].fill(0);
        put32(&mut b, 0x700, 1);
        put32(&mut b, 0x704, 1);
        put32(&mut b, 0x708, 1);
        put32(&mut b, 0x70c, 5);
        let h = hash_name("f", true);
        put64(
            &mut b,
            0x710,
            (1_u64 << (h % 64)) | (1_u64 << ((h >> 5) % 64)),
        );
        put32(&mut b, 0x718, 1);
        put32(&mut b, 0x71c, h | 1);
        assert_eq!(inspect(&b).unwrap().len(), 1);
        put64(&mut b, 0x710, 0);
        assert!(inspect(&b).unwrap_err().contains("lookup metadata"));
    }
}

use std::{io::Write, path::Path};

use crate::{
    parser::{Entry, EntryType},
    Result,
};

use fs_err as fs;

pub struct Writer;

impl Writer {
    pub fn write_entry(path: &Path, entry: Entry) -> Result<()> {
        let mut file = fs::OpenOptions::new().append(true).open(path)?;

        let kind = match entry.kind {
            EntryType::Debit => "-",
            EntryType::Credit => "+",
        };

        writeln!(
            file,
            "{d} {t} {a} {D}",
            d = entry.day,
            t = kind,
            a = entry.amount,
            D = entry.description
        )?;

        println!("Updated {}", path.display());

        Ok(())
    }
}

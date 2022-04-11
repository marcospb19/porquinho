use std::{
    ops::Not,
    path::{Path, PathBuf},
};

use chrono::{Datelike, Local};
use directories::ProjectDirs;
use fs_err as fs;

use crate::{Error, Result};

pub struct Dirs {
    inner: ProjectDirs,
}

impl Dirs {
    pub fn init() -> Result<Self> {
        let inner =
            ProjectDirs::from("com", "vrmiguel", "porquinho").ok_or(Error::NoValidHomeDirFound)?;

        let this = Self { inner };

        // this.create_dir_if_not_existent(this.config())?;
        this.create_dir_if_not_existent(this.path())?;

        Ok(this)
    }

    fn create_dir_if_not_existent(&self, path: &Path) -> Result<()> {
        if path.exists().not() {
            fs::create_dir_all(path)
                .map_err(|_| Error::CouldNotCreateFolder(PathBuf::from(path)))?;
            println!("info: created folder {:?}", path);
        }

        Ok(())
    }

    // pub fn config(&self) -> &Path {
    //     self.inner.config_dir()
    // }

    pub fn path(&self) -> &Path {
        self.inner.data_dir()
    }
}

/// The bookkeeping file for this month
/// E.g. if we're in October of 2024, the relevant file in which
/// we'll record income and expenses is `10-2024`
pub fn current_file() -> PathBuf {
    let today = Local::today();
    let month = today.month();
    let year = today.year();

    format!("{month:02}-{year}").into()
}

pub fn create_file_if_not_existent(path: &Path) -> Result<()> {
    if path.exists() {
        Ok(())
    } else {
        println!("Creating {}.", path.display());

        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map(|_| ())
            .map_err(Into::into)
    }
}

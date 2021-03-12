#![deny(clippy::all)]

#[macro_use]
mod utils;

mod cgroup_v1;
mod mount;
mod pipe;
mod proc;
mod run;
mod signal;

pub use crate::run::run;

use crate::utils::RawFd;

use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

use anyhow::Result;
use clap::Clap;
use memchr::memchr;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clap)]
#[clap(setting(clap::AppSettings::DeriveDisplayOrder))]
pub struct SandboxConfig {
    pub bin: PathBuf, // relative to chroot

    pub args: Vec<OsString>,

    #[clap(short = 'e', long)]
    pub env: Vec<OsString>,

    #[clap(short = 'c', long, value_name = "path")]
    pub chroot: Option<PathBuf>, // relative to cwd

    #[clap(long)]
    pub uid: Option<u32>,

    #[clap(long)]
    pub gid: Option<u32>,

    #[clap(long, value_name = "path")]
    pub stdin: Option<PathBuf>, // relative to cwd

    #[clap(long, value_name = "path")]
    pub stdout: Option<PathBuf>, // relative to cwd

    #[clap(long, value_name = "path")]
    pub stderr: Option<PathBuf>, // relative to cwd

    #[clap(long, value_name = "fd", conflicts_with = "stdin")]
    pub stdin_fd: Option<RawFd>,

    #[clap(long, value_name = "fd", conflicts_with = "stdout")]
    pub stdout_fd: Option<RawFd>,

    #[clap(long, value_name = "fd", conflicts_with = "stderr")]
    pub stderr_fd: Option<RawFd>,

    #[clap(short = 't', long, value_name = "milliseconds")]
    pub real_time_limit: Option<u64>,

    #[clap(long, value_name = "seconds")]
    pub rlimit_cpu: Option<u32>,

    #[clap(long, value_name = "bytes")]
    pub rlimit_as: Option<u64>,

    #[clap(long, value_name = "bytes")]
    pub rlimit_data: Option<u64>,

    #[clap(long, value_name = "bytes")]
    pub rlimit_fsize: Option<u64>,

    #[clap(long, value_name = "bytes")]
    pub cg_limit_memory: Option<u64>,

    #[clap(long, value_name = "count")]
    pub cg_limit_max_pids: Option<u32>,

    #[clap(
        long,
        value_name = "bindmount",
        parse(try_from_os_str = BindMount::try_from_os_str)
    )]
    pub bindmount_rw: Vec<BindMount>,

    #[clap(
        short = 'b',
        long,
        value_name = "bindmount",
        parse(try_from_os_str = BindMount::try_from_os_str)
    )]
    pub bindmount_ro: Vec<BindMount>,

    #[clap(
        long,
        value_name = "path",
        min_values = 0,
        require_equals = true,
        default_missing_value = "/proc"
    )]
    pub mount_proc: Option<PathBuf>, // absolute (affected by chroot)

    #[clap(
        long,
        value_name = "path",
        min_values = 0,
        require_equals = true,
        default_missing_value = "/tmp"
    )]
    pub mount_tmpfs: Option<PathBuf>, // absolute (affected by chroot)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BindMount {
    pub src: PathBuf, // absolute
    pub dst: PathBuf, // absolute (affected by chroot)
}

impl BindMount {
    fn try_from_os_str(s: &OsStr) -> Result<Self, String> {
        let (src, dst) = match memchr(b':', s.as_bytes()) {
            Some(idx) => {
                let src = OsStr::from_bytes(&s.as_bytes()[..idx]);
                let dst = OsStr::from_bytes(&s.as_bytes()[idx + 1..]);
                if src.is_empty() || dst.is_empty() {
                    return Err("invalid bind mount format".into());
                }
                (src, dst)
            }
            None => (s, s),
        };
        Ok(BindMount {
            src: src.into(),
            dst: dst.into(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SandboxOutput {
    pub code: i32,
    pub signal: i32,

    pub real_time: u64, // milliseconds
    pub sys_time: u64,  // milliseconds
    pub user_time: u64, // milliseconds

    pub memory: u64, // KiB
}

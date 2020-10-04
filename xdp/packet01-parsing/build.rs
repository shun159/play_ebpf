/* SPDX-License-Identifier: GPL-2.0 */

use cargo_bpf_lib as cargo_bpf;
use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let cargo = PathBuf::from(env::var("CARGO").unwrap());
    let target = PathBuf::from(env::var("OUT_DIR").unwrap());
    let probes = Path::new("parser");

    cargo_bpf::build(&cargo, &probes, &target.join("target"), Vec::new())
        .expect("couldn't compile probes");

    cargo_bpf::probe_files(&probes)
        .expect("couldn't list probe files")
        .iter()
        .for_each(|file| {
            println!("cargo:rerun-if-changed={}", file);
        });
}

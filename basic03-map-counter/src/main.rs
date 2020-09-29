/* SPDX-License-Identifier: GPL-2.0 */

extern crate libc;

mod options;

use redbpf::{load::Loaded, load::Loader, xdp, ArrayMap};
use probes::kern::DataRec;
use options::parse;
use std::{process, io};
use std::mem::MaybeUninit;
use tokio::runtime::Runtime;
use tokio::time::{delay_for, Duration};
use tokio::signal;
use libc::{c_int, timespec};

const MAP_NAME: &'static str = "xdp_stats_map";
const XDP_PASS: u32 = 2;
const STATS_INTERVAL: Duration = Duration::from_secs(2); // Poll every 2 seconds
const CLOCK_MONOTONIC: c_int = 1;
const NANOSEC_PER_SEC: u64 = 1000000000; // 10^9

extern "C" {
    pub fn clock_gettime(
        id: c_int,
        ts: *mut timespec
    ) -> c_int;
}

#[derive(Debug, Clone, Default)]
struct Record {
    timestamp: u64,
    total: DataRec
}

#[derive(Debug, Clone, Default)]
struct StatsRecord {
    stats: Record
}

fn main() {
    let opts = parameters();
    let mut runtime = Runtime::new().unwrap();
    let _ = runtime.block_on(do_main(opts));
}

async fn do_main(opts: options::Opts) {
    // Load the program and the map
    let mut loader: Loaded = Loader::load(probe_code()).expect("error loading probe");
    for prog in loader.xdps_mut() {
        prog.attach_xdp(&opts.interface, xdp::Flags::default())
            .expect("error attaching XDP program")
    }
    tokio::spawn(stats_poll(loader));
    signal::ctrl_c().await.unwrap();
}

async fn stats_poll(loader: Loaded) {
    let map = loader.map(MAP_NAME).unwrap();
    let hmap = &ArrayMap::<u32, DataRec>::new(map).unwrap();
    let record = &mut StatsRecord::default();

    /* Get initial reading quickly */
    stats_collect(hmap, record);
    delay_for(STATS_INTERVAL).await;

    loop {
        let prev = &mut record.clone(); // Sturct copy
        stats_collect(hmap, record);
        stats_print(record, prev);
        delay_for(STATS_INTERVAL).await;
    }
}

fn stats_print(rec_curr: &mut StatsRecord, rec_prev: &mut StatsRecord) {
    let curr: &Record = &rec_curr.stats;
    let prev: &Record = &rec_prev.stats;
    let period = calc_period(curr, prev);
    if period > 0.0 {
        let rx_bytes_in_kb = (curr.total.rx_bytes as f64) / 1000.0;
        let packets: u64 = curr.total.rx_packets - prev.total.rx_packets;
        let bytes: f64 = (curr.total.rx_bytes - prev.total.rx_bytes) as f64;
        let pps: u64 = packets / period as u64;
        let mbps: f32 = (((bytes * 8.0) / period) / 1_000_000.0) as f32;
        println!(
            "xdp_pass: {0:>12} pkts ({1:>10} pps) {2:>11} Kbytes ({3:>6.4} Mbits/s)  period: {4:>10}",
            curr.total.rx_packets,
            pps,
            rx_bytes_in_kb,
            mbps,
            period
        );
    }
}

fn stats_collect(hmap: &ArrayMap<u32, DataRec>, stats_rec: &mut StatsRecord) {
    map_collect(hmap, &mut stats_rec.stats)
}

fn calc_period(curr: &Record, prev: &Record) -> f64 {
    let mut period = (curr.timestamp - prev.timestamp) as f64;
    if period > 0.0 { period /= NANOSEC_PER_SEC as f64; }
    period
}

fn map_collect(hmap: &ArrayMap<u32, DataRec>, rec: &mut Record) {
    rec.timestamp = gettime();
    let value = hmap.get(XDP_PASS).unwrap();
    rec.total.rx_packets = value.rx_packets;
    rec.total.rx_bytes = value.rx_bytes;
}

fn gettime() -> u64 {
    unsafe {
        let mut tp = MaybeUninit::<timespec>::zeroed();
        if clock_gettime(CLOCK_MONOTONIC, &mut tp as *mut _ as *mut _) < 0 {
            println!("Error with gettimeofday! {}", io::Error::last_os_error());
            process::exit(1);
        } else {
            let tp = tp.assume_init();
            (tp.tv_sec as u64) * NANOSEC_PER_SEC + (tp.tv_nsec as u64)
        }
    }
}

fn probe_code() -> &'static [u8] {
    include_bytes!(concat!(
        env!("OUT_DIR"),
        "/target/bpf/programs/map-counter/map-counter.elf"
    ))
}

fn parameters() -> options::Opts {
    match parse() {
        Some(o) => o,
        None => process::exit(1),
    }
}

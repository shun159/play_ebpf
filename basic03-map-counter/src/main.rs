/* SPDX-License-Identifier: GPL-2.0 */

extern crate libc;

mod options;

use redbpf::{load::Loaded, load::Loader, xdp, HashMap};
use probes::kern::DataRec;
use options::parse;
use std::{process, mem, io};
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

#[derive(Debug, Clone)]
struct Record {
    timestamp: u64,
    total: DataRec
}

impl Default for Record {
    fn default() -> Self {
        Record {
            timestamp: 0,
            total: DataRec { rx_packets: 0 }
        }
    }
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
    let hmap = &HashMap::<u32, DataRec>::new(map).unwrap();

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
    if period > 0 {
        let packets: u64 = curr.total.rx_packets - prev.total.rx_packets;
        let pps: u64 = packets / period;
        println!("xdp_pass: {0:} pkts ({1:<010} pps) period: {2:}", packets, pps, period);
    }
}

fn stats_collect(hmap: &HashMap<u32, DataRec>, stats_rec: &mut StatsRecord) {
    map_collect(hmap, &mut stats_rec.stats)
}

fn calc_period(curr: &Record, prev: &Record) -> u64 {
    let mut period: u64 = curr.timestamp - prev.timestamp;
    if period > 0 { period /= NANOSEC_PER_SEC; }
    period
}

fn map_collect(hmap: &HashMap<u32, DataRec>, rec: &mut Record) {
    rec.timestamp = gettime();
    let value = match hmap.get(XDP_PASS) {
        Some(value) =>
            value,
        None => {
            hmap.set(XDP_PASS, DataRec { rx_packets: 0 });
            DataRec { rx_packets: 0 }
        }
    };
    rec.total.rx_packets = value.rx_packets;
}

#[allow(deprecated)]
fn gettime() -> u64 {
    unsafe {
        let mut tp = mem::uninitialized();
        if clock_gettime(CLOCK_MONOTONIC, &mut tp) < 0 {
            println!("Error with gettimeofday! {}", io::Error::last_os_error());
            process::exit(1);
        } else {
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

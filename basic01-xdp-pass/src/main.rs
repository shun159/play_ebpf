/* SPDX-License-Identifier: GPL-2.0 */

mod options;

use redbpf_probes::bindings::iphdr;
use futures::stream::StreamExt;
use options::parse;
use redbpf::{load::Loaded, load::Loader, xdp};
use std::net::Ipv4Addr;
use std::env;
use std::process;
use std::ptr;
use tokio;
use tokio::runtime::Runtime;
use tokio::signal;

fn main() {
    let opts = parameters();
    let mut runtime = Runtime::new().unwrap();
    let _ = runtime.block_on(event_handler(opts));
}

async fn event_handler(opts: options::Opts) {
    let loader = xdp_load(opts);
    tokio::spawn(handle_event(loader));
    signal::ctrl_c().await.unwrap();
}

async fn handle_event(mut loader: Loaded) {
    while let Some((name, events)) = loader.events.next().await {
        for event in events {
            match name.as_str() {
                "events" => {
                    let pkt_ev = unsafe { ptr::read(event.as_ptr() as *const iphdr) };
                    process_event(pkt_ev)
                }
                _ =>
                    panic!("unexpected event"),
            }
        }
    }
}

fn process_event(ev: iphdr) {
    let saddr =  Ipv4Addr::from(u32::from_be(ev.saddr));
    let daddr =  Ipv4Addr::from(u32::from_be(ev.daddr));
    println!("packet passed: src: {} -> dst: {} protocol: {}", saddr, daddr, ev.protocol);
}

fn xdp_load(opts: options::Opts) -> Loaded {
    let mut loader: Loaded = Loader::load(probe_code()).expect("error loading probe");
    loader.map("events").unwrap();
    for prog in loader.xdps_mut() {
        prog.attach_xdp(&opts.interface, xdp::Flags::default())
            .expect("error attaching XDP program")
    }

    loader
}

fn probe_code() -> &'static [u8] {
    include_bytes!(concat!(
        env!("OUT_DIR"),
        "/target/bpf/programs/xdp_pass/xdp_pass.elf"
    ))
}

fn parameters() -> options::Opts {
    match parse() {
        Some(o) => o,
        None => process::exit(1),
    }
}


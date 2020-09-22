/* SPDX-License-Identifier: GPL-2.0 */

#![no_std]
#![no_main]

use probes::kern::DataRec;
use redbpf_probes::xdp::prelude::*;

program!(0xFFFFFFFE, "GPL");

static XDP_PASS: u32 = 2;

#[map("xdp_stats_map")]
static mut hmap: HashMap<u32, DataRec> = HashMap::with_max_entries(10);

#[xdp("xdp_stats1")]
pub fn xdp_stats1_func(ctx: XdpContext) -> XdpResult {
    /* Lookup in kernel BPF-side return pointer to actual data record */
    let mut rec = find_datarec(XDP_PASS);
    rec.rx_packets += 1;
    return Ok(XdpAction::Pass)
}

#[inline]
fn find_datarec(key: u32) -> &'static mut DataRec {
    unsafe {
        match hmap.get_mut(&key) {
            Some(rec) => rec,
            None => {
                let rec = DataRec { rx_packets: 0 };
                hmap.set(&key, &rec);
                hmap.get_mut(&key).unwrap()
            }
        }
    }
}

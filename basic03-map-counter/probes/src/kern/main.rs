/* SPDX-License-Identifier: GPL-2.0 */

#![no_std]
#![no_main]

use probes::kern::DataRec;
use redbpf_probes::xdp::prelude::*;

program!(0xFFFFFFFE, "GPL");

static XDP_PASS: u32 = 2;

#[map("xdp_stats_map")]
static mut hmap: ArrayMap<u32, DataRec> = ArrayMap::with_max_entries(4);

#[xdp("xdp_stats1")]
pub fn xdp_stats1_func(ctx: XdpContext) -> XdpResult {
    /* Lookup in kernel BPF-side return pointer to actual data record */
    let key = XDP_PASS;
    unsafe {
        match hmap.get_mut(&key) {
            None =>
                Ok(XdpAction::Aborted),
            Some(mut rec) => {
                rec.rx_bytes += ctx.len();
                rec.rx_packets += 1;
                Ok(XdpAction::Pass)
            }
        }
    }
}

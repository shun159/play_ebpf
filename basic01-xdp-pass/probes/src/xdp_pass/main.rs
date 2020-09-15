/* SPDX-License-Identifier: GPL-2.0 */

#![no_std]
#![no_main]

use redbpf_probes::bindings::iphdr;
use redbpf_probes::xdp::prelude::*;

program!(0xFFFFFFFE, "GPL");

#[map("events")]
static mut events: PerfMap<iphdr> = PerfMap::with_max_entries(10);

#[xdp("xdp_pass")]
pub fn probe(ctx: XdpContext) -> XdpResult {
    unsafe {
        let ip_h = *ctx.ip()?;
        events.insert(&ctx, &MapData::new(ip_h));
    };

    Ok(XdpAction::Pass)
}

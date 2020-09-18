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
    unsafe {
        let rec = hmap.get_mut(&XDP_PASS).ok_or(NetworkError::Other)?;
        lock_xadd(&mut rec.rx_packets, 1);
    }

    Ok(XdpAction::Pass)
}

// LLVM maps __sync_fetch_and_add() as a built-in function to the BPF atomic add
// instruction (that is BPF_STX | BPF_XADD | BPF_W for word sizes)
fn lock_xadd(val: &mut u64, incr: u64) {
    unsafe {
        let ptr: *mut u64 = val;
        __sync_fetch_and_add(ptr, incr);
    }
}

extern "C" {
    pub fn __sync_fetch_and_add(
        ptr: *mut u64,
        val: u64
    ) -> i32;
}


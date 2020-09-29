/* SPDX-License-Identifier: GPL-2.0 */

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct DataRec {
    pub rx_packets: u64,
    pub rx_bytes: usize
}

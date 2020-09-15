/* SPDX-License-Identifier: GPL-2.0 */

pub struct PacketEvent {
    pub tos: u8,
    pub tot_len: u16,
    pub id: u16,
    pub frag_off: u16,
    pub ttl: u8,
    pub protocol: u8,
    pub checksum: u16,
    pub saddr: u32,
    pub daddr: u32
}

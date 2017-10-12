#![feature(global_allocator)]

#![no_std]

#[macro_use]
extern crate uavcan;
extern crate bit_field;
extern crate uavcan_s32k;
extern crate s32k144;
extern crate s32k144evb;
extern crate embedded_types;
extern crate alloc_cortex_m;

use alloc_cortex_m::CortexMHeap;

#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

extern "C" {
    static mut _sheap: u32;
    static mut _eheap: u32;
}

use s32k144evb::{
    wdog,
};

use s32k144evb::can::{
    CanSettings,
};

use uavcan::types::*;
use uavcan::{
    PrimitiveType,
    Frame,
    MessageFrameHeader,
    Header,
};

use uavcan::transfer::TransferInterface;
use uavcan::transfer::FullTransferID;
use uavcan::transfer::TransferID;

use uavcan::frame_disassembler::FrameDisassembler;
use uavcan::frame_assembler::{
    FrameAssembler,
    AssemblerResult,
};

use bit_field::BitField;

use uavcan_s32k::Interface;

#[derive(Debug, UavcanStruct, Default)]
struct NodeStatus {
    uptime_sec: Uint32,
    health: Uint2,
    mode: Uint3,
    sub_mode: Uint3,
    vendor_specific_status_code: Uint16,
}
message_frame_header!(NodeStatusHeader, 341);
uavcan_frame!(derive(Debug), NodeStatusFrame, NodeStatusHeader, NodeStatus, 0);

fn main() {

    let mut wdog_settings = wdog::WatchdogSettings::default();
    wdog_settings.enable = false;
    wdog::configure(wdog_settings);
    
    s32k144evb::serial::init();

    let start = unsafe { &mut _sheap as *mut u32 as usize };
    let end = unsafe { &mut _sheap as *mut u32 as usize };
    unsafe { ALLOCATOR.init(start, end - start) }

    let peripherals = unsafe{ s32k144::Peripherals::all() };

    let mut can_settings = CanSettings::default();    
    can_settings.source_frequency = 8000000;
    can_settings.self_reception = false;
    
    let scg = peripherals.SCG;
    let porte = peripherals.PORTE;
    let pcc = peripherals.PCC;
        
    scg.sosccfg.modify(|_, w| w
                       .range()._11()
                       .hgo()._1()
                       .erefs()._1()
    );
    
    scg.soscdiv.modify(|_, w| w
                       .soscdiv2().bits(0b001)
    );
    
    scg.sosccsr.modify(|_, w| w.soscen()._1());
    
    // Configure the can i/o pins
    pcc.pcc_porte.modify(|_, w| w.cgc()._1());
    porte.pcr4.modify(|_, w| w.mux()._101());
    porte.pcr5.modify(|_, w| w.mux()._101());
    
    pcc.pcc_flex_can0.modify(|_, w| w.cgc()._1());
    

    let can_interface = s32k144evb::can::Can::init(peripherals.CAN0, &can_settings).unwrap();
    let uavcan_interface = Interface::new(&can_interface);

    let loop_max = 100000;

    loop {
        let uavcan_frame = NodeStatusFrame::from_parts(
            NodeStatusHeader::new(0, 32),
            NodeStatus{
                uptime_sec: 0.into(),
                health: 0.into(),
                mode: 0.into(),
                sub_mode: 0.into(),
                vendor_specific_status_code: 0.into(),
            }
        );

        let mut generator = FrameDisassembler::from_uavcan_frame(uavcan_frame, 0.into());
        let can_frame = generator.next_transfer_frame::<embedded_types::can::ExtendedDataFrame>().unwrap();
                
        for i in 0..loop_max {
            if i == 0 {
                uavcan_interface.transmit(&can_frame).unwrap();
            }
        }
        
        
    }
}


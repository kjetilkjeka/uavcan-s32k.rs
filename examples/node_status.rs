#![no_std]

#[macro_use]
extern crate uavcan;
extern crate dsdl;

extern crate uavcan_s32k;
extern crate s32k144;
#[macro_use]
extern crate s32k144evb;
extern crate embedded_types;

use embedded_types::io::Write;

use s32k144evb::{
    wdog,
    spc,
};

use s32k144evb::can::{
    CanSettings,
};

use uavcan::types::*;
use uavcan::{
    Message,
    NodeID,
    NodeConfig,
    SimpleNode,
    Node,
};

use uavcan::transfer::TransferInterface;
use uavcan::transfer::TransferID;

use uavcan_s32k::Interface;

fn main() {
    
    let mut wdog_settings = wdog::WatchdogSettings::default();
    wdog_settings.enable = false;
    wdog::configure(wdog_settings).unwrap();    

    let peripherals = unsafe{ s32k144::Peripherals::all() };

    let pc_config = spc::Config{
        system_oscillator: spc::SystemOscillatorInput::Crystal(8_000_000),
        soscdiv2: spc::SystemOscillatorOutput::Div1,
        .. Default::default()
    };
    
    let spc = spc::Spc::init(
        peripherals.SCG,
        peripherals.SMC,
        peripherals.PMC,
        pc_config
    ).unwrap();
    
    let mut console = s32k144evb::console::LpuartConsole::init(peripherals.LPUART1, &spc);

    let mut can_settings = CanSettings::default();    
    can_settings.self_reception = false;
    
    let porte = peripherals.PORTE;
    let pcc = peripherals.PCC;
            
    // Configure the can i/o pins
    pcc.pcc_porte.modify(|_, w| w.cgc()._1());
    porte.pcr4.modify(|_, w| w.mux()._101());
    porte.pcr5.modify(|_, w| w.mux()._101());
    
    pcc.pcc_flex_can0.modify(|_, w| w.cgc()._1());
    
    let uavcan_interface = Interface::new(peripherals.CAN0, &spc);

    let node_config = NodeConfig{id: Some(NodeID::new(32))};
    let node = SimpleNode::new(uavcan_interface, node_config);


    let loop_max = 5000;

    
    loop {
        let uavcan_frame = dsdl::uavcan::protocol::NodeStatus {
            uptime_sec: 0,
            health: u2::new(0),
            mode: u3::new(0),
            sub_mode: u3::new(0),
            vendor_specific_status_code: 0,
        };

        
        for i in 0..loop_max {
            if i == 0 {
                node.transmit_message(uavcan_frame.clone());
            }
            
            if let Ok(message) = node.receive_message::<dsdl::uavcan::protocol::NodeStatus>() {
                writeln!(console, "Received node status frame: {:?}",  message);
            }
            
        }
    }
}


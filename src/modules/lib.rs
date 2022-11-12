use std::fs::File;
use std::{fs, io};
use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};
use chrono::{DateTime, NaiveDateTime, Utc};
use chrono::format::{DelayedFormat, StrftimeItems};
use etherparse::{IpHeader, PacketHeaders, TransportHeader};
use etherparse::IpHeader::Version4;
use pcap::{Active, Capture, Device, Packet};
use crate::modules::aggregator::Aggregator;
use crate::modules::parsedpacket::ParsedPacket;


//used only for debugging
pub fn select_debug() -> Capture<Active> {
    let cap = Capture::from_device("\\Device\\NPF_{DFADCF5E-E518-4EB5-A225-3126223CB9A2}").unwrap()
        .promisc(true)
        .open().unwrap();
    return cap;
}

pub fn select_default() -> Capture<Active> {
    let main_device = Device::lookup().expect("lookup error").expect("No default device found");
    let cap = Capture::from_device(main_device).unwrap()
        .promisc(true)
        .open().unwrap();
    return cap;
}

pub fn select_device() -> Capture<Active> {
    let mut i=0;
    // list all of the devices pcap tells us are available
    let dev_list= Device::list().expect("device lookup failed");
    for device in &dev_list {
        i+=1;
        println!("{})  {:?}",i, device.desc.as_ref().unwrap());
    }
    let mut input_line = String::new();
    io::stdin()
        .read_line(&mut input_line)
        .expect("Failed to read line");
    let mut number: usize = input_line.trim().parse().expect("Input not an integer");
    number-=1;
    let device = dev_list[number].clone();
    println!("Selected {:?}",device.desc.as_ref().unwrap());
    let cap = Capture::from_device(device).unwrap()
        .promisc(true)
        .open().unwrap();
    return cap;
}

pub fn parse_packet(packet:Packet) -> Option<ParsedPacket> {
    let ph=PacketHeaders::from_ethernet_slice(&packet).unwrap();
    let mut source=String::new();
    let mut destination=String::new();
    let mut size = 0;
    let mut ts=String::new();
    let mut trs_protocol =String::new();
    let mut src_port =0;
    let mut dest_port =0;
    let mut show=(false);
    match ph.ip {
        Some(x)=> match x {
            Version4(h,_)=> {
                println!("V4");
                let mut s=h.source.into_iter().map(|i| i.to_string() + ".").collect::<String>();
                s.pop();
                source=s;
                let mut d=h.destination.into_iter().map(|i| i.to_string() + ".").collect::<String>();
                d.pop();
                destination=d;
                size=packet.header.len as usize;
                let time_number=packet.header.ts.tv_sec as i64;
                let nt = NaiveDateTime::from_timestamp(time_number, 0);
                let dt: DateTime<Utc> = DateTime::from_utc(nt, Utc);
                ts = dt.format("%Y-%m-%d %H:%M:%S").to_string();
            },
            IpHeader::Version6(h, _) => {
                println!("IP VERSION 6");
                let mut s=h.source.into_iter().map(|i| i.to_string() + ".").collect::<String>();
                s.pop();
                source=s;
                let mut d=h.destination.into_iter().map(|i| i.to_string() + ".").collect::<String>();
                d.pop();
                destination=d;
                size=packet.header.len as usize;
                let time_number=packet.header.ts.tv_sec as i64;
                let nt = NaiveDateTime::from_timestamp(time_number, 0);
                let dt: DateTime<Utc> = DateTime::from_utc(nt, Utc);
                ts = dt.format("%Y-%m-%d %H:%M:%S").to_string();
                }
        },
        None => {}
    }
    match  ph.transport {
        Some(x)=> match x {
            TransportHeader::Udp(y) => {trs_protocol=String::from("Udp");src_port=y.source_port as usize;dest_port=y.destination_port as usize;show=true}
            TransportHeader::Tcp(y) => {trs_protocol=String::from("Tcp");src_port=y.source_port as usize;dest_port=y.destination_port as usize;show=true}
            _ => {}
        },
        _ => {}
    }
    if show
    {
        let parsed_p= ParsedPacket::new(ts, source, destination, src_port, dest_port, trs_protocol, size);
        //println!("{:?}", parsed_p);
        return Some(parsed_p);
    }
    None
}

pub fn print_packets(mut cap:Capture<Active>){
    while let Ok(packet) = cap.next_packet() {
        let p=parse_packet(packet);
        match p {
            None => {}
            Some(x) => {println!("{:?}",x);}
        }
    }
}

//used only for debugging
pub fn send_to_aggregator(mut cap:Capture<Active>){
    let agg=Aggregator::new();

    while let Ok(packet) = cap.next_packet() {
        let p=parse_packet(packet);
        match p {
            None => {}
            Some(x) => {agg.send(x)}
        }
    }
}

pub fn create_dir_report(filename:&str) -> BufWriter<File> {
    let res_dir=fs::create_dir("report");
    match res_dir {
        Ok(_) => {}
        Err(_) => {}
    }
    let mut path =String::from("report/");
    path.push_str(filename);
    path.push_str(".txt");
    let input=File::create(path.as_str()).expect("Error creating output file\n\r");
    let output = BufWriter::new(input);
    return output;

}
//, aggregated_data: Arc<Mutex<HashMap<(String,usize),(String,usize,usize,usize)>>>
pub fn write_report(filename:&str,aggregated_data: Arc<Mutex<HashMap<(String, usize), (String, usize, String, String)>>>){
   let aggregated_data=aggregated_data.lock().unwrap();

    let mut output =create_dir_report(filename);
   // output.write_all(aggregated_data).unwrap();
    writeln!(output, "----------------------------------------------------------------------------------------------------------").expect("Error writing output file\n\r");
    writeln!(output, "|   Dst IP address  |  Dst port |  Protocol |    Bytes      |  Initial timestamp    |   Final timestamp  |").expect("Error writing output file\n\r");
    writeln!(output, "----------------------------------------------------------------------------------------------------------").expect("Error writing output file\n\r");

    for x in aggregated_data.iter(){
        let key=x.0;
        let value=x.1;
        let k1=key.0.clone();
        let k2=key.1;
        let val1=value.0.clone();
        let val2=value.1;
        let val3=value.2.clone();
        let val4=value.3.clone();
        writeln!(output, "| {0:<15} \t| {1:<5} \t| {2:<7} \t| {3:<9} \t| {4:<15} \t| {5:<3}| ",k1,k2,val1,val2,val3,val4).expect("Error writing output file\n\r");
    }


}
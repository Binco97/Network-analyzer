use std::io;
use crate::modules::report_writer::ReportWriter;
use crate::modules::socketlistener::SocketListener;

pub struct Analyzer{
    pub sl: SocketListener,
    pub report_writer: ReportWriter,
}

impl Analyzer{
    pub fn new(device:&str,filename:&str,timer:u64)->Self{
        let sl=SocketListener::new(device);
        let report_writer = ReportWriter::new(filename.to_string(), timer, sl.get_aggregated_data());
        Analyzer{sl,report_writer}
    }

    pub fn pause(&self){
        println!("choice 1: pausing SocketListener and report writer");
        self.sl.pause();
        self.report_writer.pause();
    }

    pub fn resume(&self){
        println!("choice 2: resuming SocketListener and report writer");
        self.sl.resume();
        self.report_writer.resume();
    }

    pub fn exit(&self){
        //TODO: implement exit also for SocketListener and its submodules (parser and aggregator)
        // such that they can be stopped gracefully (thread exits from the loop)
        self.sl.pause();
        self.report_writer.exit();
    }
}
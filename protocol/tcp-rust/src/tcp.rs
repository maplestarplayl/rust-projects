use etherparse::IpNumber;
use std::io;
pub enum State {
    CLOSED,
    LISTEN,
    SYNC_RECV,
    ESTABLISHED,
}
pub struct Connection {
    state: State,
    send_seq_vars: SendSeqVars,
    recv_seq_vars: RecvSeqVars,
}

/// Send Sequence Space
///
///                    1         2          3          4      
///               ----------|----------|----------|----------
///                      SND.UNA    SND.NXT    SND.UNA        
///                                           +SND.WND        
///
///         1 - old sequence numbers which have been acknowledged  
///         2 - sequence numbers of unacknowledged data            
///         3 - sequence numbers allowed for new data transmission
///         4 - future sequence numbers which are not yet allowed  

///                           Send Sequence Space
///
///                                Figure 4.
///
///   The send window is the portion of the sequence space labeled 3 in
///   figure 4.
struct SendSeqVars {
    ///send unacknowledged
    una: u32,
    ///send next
    nxt: u32,
    ///send window
    wnd: u16,
    ///send window lower bound 1
    wl1: usize,
    ///send window lower bound 2
    wl2: usize,
    ///send initial sequence number
    iss: u32,
    up: bool,
}
/// Receive Sequence Space
///
///                        1          2          3      
///                    ----------|----------|----------
///                           RCV.NXT    RCV.NXT        
///                                     +RCV.WND        
///
///         1 - old sequence numbers which have been acknowledged  
///         2 - sequence numbers allowed for new reception         
///         3 - future sequence numbers which are not yet allowed  
///
///                          Receive Sequence Space
///
///                                Figure 5.
///

///   The receive window is the portion of the sequence space labeled 2 in
struct RecvSeqVars {
    ///recv next
    nxt: u32,
    ///recv window
    wnd: u16,
    ///recv initial sequence number
    irs: u32,
    up: bool,
}
impl Connection {
    pub fn accept(
        tun: &mut tappers::macos::Utun,
        ip_header: etherparse::Ipv4HeaderSlice,
        tcp_header: etherparse::TcpHeaderSlice,
        data: &[u8],
    ) -> io::Result<Option<Self>> {
        println!("ready to accept");
        let mut buf = [0u8; 1500];
        let mut unwritten: &mut [u8] = &mut buf[..];
        if !tcp_header.syn() {
            //only expect sync
            println!("not syn");
            return Ok(None);
        }
        let iss = 0;
        let conn = Self {
            state: State::SYNC_RECV,
            send_seq_vars: SendSeqVars {
                una: iss,
                nxt: iss + 1,
                wnd: 10,
                wl1: 0,
                wl2: 0,
                iss,
                up: false,
            },
            recv_seq_vars: RecvSeqVars {
                nxt: tcp_header.sequence_number() + 1,
                wnd: tcp_header.window_size(),
                irs: tcp_header.sequence_number(),
                up: false,
            },
        };

        //send sync ack
        let mut syn_ack = etherparse::TcpHeader::new(
            tcp_header.destination_port(),
            tcp_header.source_port(),
            0,
            conn.send_seq_vars.wnd,
        );
        syn_ack.acknowledgment_number = tcp_header.sequence_number() + 1;
        syn_ack.syn = true;
        syn_ack.ack = true;

        //add ip header
        let ip = etherparse::Ipv4Header::new(
            syn_ack.header_len_u16(),
            64,
            IpNumber::TCP,
            [
                ip_header.destination()[0],
                ip_header.destination()[1],
                ip_header.destination()[2],
                ip_header.destination()[3],
            ],
            [
                ip_header.source()[0],
                ip_header.source()[1],
                ip_header.source()[2],
                ip_header.source()[3],
            ],
        )
        .expect("failed to create ip header");

        ip.write(&mut unwritten)?;
        syn_ack.write(&mut unwritten)?;
        let len = unwritten.len();
        
        let family_prefix =  buf[0] & 0xf0 ;
        // println!("{:02x?}", family_prefix);
        // println!("{:02x?}", buf[0]);
        // println!("{:02x?}", 0x54 & 0x0f);
        println!("{:02x?}", family_prefix);
        // buf[0] = 0x54;
        eprintln!("{:02x?}", ip_header);
        eprintln!("{:02x?}", tcp_header);
        eprintln!("{:02x?}", &buf[..buf.len() - len]);
        tun.send(&buf[..buf.len() - len]).expect("failed to send");
        eprintln!(
            "{}: {} -> {}:{} {}b len of tcp",
            ip_header.source_addr(),
            tcp_header.source_port(),
            ip_header.destination_addr(),
            tcp_header.destination_port(),
            data.len(),
        );
        Ok(Some(conn))
    }

    pub fn on_packet(
        &mut self,
        tun: &mut tappers::macos::Utun,
        ip_header: etherparse::Ipv4HeaderSlice,
        tcp_header: etherparse::TcpHeaderSlice,
        data: &[u8],
    ) -> io::Result<Option<Self>> {
        unimplemented!()
    }
}

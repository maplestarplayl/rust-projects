# TUN/TAP
- Use tappers lib to create a tun device
-- In macos, creation of tun device needs to be allocated with a destination address 
- Use etherparse to parse the received packet
- 
# TCP
THE TCP connection is basically a quad
- Each byte in Tcp has a sequence number

## TCP Connection
- Each Connection has a send sequence space and a receive sequence space 
## Handshake
- By handshake, the server and client exchange their initial sequence numbers(randomly generated)
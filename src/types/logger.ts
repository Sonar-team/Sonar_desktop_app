export interface DataLinkLayer {
    source_mac?: string;
    destination_mac?: string;
    ethertype?: number;
  }
  
  export interface InternetLayer {
    protocol_name?: string;
    source?: string;
    destination?: string;
  }
  
  export interface TransportLayer {
    protocol?: string;
    source_port?: number | string;
    destination_port?: number | string;
  }
  
  export interface ApplicationLayer {
    application_protocol?: string;
  }
  
  export interface PacketFlow {
    data_link?: DataLinkLayer;
    internet?: InternetLayer;
    transport?: TransportLayer;
    application?: ApplicationLayer;
  }
  
  export interface PacketMinimal {
    ts_sec: number;
    ts_usec: number;
    len: number;
    flow: PacketFlow | null;
    interface_name?: string;
  }
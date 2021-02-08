The metrics gatherer listens for connections from clients over WebSocket
and stores metrics, received in JSON format, into InfluxDB using UDP.

To make UDP performant, the configuration according official InfluxDB docs
should be performed: https://docs.influxdata.com/influxdb/v1.8/supported_protocols/udp/

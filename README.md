sheave - A streaming server implementaton for Rust
===

A streaming server which cloned [Red5](https://github.com/Red5/red5-server), written by Rust.

## Currently TODOs

* [ ] Handle all RTMP packet.
* [ ] Replace the Remote Procedure Call derived from ActionScript with HTML5 and ES2017. (If is possible, I'd like to write it by WebAssembly, not ES2017)
* [ ] Implement asynchronous communication by using [hyper](https://github.com/hyperium/hyper).
* [ ] Sign up a CI, then make it run jobs for tests and builds. 
* [ ] Sign up a VPS service, then measure the throughput and the responsivity in real envirionment.

## Goals

* [ ] Clone the Red5 completely.
* [ ] Support WebSocket and HLS.
  * [ ] Low Latency.
  * [ ] Handle HLS packets directly, converting no packets. (without frame loss and audio lag)
* [ ] Support WebRTC. (in the future)
  * [ ] For Chrome/Chromium.
  * [ ] For Firefox.
  * [ ] For other web browsers. (If I have the motivation...)

# 36C3 Led Stuff

Blinkin' stuff for the 36c3 rust assembly.
Built out of stuff i borrowed from other people/had lying around.

Has a display component with a nucle-g071rb & hub75, as well as multiple
trailing led strips, with:
- stm32f042 & sk6812w
- adafruit trinket m0 (atsamd21) with the onboard apa102 led and an apa102 strip
- microbit (nrf51) & ws2812
  (Doesn't work, it's a bit too slow currently)

This is coordinated using a simple serial line at 9600 baud, which just sends
the byte of the current frame (c3_host).

This probably won't be maintained in the future.

## Ideas
- Audio visualization, beat detection
- Image streaming over serial with dma


## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

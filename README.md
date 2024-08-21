# Warning

Make sure you have an antenna connected to the TX1 port that provides the proper load. 
While I did use a low gain setting it is still important for the test to work.

# About

This uses the `rust-bladerf` crate to initialize the first BladeRF board found. It then
creates a single tone frequency. It uses a thread to continually send this signal. While
that signal is being sent it receives data and correlates the known signal with the
recieved data. It then prints the magnitude of the correlation.

When the TX thread is running you should get a steady and high magnitude. If the TX
thread is not running you should get a low and changing value represenitng the fact
the signal is missing.

The actual magnitude depends on your antenna gain. I've set the TX gain low and the
RX gain high as default. You should review these, the frequency used, and make sure
you have the proper antennas or loads connected to the BladeRF!

# How

You should be able to clone or download this repository then use the command
`cargo run` to have the project built and ran. It should download the needed
dependencies.

To get `cargo` working visit https://www.rust-lang.org/tools/install.

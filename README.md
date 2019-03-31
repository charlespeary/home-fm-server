# home-fm-server


home-fm-server is actix powered backend for fm-transmitter running on Raspberry Pi.
# What does it do?
  - communicate with [home-fm-client] in order to schedule next songs to play
  - download songs from youtube and persists them in SQLite
  - encode songs into radio waves via [fm-transmitter] (soon to be rewritten in Rust)

# How to set it up on your RPi?
I will create a script to quickly install it after I'm done.

[home-fm-client]: <https://github.com/Sniadekk/home-fm-client>
[fm-transmitter]: <https://github.com/somu1795/fm_transmitter>
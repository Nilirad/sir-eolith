# sir-eolith

This is a reverse-engineering project I did back in the summer of 2020. I looked at the source code of the [slither.io] game, and I slowly built a pretty accurate client that connects to the game server, letting me play the game. The gameplay was pretty accurate.

At that time, Rust and its ecosystem became much more richer, so I decided to switch from C++ for my hobby projects.

When Bevy was announced, my original intention was to port the client from `ggez` and `legion` to Bevy. However, when I realized that Bevy didn't have a way to draw geometric shapes, I created  the plugin [`bevy_prototype_lyon`](https://github.com/rparrett/bevy_prototype_lyon). Many people appreciated the library, and I focused so much effort developing it that I neglected developing this client.

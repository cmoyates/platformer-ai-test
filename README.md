# Platformer AI Test

This is a test project I've put together to let others try interacting with an AI for a game I'm working on.

The behavior is supposed to resemble that of the lizards from [Rain World](https://store.steampowered.com/app/312520/Rain_World/). However, I have never played Rain World, so I'm just kind of guessing. In the game this AI will control a creature that hunts down the player.

## TODO

- [ ] Implement [platformer pathfinding](https://www.youtube.com/watch?v=kNI2I8kzpnE&t=123s)

  - [ ] Contruct node graph
    - [x] Place nodes in all valid positions
    - [x] Connect the nodes
    - [x] Remove duplicate nodes
    - [x] Fix the order
    - [ ] Jumpable stuff
  - [ ] A\*

- [x] Set up a way to set goal points
  - [x] Starts at (0,0), move with arrow keys
  - [x] Toggle goal enabled with space
- [x] Add button to reset the AI (R)
- [ ] Add procedural animation
  - [ ] Multiple body segments
  - [ ] [IK](https://youtu.be/wgpgNLEEpeY)

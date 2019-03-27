# Game Engine / 3D renderer experimentations


# Cartoon colors

https://www.schemecolor.com/powerpuff-girls-theme-colors.php

deep blue: 33ACE3
sky blue: 98DAF1
red: EA6964
pink: E78C89
green: 4AB62C
light green: 81CB71

# Resource management

TODO -> Let's not store resource IDS as strings...Implement CRC32 as compile-time macro.
http://cowboyprogramming.com/2007/01/04/practical-hash-ids/

# Editor

Todo:
- [ ] Gizmos: show transform + Axis
- [ ] Easier load and save
- [ ] Quick switch between existing levels
- [ ] Create template from entity in editor

# Input

- [ ] Create different backend for GUI and headless mode. (headless for server, should
      read input from the console)

# BUGS:

- [ ] GUI does not resize (imgui renderer problem)

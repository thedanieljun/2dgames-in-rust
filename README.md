# 2dgames-in-rust
A set of 2d games coded in Rust. Done in collaboration with the peers at the Claremont Consortium.

For Unit 2, our group managed to complete the two 2-d games with a total of 7 components for

the first game and 4 for the second game. While our original intention was to balance the components

between the two games, we found it much easier to focus our energy and resources onto the first game,

and have a bit more fun for our second game. For our first game, we decided to recreate “Flappy Bird”

with a more personal twist and thus, “Flappy Pigeon” was born. The first component we hit was the four

different graphical building blocks, where we used filled and non-filled rectangles to create obstacles,

textures to create the pigeon and backgrounds, and finally text to display scores and other various menu

items (1). We have separate title, play, and game over screens, each with different inputs, fulfilling the

menu/modal UI component of the unit (2). Focusing on the pigeon specifically, we implemented the sprite

and paper doll animation with atlases component, where the pigeon flapping its wings counts as the sprite,

and the paper doll animation as the object in the pigeons mouth changing each time the user starts a new

game (3). As for game feel, the pigeon falls realistically due to gravity, and it’s vertical movement feels

similar to that of the original Flappy Bird’s (4). Looking at the background and foreground, our pigeon

remains in the same x position while the background tilemap continuously scrolls to provide the effect

that the pigeon is moving (5). In conjunction with this component, we implemented the infinite tile map

and PCG component. While we did not adhere to the traditional definition of an infinite tile map, we

essentially added to the background and foreground obstacles as the player continues playing the game. In

order to accomplish this, similarly to a sprite sheet, our code reads pieces of a texture and adds them to

the game world. From this we procedurally generate our background, where the buildings, clouds, and

obstacles randomly switch when added to the screen (6). As for our final component, we added music. To

emulate a pigeon in NYC, we added dim city noises to the background, pigeon cooing sounds when the

game starts, and pigeon flapping sounds during the actual gameplay (7).

For our second game, we decided to stray from our game 1 concept, and produce a more

visual-novel style game. Named “Finding Nemo - The Afterstory”, game 2 focuses on presenting the

potential sequel to Pixar’s “Finding Nemo”. All 4 components implemented in this game overlap with

“Flappy Pigeon”. The first component we re-used were graphical building blocks, as we implemented

filled and unfilled rectangles to create textboxes, filled the textboxes with text, and included textures in

developing the background as well as sprites for characters (1). The second component was game feel,

which we argue exists when we consider the core of the game. By attempting to mimic a visual novel, we

focused more on the story line (albeit this is a game engine class) and upgraded what we learned from our

unit 1 adventure games. The use of background and sprites to create a 2-d environment, we’d argue,

enhances the dialogue aspect of the game, which is a foundational component of content-based games (2).

The third component was menus, turn taking, and modal UI. Separate title, play, and end game screens, as

well as pickable dialogue options all contribute to this component (3). The final component we used again

was music (4). In order to create an underwater vibe, we just added simple music playing in the

background while the user runs through the game. Originally, we had wanted to play different music for

different scenes, but decided it wasn’t necessary to hit the component for the game, and plan on

implementing it in the future if desirable.

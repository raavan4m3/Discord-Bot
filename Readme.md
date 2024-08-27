Discord Puzzle Bot
Overview
The Discord Puzzle Bot is an engaging and interactive bot designed for Discord servers to play a picture puzzle game. Users can start a puzzle, swap tiles to solve it, and track their scores. The bot is easy to set up and integrates seamlessly with Discord to provide a fun and interactive experience for users.
## Demo Video

[![Watch the video]](https://drive.google.com/file/d/1jnXaqI-kD7Pe1zsrWPHeS4M5oc2i8fja/view?usp=drive_link)

Why Adopt This Tool?
Interactive Gameplay: Adds a unique and entertaining puzzle game to your Discord server, keeping members engaged and having fun.
Easy Integration: Works directly within Discord, making it simple to interact with and use without leaving the platform.
Score Tracking: Tracks user scores and provides feedback, adding a competitive element to the game.
How It Works
Start a Puzzle: Use the !start_picture_puzzle command to initiate a new puzzle. The bot scrambles an image and displays it in the chat along with interactive buttons.
Solve the Puzzle: Click the "Solve Puzzle" button to reveal the original image once the puzzle is solved.
Swap Tiles: Use the !swap_tiles [index1] [index2] command to swap two tiles in the puzzle. The bot will update the puzzle image based on your swaps.
Submit a Guess: Use the !submit_guess [guess] command to submit a guess and receive feedback on its correctness.


Requirements:-
Rust
Cargo
A Discord bot token

Usage:-
```!start_picture_puzzle```: Starts a new puzzle with a scrambled image.
```!submit_guess [guess]```: Submits a guess for the puzzle.
```!swap_tiles [index1] [index2]```: Swaps two tiles in the puzzle


Acknowledgments:-
Serenity - Discord API library for Rust
Reqwest - HTTP client for Rust
Image - Image processing library for Rust.
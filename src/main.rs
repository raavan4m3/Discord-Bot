use serenity::{
    async_trait,
    builder::{CreateActionRow, CreateButton},
    model::{channel::Message, prelude::*},
    prelude::*,
    framework::standard::{
        CommandResult,
        macros::{command, group},
        Args, StandardFramework,
    },
    Client,
    model::gateway::GatewayIntents,
    model::interactions::{Interaction, InteractionResponseType},
    model::interactions::message_component::ButtonStyle,
};
use dotenv::dotenv;
use reqwest;
use futures::stream::StreamExt;
use std::env;
use image::{DynamicImage, GenericImageView};
use std::time::Duration;
use std::collections::HashMap;

mod image_processing;
use image_processing::scramble_image;

#[group]
#[commands(start_picture_puzzle, submit_guess, swap_tiles)]
struct General;

struct PicturePuzzleGame {
    current_image_url: String,
    scrambled_image: Option<DynamicImage>,
    tile_positions: Vec<usize>, // Store current positions of the tiles
    correct_positions: Vec<usize>, // Correct order of the tiles
    user_scores: HashMap<UserId, usize>,
}

impl PicturePuzzleGame {
    fn new() -> Self {
        PicturePuzzleGame {
            current_image_url: String::new(),
            scrambled_image: None,
            tile_positions: vec![0, 1, 2, 3, 4, 5, 6, 7, 8], // Default correct positions
            correct_positions: vec![0, 1, 2, 3, 4, 5, 6, 7, 8], // Correct positions
            user_scores: HashMap::new(),
        }
    }

    async fn load_image(&mut self, url: &str) {
        self.current_image_url = url.to_string();

        // Download the image
        let response = reqwest::get(url).await.expect("Failed to fetch image");
        let bytes = response.bytes().await.expect("Failed to read image bytes");

        match image::load_from_memory(&bytes) {
            Ok(img) => {
                let mut tile_positions = vec![];
                let scrambled_image = scramble_image(img.clone(), &mut tile_positions);
                
                // Save scrambled image to a file
                let scrambled_image_path = "scrambled_image.png";
                scrambled_image.save(scrambled_image_path).expect("Failed to save scrambled image");
                self.scrambled_image = Some(scrambled_image);
                self.tile_positions = tile_positions;
            },
            Err(e) => {
                println!("Failed to load image: {:?}", e);
            }
        }
    }

    fn check_guess(&self, guess: &str) -> bool {
        guess.to_lowercase() == "correct_answer" // Placeholder logic
    }

    fn increase_score(&mut self, user_id: UserId) {
        *self.user_scores.entry(user_id).or_insert(0) += 1;
    }

    fn get_score(&self, user_id: UserId) -> usize {
        *self.user_scores.get(&user_id).unwrap_or(&0)
    }

    fn recreate_scrambled_image(&self) -> Option<DynamicImage> {
        if let Some(ref scrambled_image) = self.scrambled_image {
            let (width, height) = scrambled_image.dimensions();
            let tile_width = width / 3;
            let tile_height = height / 3;
    
            // Create a new image buffer to hold the reordered image
            let mut buffer = image::ImageBuffer::new(width, height);
    
            // Recreate the scrambled image based on the current tile positions
            for (i, &pos) in self.tile_positions.iter().enumerate() {
                let x = (i % 3) as u32 * tile_width;
                let y = (i / 3) as u32 * tile_height;
                let tile_x = (pos % 3) as u32 * tile_width;
                let tile_y = (pos / 3) as u32 * tile_height;
    
                let tile = scrambled_image.crop_imm(tile_x, tile_y, tile_width, tile_height);
    
                for tile_y in 0..tile_height {
                    for tile_x in 0..tile_width {
                        let pixel = tile.get_pixel(tile_x, tile_y);
                        buffer.put_pixel(x + tile_x, y + tile_y, pixel);
                    }
                }
            }
    
            return Some(DynamicImage::ImageRgba8(buffer));
        }
    
        None
    }
}

struct GameKey;

impl TypeMapKey for GameKey {
    type Value = PicturePuzzleGame;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::MessageComponent(component) = interaction {
            let mut data = ctx.data.write().await;
            let game = data.get_mut::<GameKey>().unwrap();

            match component.data.custom_id.as_str() {
                "solve_puzzle" => {
                    // Ensure we have a valid game state and image URL
                    let original_image_url = &game.current_image_url;
                    if original_image_url.is_empty() {
                        eprintln!("No image URL set in game state.");
                        return;
                    }

                    let original_image_path = "original_image.png"; // Save image locally

                    // Download and save the original image
                    match reqwest::get(original_image_url).await {
                        Ok(response) => {
                            match response.bytes().await {
                                Ok(bytes) => {
                                    match image::load_from_memory(&bytes) {
                                        Ok(original_image) => {
                                            if let Err(e) = original_image.save(original_image_path) {
                                                eprintln!("Failed to save original image: {:?}", e);
                                            } else {
                                                if let Err(e) = component.create_interaction_response(&ctx.http, |response| {
                                                    response
                                                        .kind(InteractionResponseType::ChannelMessageWithSource)
                                                        .interaction_response_data(|message| {
                                                            message.content("The puzzle has been solved! Here is the original image:")
                                                        })
                                                })
                                                .await {
                                                    eprintln!("Failed to respond to interaction: {:?}", e);
                                                }

                                                if let Err(e) = component.channel_id.send_message(&ctx.http, |m| {
                                                    m.add_file(original_image_path)
                                                })
                                                .await {
                                                    eprintln!("Failed to send follow-up message with image: {:?}", e);
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            eprintln!("Failed to load original image: {:?}", e);
                                        }
                                    }
                                },
                                Err(e) => {
                                    eprintln!("Failed to read original image bytes: {:?}", e);
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to fetch original image: {:?}", e);
                        }
                    }
                }
                "swap_tiles" => {
                    let channel_id = component.channel_id;
                    if let Err(e) = channel_id.send_message(&ctx.http, |m| {
                        m.content("Please reply with the indices of the tiles you want to swap, separated by a space (e.g., '1 2').")
                    })
                    .await {
                        eprintln!("Failed to send prompt message: {:?}", e);
                    }

                    // Fetch and filter messages
                    match channel_id.messages(&ctx.http, |retriever| {
                        retriever.limit(100)
                    })
                    .await {
                        Ok(messages) => {
                            let user_messages: Vec<_> = messages
                                .into_iter()
                                .filter(|msg| msg.author.id == component.user.id)
                                .collect();

                            if let Some(reply) = user_messages.first() {
                                let content = reply.content.clone();
                                let mut indices = content.split_whitespace()
                                    .filter_map(|s| s.parse::<usize>().ok())
                                    .collect::<Vec<_>>();

                                if indices.len() == 2 {
                                    let index1 = indices[0] - 1;
                                    let index2 = indices[1] - 1;

                                    if index1 < 9 && index2 < 9 {
                                        game.tile_positions.swap(index1, index2);

                                        if let Some(new_scrambled_image) = game.recreate_scrambled_image() {
                                            if let Err(e) = new_scrambled_image.save("scrambled_image.png") {
                                                eprintln!("Failed to save scrambled image: {:?}", e);
                                            } else {
                                                if let Err(e) = channel_id.send_message(&ctx.http, |m| {
                                                    m.content("Here is the updated puzzle after the swap!")
                                                        .add_file("scrambled_image.png")
                                                })
                                                .await {
                                                    eprintln!("Failed to send updated puzzle image: {:?}", e);
                                                }

                                                if game.tile_positions == game.correct_positions {
                                                    if let Err(e) = channel_id.send_message(&ctx.http, |m| {
                                                        m.content("Congratulations! You solved the puzzle!")
                                                    })
                                                    .await {
                                                        eprintln!("Failed to send congratulations message: {:?}", e);
                                                    }
                                                    game.increase_score(component.user.id);
                                                }
                                            }
                                        }
                                    } else {
                                        if let Err(e) = channel_id.send_message(&ctx.http, |m| {
                                            m.content("Invalid tile indices! Please use numbers between 1 and 9.")
                                        })
                                        .await {
                                            eprintln!("Failed to send invalid indices message: {:?}", e);
                                        }
                                    }
                                } else {
                                    if let Err(e) = channel_id.send_message(&ctx.http, |m| {
                                        m.content("Please provide exactly two tile indices.")
                                    })
                                    .await {
                                        eprintln!("Failed to send invalid indices message: {:?}", e);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to fetch messages: {:?}", e);
                        }
                    }
                }
                _ => {}
            }
        }
    }
}


#[tokio::main]
async fn main() {
    dotenv().ok(); // Ensure .env file is loaded

    let token = env::var("DISCORD_TOKEN").expect("Token not found");

    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&token, intents)
        .framework(StandardFramework::new()
            .configure(|c| c.prefix("!"))
            .group(&GENERAL_GROUP))
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<GameKey>(PicturePuzzleGame::new());
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

fn create_button_row() -> CreateActionRow {
    let mut row = CreateActionRow::default();
    row.add_button({
        let mut button = CreateButton::default();
        button.label("Solve Puzzle")
              .custom_id("solve_puzzle")
              .style(ButtonStyle::Primary);
        button
    });
    row.add_button({
        let mut button = CreateButton::default();
        button.label("Swap Tiles")
              .custom_id("swap_tiles")
              .style(ButtonStyle::Secondary);
        button
    });
    row
}

#[command]
async fn start_picture_puzzle(ctx: &Context, msg: &Message) -> CommandResult {
    let mut data = ctx.data.write().await;
    let game = data.get_mut::<GameKey>().unwrap();

    let image_url = "https://images.pexels.com/photos/7418561/pexels-photo-7418561.jpeg?auto=compress&cs=tinysrgb&w=1260&h=750&dpr=2";
    game.load_image(image_url).await;

    // Send the scrambled image and buttons
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.content("Puzzle Mastermind! Solve the puzzle by swapping tiles or submitting guesses.")
                .add_file("scrambled_image.png")
                .components(|c| {
                    c.add_action_row(create_button_row())
                })
        })
        .await?;

    Ok(())
}

#[command]
async fn submit_guess(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let guess = args.rest();
    let user_id = msg.author.id;

    let mut data = ctx.data.write().await;
    let game = data.get_mut::<GameKey>().unwrap();

    if game.check_guess(guess) {
        game.increase_score(user_id);
        msg.channel_id.say(&ctx.http, "Correct!").await?;
    } else {
        msg.channel_id.say(&ctx.http, "Incorrect guess. Try again!").await?;
    }

    Ok(())
}

#[command]
async fn swap_tiles(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut data = ctx.data.write().await;
    let game = data.get_mut::<GameKey>().unwrap();

    // Get the two indices to swap from the command arguments
    let index1 = args.single::<usize>()?;
    let index2 = args.single::<usize>()?;

    // Ensure the indices are valid (1 through 9)
    if index1 < 1 || index1 > 9 || index2 < 1 || index2 > 9 {
        msg.channel_id.say(&ctx.http, "Invalid tile indices! Please use numbers between 1 and 9.").await?;
        return Ok(());
    }

    // Adjust for 0-based indexing
    let index1 = index1 - 1;
    let index2 = index2 - 1;

    // Swap the tiles in the game's tile_positions
    game.tile_positions.swap(index1, index2);

    // Re-create the scrambled image based on the new tile positions
    if let Some(new_scrambled_image) = game.recreate_scrambled_image() {
        new_scrambled_image.save("scrambled_image.png").expect("Failed to save image");

        msg.channel_id
            .send_message(&ctx.http, |m| {
                m.content("Here is the updated puzzle after the swap!")
                    .add_file("scrambled_image.png")
            })
            .await?;
    }

    // Check if the puzzle is solved
    if game.tile_positions == game.correct_positions {
        msg.channel_id.say(&ctx.http, "Congratulations! You solved the puzzle!").await?;
        game.increase_score(msg.author.id);
    }

    Ok(())
}

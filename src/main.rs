
use std::fs;
use eframe::{NativeOptions, App, Frame};
use eframe::egui::{self, Button, CentralPanel, Color32, Context, FontId, Grid, Key, RichText, Vec2};
use serde::Deserialize;
use rand::seq::SliceRandom;
use std::time::{Duration, Instant};

// the Puzzle struct stores the unsolved puzzle as well as the solution as strings
    // the puzzle and solution variables are deserialized from the puzzle json files
#[derive(Deserialize)]
struct Puzzle {
    puzzle: String,
    solution: String,
}

// the Puzzles struct stores a vector of puzzles, which also needs deserialization
    // The Puzzles struct is necessary because of how the json file is formatted
#[derive(Deserialize)]
struct Puzzles {
    puzzles: Vec<Puzzle>,
}

/*
    The Sudoku struct is the egui app itself
    username and user_id are needed for sending the user's scores to our database
    starting_grid stores the puzzle from the json file as an array of arrays (9x9 grid)
    player_grid also stores the puzzle from the json file, but the player_grid will be modified as the game is played, while starting_grid will not be
    solution_grid stores the solution from the json file
    difficulty is a string that can either be "Beginner", "Intermediate", "Advanced", or an empty string
    strikes is an unsigned 8-bit integer that represents the number of incorrect guesses the user has made -- the game ends at three strikes
    time_elapsed and timer_start are used to update the clock while the game is running
    game_over is a bool that represents whether the game has ended or not
*/
struct Sudoku {
    username: String,
    user_id: i32,
    starting_grid: [[char; 9]; 9],
    player_grid: [[char; 9]; 9],
    solution_grid: [[char; 9]; 9],
    selected: [usize; 2],
    difficulty: String,
    strikes: u8,
    time_elapsed: Duration,
    timer_start: Option<Instant>,
    game_over: bool,
}

impl Puzzle {
    // Puzzle constructor (takes one argument: difficulty)
    fn new(difficulty: String) -> Self {
        // Initialize empty strings to store the puzzle and solution data from the json file
        let mut puzzle = String::new();
        let mut solution = String::new();

        // insert the difficulty string into the file path
            // e.g. if difficulty is "Intermediate", the file_path will be "./puzzles/Intermediate.json"
        let file_path = format!("./puzzles/{}.json", difficulty);
        let file_contents = fs::read_to_string(file_path).unwrap(); // read the file into a string and store it as file_contents

        // deserialize the string into a Puzzles struct -- note that this gets ALL of the puzzles in the singular json file
        let puzzles: Puzzles = serde_json::from_str(&file_contents).expect("Failed to deserialize data");
        
        // make a random number generator
        let mut rng = rand::thread_rng();

        // get the random puzzle/solution pair from the Puzzles struct using the rng
        if let Some(random_puzzle) = puzzles.puzzles.choose(&mut rng) {
            puzzle = random_puzzle.puzzle.clone();
            solution = random_puzzle.solution.clone();
        }
        else {
            println!("Failed to get puzzle");
        }

        // return puzzle and solution
        Self {
            puzzle,
            solution,
        }
    }
}

// This is the implementation of the egui app for the Sudoku struct (this is what makes the Sudoku struct into an app)
impl App for Sudoku {
    // the update function runs every few milliseconds -- we can treat it like a while loop
    fn update(&mut self, ctx: &egui::Context, _frame: &mut Frame) {
        // if difficulty has not been set, show the difficulty screen to the user
            // the user can set the difficulty inside of the difficulty screen
        if self.difficulty.is_empty() {
            self.difficulty_screen(ctx);
        }
        else {  // if difficulty has been set, start the game
            // if 3 or more strikes, display the game over screen
            if self.strikes >= 3 {
                self.lose_screen(&ctx);
            }

            // if the player's grid matches the solution grid exactly, display the win screen
            else if self.player_grid == self.solution_grid {
                self.win_screen(&ctx);
            }

            // otherwise, the game is still running
            else {
                // calculate the time that has elapsed since the game started
                let elapsed = match self.timer_start {
                    Some(timer) => { 
                        if let Some(time) = self.time_elapsed.checked_add(timer.elapsed()) {
                            time
                        }
                        else {
                            //self.timer_start = Some(Instant::now());
                            Duration::ZERO
                        }
                    }
                    None => {
                        self.timer_start = Some(Instant::now());
                        Duration::ZERO
                    }
                };

                let selected_row = self.selected[0];
                let selected_col = self.selected[1];
                let selected_num = if selected_row < 10 && selected_col < 10 {
                    self.player_grid[selected_row][selected_col]
                }
                else {
                    '.'
                };

                CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading(self.difficulty.clone());
                        ui.heading(format!("Time elapsed: {}", elapsed.as_secs().to_string()));
                    });
                    ui.add_space(100.0);
                    ui.horizontal(|ui| {
                        // 4.5 buttons, width of 80 per button = 360
                        ui.add_space(ui.available_width() / 2.0 - 360.0 - 17.5);
                        // Create a 9x9 grid
                        Grid::new("9x9_grid") .spacing([5.0, 5.0]) // Optional spacing between cells 
                        .show(ui, |ui| {
                            for row in 0..9 {
                                for col in 0..9 {
                                    let num = self.player_grid[row][col];

                                    // if the cell is empty
                                    if num != '.' {
                                        let mut button_text = RichText::new(format!("{}", num.to_string()))
                                            .font(FontId::new(34.0, egui::FontFamily::Proportional));
                                        
                                        if self.solution_grid[row][col] != num {
                                            button_text = button_text.color(Color32::from_rgb(255, 60, 110));
                                        }

                                        else if self.starting_grid[row][col] == '.' {
                                            button_text = button_text.color(Color32::from_rgb(0, 124, 255));
                                        }

                                        let button_element = if selected_row < 10
                                            && selected_col < 10
                                            && self.player_grid[row][col] == selected_num {
                                                Button::new(button_text)
                                                    .min_size(Vec2::new(80.0, 80.0))
                                                    .fill(Color32::from_rgb(200, 200, 255))
                                        }
                                        else if (row <= 2 && 3 <= col && col <= 5)
                                            || (6 <= row && row <= 8 && 3 <= col && col <= 5)
                                            || (3 <= row && row <= 5 && col <= 2)
                                            || (3 <= row && row <= 5 && 6 <= col && col <= 8) {
                                                Button::new(button_text)
                                                    .min_size(Vec2::new(80.0, 80.0))
                                                    .fill(Color32::from_rgb(255, 255, 255))
                                        }
                                        else {
                                            Button::new(button_text)
                                                    .min_size(Vec2::new(80.0, 80.0))
                                        };
                                        
                                        let button = ui.add(button_element);
                                        let button_clone = button.clone();

                                        if row == selected_row || col == selected_col{
                                            button.highlight();
                                        }
                                        if button_clone.clicked() {
                                            self.selected[0] = row;
                                            self.selected[1] = col;
                                        }
                                    }
                                    else {
                                        let button_element = if (row <= 2 && 3 <= col && col <= 5)
                                            || (6 <= row && row <= 8 && 3 <= col && col <= 5)
                                            || (3 <= row && row <= 5 && col <= 2)
                                            || (3 <= row && row <= 5 && 6 <= col && col <= 8) {
                                                Button::new("")
                                                    .min_size(Vec2::new(80.0, 80.0))
                                                    .fill(Color32::from_rgb(255, 255, 255))
                                        }
                                        else {
                                            Button::new("")
                                                    .min_size(Vec2::new(80.0, 80.0))
                                        };

                                        let button = ui.add(button_element);
                                        let button_clone = button.clone();

                                        if row == selected_row || col == selected_col {
                                            button.highlight();
                                        }
                                        if button_clone.clicked() {
                                            self.selected[0] = row;
                                            self.selected[1] = col;
                                        }
                                    }
                                }
                                ui.end_row();
                            }
                        });
                    });

                    let valid_keys = [
                        Key::Num1, Key::Num2, Key::Num3,
                        Key::Num4, Key::Num5, Key::Num6,
                        Key::Num7, Key::Num8, Key::Num9,
                        Key::Delete
                    ];

                    for &key in &valid_keys {
                        if ui.input(|input| input.key_pressed(key))
                            && selected_row != 10
                            && selected_col != 10
                            && self.starting_grid[selected_row][selected_col] == '.' {
                                let num = key.name();
                                self.player_grid[selected_row][selected_col] = num.chars().next().unwrap();

                                if self.solution_grid[selected_row][selected_col] != self.player_grid[selected_row][selected_col] {
                                    self.strikes += 1;
                                }
                        }
                    }

                    if ui.input(|input| input.key_pressed(Key::Backspace)) {
                        self.player_grid[selected_row][selected_col] = '.';
                    }
                });
            }
            ctx.request_repaint();
        }
    }
}

impl Sudoku {
    fn new(username: String, user_id: i32) -> Self {
        Self {
            username,
            user_id,
            starting_grid: [['.'; 9]; 9],
            player_grid: [['.'; 9]; 9],
            solution_grid: [['.'; 9]; 9],
            selected: [10; 2],
            difficulty: "".into(),
            strikes: 0,
            time_elapsed: Duration::from_secs(0),
            timer_start: None,
            game_over: false,
        }
    }

    fn get_puzzle(&mut self) {
        let puzzle = Puzzle::new(self.difficulty.clone());

        for row in 0..9 {
            for col in 0..9 {
                let index = row * 9 + col;
                let puzzle_char_vec: Vec<char> = puzzle.puzzle.chars().collect();
                let solution_char_vec: Vec<char> = puzzle.solution.chars().collect();
                let puzzle_option = puzzle_char_vec.get(index);
                let solution_option = solution_char_vec.get(index);
                match puzzle_option {
                    Some(c) => {
                        self.starting_grid[row][col] = *c;
                        self.player_grid[row][col] = *c;
                    }
                    None => {}
                }
                match solution_option {
                    Some(c) => {
                        self.solution_grid[row][col] = *c;
                    }
                    None => {}
                }
            }
        }
    }

    fn difficulty_screen(&mut self, ctx: &Context) {
        CentralPanel::default().show(&ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(400.0);
                let title_text = RichText::new("Sudoku")
                    .font(FontId::new(30.0, egui::FontFamily::Proportional))
                    .color(Color32::from_rgb(150, 200, 255));
                ui.heading(title_text);
                ui.add_space(-300.0);
                ui.horizontal_centered(|ui| {
                    ui.add_space(ui.available_width() / 2.0 - 230.0 - 30.0);
                    let beginner_button_text = RichText::new("Beginner")
                        .font(FontId::new(24.0, egui::FontFamily::Proportional));
                    let intermediate_button_text = RichText::new("Intermediate")
                        .font(FontId::new(24.0, egui::FontFamily::Proportional));
                    let advanced_button_text = RichText::new("Advanced")
                        .font(FontId::new(24.0, egui::FontFamily::Proportional));

                    if ui.add(Button::new(beginner_button_text).min_size(Vec2::new(150.0, 100.0))).clicked() {
                        self.difficulty = "Beginner".to_string();
                    };
                    ui.add_space(30.0);
                    if ui.add(Button::new(intermediate_button_text).min_size(Vec2::new(150.0, 100.0))).clicked() {
                        self.difficulty = "Intermediate".to_string();
                    };
                    ui.add_space(30.0);
                    if ui.add(Button::new(advanced_button_text).min_size(Vec2::new(150.0, 100.0))).clicked() {
                        self.difficulty = "Advanced".to_string();
                    };
                });
                ui.add_space(-350.0);
                let test_button_text = RichText::new("Test")
                    .font(FontId::new(24.0, egui::FontFamily::Proportional));
                if ui.add(Button::new(test_button_text).min_size(Vec2::new(150.0, 100.0))).clicked() {
                    self.difficulty = "Test".to_string();
                };
            });
            if self.difficulty != "" {
                self.get_puzzle();
            }
        });
    }

    fn lose_screen(&self, ctx: &Context) {
        let mut count= 0.0;
        for row in 0..9 {
            for col in 0..9 {
                if self.player_grid[row][col] == self.solution_grid[row][col] {
                    count += 1.0;
                }
            }
        }
        let percentage: f32 = (count / 81.0) * 100.0;
        let rounded = percentage.round() as i32;

        CentralPanel::default().show(&ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Game over!");
                ui.label(format!("You filled {} percent of the board", rounded));
            });
        });
    }

    fn win_screen(&mut self, ctx: &Context) {
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("You Win!");
                if !self.game_over {
                    if let Some(time) = self.timer_start {
                        self.time_elapsed = time.elapsed();
                    }
                    else {
                        
                    }
                    //self.time_elapsed = self.timer_start.elapsed();
                    self.game_over = true;
                }

                ui.label(format!("You completed the puzzle in {} seconds", self.time_elapsed.as_secs()));
            });
        });
    }
}

fn main() {
    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        ..Default::default()
    };

    let _ = eframe::run_native( // Start Vapor
        "Sudoku", // Set the app title
        native_options, 
        Box::new(|_cc| Ok(Box::new(Sudoku::new("John".into(), 2)))),
    );
}

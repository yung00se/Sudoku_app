
use std::fs;
use eframe::{NativeOptions, App, Frame};
use eframe::egui::{self, Button, CentralPanel, Color32, Context, FontId, Grid, Key, RichText, Vec2, Rect, Pos2, Align2, FontFamily};
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
                // if timer_start is None (uninitialized), it will be initialized
                // if it is already initialized, time_elapsed will be incremented
                let elapsed = match self.timer_start {
                    Some(timer) => { 
                        if let Some(time) = self.time_elapsed.checked_add(timer.elapsed()) {
                            time
                        }
                        else {
                            Duration::ZERO
                        }
                    }
                    None => {
                        self.timer_start = Some(Instant::now());
                        Duration::ZERO
                    }
                };

                // selected_row is the row of the cell that the user currently has selected
                // same is true for selected_col
                let selected_row = self.selected[0];
                let selected_col = self.selected[1];
                // selected_num is the character in the cell that the user currently has selected
                let selected_num = if selected_row < 10 && selected_col < 10 {
                    self.player_grid[selected_row][selected_col]
                }
                // if the user has not clicked on a cell yet, selected num is set to '.'
                else {
                    '.'
                };

                // egui window
                CentralPanel::default().show(ctx, |ui| {
                    // shows the selected difficulty and the time elapsed since the game started
                    ui.vertical_centered(|ui| {
                        let header_text = RichText::new(self.difficulty.clone())
                            .font(FontId::new(30.0, FontFamily::Proportional));
                        ui.heading(header_text);
                        ui.add_space(30.0);
                        ui.heading(format!("Time elapsed: {}", elapsed.as_secs().to_string()));
                        ui.add_space(20.0);
                        ui.horizontal(|ui| {
                            ui.add_space(ui.available_width() / 2.0 - 75.0 - 10.0);
                            for i in 1..=3 {
                                let (rect_response, painter) = ui.allocate_painter(Vec2::new(50.0, 50.0), egui::Sense::hover()); 
                                let rect = rect_response.rect;
                                
                                // Draw the rectangle
                                painter.rect_filled(rect, 0.0, Color32::WHITE);
                                // Blue rectangle
                                // Draw the text inside the rectangle

                                let text = if i <= self.strikes {
                                    "X"
                                }
                                else {
                                    ""
                                };

                                painter.text(rect.center(), 
                                    Align2::CENTER_CENTER,
                                    text,
                                    FontId::new(40.0, FontFamily::Proportional),
                                    Color32::RED);
                            }
                        });
                    });
                    ui.add_space(20.0);
                    ui.horizontal(|ui| {
                        // place the grid at the center of the window, then offset it to the left by half of its width
                        // half of grid width -- 4.5 buttons, width of 80 per button = 360
                        // we also have to include the spaces between buttons when calculating the offset
                        // spaces -- 4 spaces, width of 5 per space = 20
                        ui.add_space(ui.available_width() / 2.0 - 360.0 - 20.0);
                        // this is the grid that holds the 9x9 grid of cells
                        Grid::new("9x9_grid")
                            .spacing([5.0, 5.0]) // Optional spacing between cells 
                            .show(ui, |ui| {
                                // iterate through each row and column
                                for row in 0..9 {
                                    for col in 0..9 {
                                        // get the number currenlty stored in the player grid at the current row and column
                                        let num = self.player_grid[row][col];

                                        // if the cell does not have a number
                                        if num != '.' {
                                            // create the text for the cell
                                            let mut button_text = RichText::new(format!("{}", num.to_string()))
                                                .font(FontId::new(34.0, FontFamily::Proportional));
                                            
                                            // if the number in the grid does not match the solution grid, make the text color Red
                                            if self.solution_grid[row][col] != num {
                                                button_text = button_text.color(Color32::from_rgb(255, 60, 110));
                                            }
                                            
                                            // if the number in the grid does match the solution grid,
                                                // and the starting grid is empty at the current row and colunn, make the text color Blue
                                            else if self.starting_grid[row][col] == '.' {
                                                button_text = button_text.color(Color32::from_rgb(0, 124, 255));
                                            }

                                            // create the button element
                                            // first, highlight all cells in the grid that are the same as the selected number
                                                // for example, if the user has selected a cell with 3 in it, all cells in the grid that contain 3 will be highlighted Blue
                                            let button_element = if selected_row < 10
                                                && selected_col < 10
                                                && self.player_grid[row][col] == selected_num {
                                                    Button::new(button_text)
                                                        .min_size(Vec2::new(80.0, 80.0))
                                                        .fill(Color32::from_rgb(200, 200, 255))
                                            }
                                            // next we make the checkerboard pattern
                                                // for example, the top left, top right, bottom left, and bottom right 3x3 areas will have white cells,
                                                // while the remaining cells will be gray
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
                                            
                                            // add the button, and make a clone of it to check for clicks
                                            let button = ui.add(button_element);
                                            let button_clone = button.clone();

                                            // highlight the entire row and the entire column that correspond to the cell the user has selected
                                            if row == selected_row || col == selected_col{
                                                button.highlight();
                                            }
                                            // if a button is clicked, set self.selected to the correct coordinates
                                            if button_clone.clicked() {
                                                self.selected[0] = row;
                                                self.selected[1] = col;
                                            }
                                        }
                                        // for all of the empty cells on the board
                                            // again make the checkerboard pattern, dividing up each 3x3 area in the grid
                                            // this time, the text in the button is just an empty string
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

                                            // this code is identical to the code at the bottom of the last if block
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
                                    // after each row, call ui.end_row() to tell the grid that we want to start a new row
                                    ui.end_row();
                                }
                        });
                    });

                    // define key presses that are allowed -- the only ones allowed are digits 1-9
                    // NOTE: below, we also allow for the user to press the backspace key, but we do not need to include it in this array
                    let valid_keys = [
                        Key::Num1, Key::Num2, Key::Num3,
                        Key::Num4, Key::Num5, Key::Num6,
                        Key::Num7, Key::Num8, Key::Num9,
                    ];

                    // iterate through the valid keys (digits) to check if any were pressed during the last frame
                    for &key in &valid_keys {
                        // if a number key was pressed and the selected_row and selected_col are in range
                        // and the starting grid at that position is empty,
                            // we get the digit associated with that key press and store it in the player grid
                        if ui.input(|input| input.key_pressed(key))
                            && selected_row != 10
                            && selected_col != 10
                            && self.starting_grid[selected_row][selected_col] == '.' {
                                let num = key.name();
                                self.player_grid[selected_row][selected_col] = num.chars().next().unwrap();

                                // if the number entered is incorrect, increment the user's strikes by 1
                                if self.solution_grid[selected_row][selected_col] != self.player_grid[selected_row][selected_col] {
                                    self.strikes += 1;
                                }
                        }
                    }

                    // if the backspace key was pressed during the last frame, reset the player grid at that position to be empty
                    if ui.input(|input| input.key_pressed(Key::Backspace)) {
                        self.player_grid[selected_row][selected_col] = '.';
                    }
                });
            }
            // by default, egui only updates the window when there is user input like mouse movement or keyboard presses.
            // request repaint solves this by updating the window each frame
            ctx.request_repaint();
        }
    }
}

// functions for Sudoku struct
impl Sudoku {
    // Sudoku constructor -- takes username and user_id -- all other member variables are initialized to a default value 
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

    // gets a new puzzle from json file and stores it in Sudoku structs member variables
    fn get_puzzle(&mut self) {
        // when Puzzle::new is called, we fetch a random puzzle from the json file associated with the current difficulty
        // NOTE: self.difficulty will always be populated to either "Beginner", "Intermediate", or "Advanced" when this function is called
        let puzzle = Puzzle::new(self.difficulty.clone());

        // Convert the puzzle string to a vector of chars
        // Do the same for the solution string
        let puzzle_char_vec: Vec<char> = puzzle.puzzle.chars().collect();
        let solution_char_vec: Vec<char> = puzzle.solution.chars().collect();

        // iterate through each row and column for self.starting_grid, self.player_grid, and self.solution_grid
        for row in 0..9 {
            for col in 0..9 {
                // the puzzle string and solution string are just that: strings -- they are not 2d arrays.
                    // So, we cannnot index them as we would a 2d array. We must use one and only one index
                    // we can calculate the index by multiplying row by 9 and adding col
                let index = row * 9 + col;

                // get the char at the specified index in the puzzle char vector
                if let Some(puzzle_char) = puzzle_char_vec.get(index) {
                    // store the character from the puzzle string in self.starting_grid as well as self.player_grid
                    self.starting_grid[row][col] = *puzzle_char;
                    self.player_grid[row][col] = *puzzle_char;
                } else { }

                // get the char at the specified index in the solution char vector
                if let Some(solution_char) = solution_char_vec.get(index) {
                    // store the character from the solution string in self.solution_grid
                    self.solution_grid[row][col] = *solution_char;
                } else { }
            }
        }
    }

    // displays the start screen where the user selects the difficulty
    fn difficulty_screen(&mut self, ctx: &Context) {
        CentralPanel::default().show(&ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(400.0);
                // Sudoku title
                let title_text = RichText::new("Sudoku")
                    .font(FontId::new(30.0, FontFamily::Proportional))
                    .color(Color32::from_rgb(60, 190, 220));
                ui.heading(title_text);

                // Beginner, Intermediate, and Advanced butttons
                ui.add_space(-300.0);
                ui.horizontal_centered(|ui| {
                    ui.add_space(ui.available_width() / 2.0 - 230.0 - 30.0);
                    let beginner_button_text = RichText::new("Beginner")
                        .font(FontId::new(24.0, FontFamily::Proportional));
                    let intermediate_button_text = RichText::new("Intermediate")
                        .font(FontId::new(24.0, FontFamily::Proportional));
                    let advanced_button_text = RichText::new("Advanced")
                        .font(FontId::new(24.0, FontFamily::Proportional));

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
                // THIS SHOULD NOT BE INCLUDED IN FINAL SUBMISSION -- THIS IS FOR TESTING WIN SCREEN
                ui.add_space(-350.0);
                let test_button_text = RichText::new("Test")
                    .font(FontId::new(24.0, FontFamily::Proportional));
                if ui.add(Button::new(test_button_text).min_size(Vec2::new(150.0, 100.0))).clicked() {
                    self.difficulty = "Test".to_string();
                };
            });

            // if the difficulty is not an empty string, call self.get_puzzle to randomly get a puzzle
            if self.difficulty != "" {
                self.get_puzzle();
            }
        });
    }

    // displays the game over screen when the user loses
    fn lose_screen(&self, ctx: &Context) {
        // iterate through self.player_grid and self.solution_grid, and count how many of the 81 cells the user had correct
        let mut count= 0.0;
        for row in 0..9 {
            for col in 0..9 {
                if self.player_grid[row][col] == self.solution_grid[row][col] {
                    count += 1.0;
                }
            }
        }
        // calculate the percentage of cells the user had correct
        let percentage: f32 = (count / 81.0) * 100.0;
        // convert percentage to i32
        let rounded = percentage.round() as i32;

        // display ui elements, including the percentage of the board the user had correct
        CentralPanel::default().show(&ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("Game over!");
                ui.label(format!("You filled {} percent of the board", rounded));
            });
        });
    }

    // displays win screen when the user has correctly filled the entire board
    fn win_screen(&mut self, ctx: &Context) {
        // display ui elements
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("You Win!");
                // this checks to see if self.game_over has been set or not
                // if self.game_over has not been set, record the time elapsed and store it in self.time_elapsed
                    // then set self.game_over to true so the program only enters this if block once
                if !self.game_over {
                    if let Some(time) = self.timer_start {
                        self.time_elapsed = time.elapsed();
                    } else { }
                    self.game_over = true;
                }

                // display how many seconds it took the user to complete the puzzle
                ui.label(format!("You completed the puzzle in {} seconds", self.time_elapsed.as_secs()));
            });
        });
    }
}

fn main() {
    // create a NativeOptions struct to pass to the eframe app
    // the viewport member varialbe is specified here because we wont a maximized window
    let native_options = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true).with_resizable(false),
        ..Default::default()
    };

    // run the eframe app, passing a newly constructed Sudoku struct
    let _ = eframe::run_native( // Start Vapor
        "Sudoku", // Set the app title
        native_options, 
        Box::new(|_cc| Ok(Box::new(Sudoku::new("John".into(), 2)))),
    );
}

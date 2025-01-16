use std::{env, fs::File, io::{Read, Write}, path::{Path, PathBuf}, result};

use maze_gen::generate_maze;
use nannou::{app, color, event::{Key, Update}, geom::pt2, glam::vec2, App, Frame, LoopMode};
use nannou_egui::{egui::{self, Button, Slider, TextEdit}, Egui};
use rand::rngs::ThreadRng;

mod maze_gen;

#[derive(Default, Clone, PartialEq, Eq, Debug)]
enum CellType {
    Start,
    End,
    #[default]
    Normal
}

#[derive(Default, Clone, Debug)]
struct Cell {
    top_wall: bool,
    bottom_wall: bool,
    left_wall: bool,
    right_wall: bool,
    visited: bool,
    cell_type: CellType
}

impl Into<u8> for &Cell {
    fn into(self) -> u8 {
        (self.top_wall as u8)                       << 0 |
        (self.bottom_wall as u8)                    << 1 |
        (self.left_wall as u8)                      << 2 |
        (self.right_wall as u8)                     << 3 |
        (self.visited as u8)                        << 4 |
        ((self.cell_type == CellType::End) as u8)   << 5 |
        ((self.cell_type == CellType::Start) as u8) << 6
    }
}

impl From<u8> for Cell {
    fn from(value: u8) -> Self {
        Cell {
            top_wall: (value & (1 << 0)) != 0,
            bottom_wall: (value & (1 << 1)) != 0,
            left_wall: (value & (1 << 2)) != 0,
            right_wall: (value & (1 << 3)) != 0,
            visited: (value & (1 << 4)) != 0,
            cell_type: 
                if (value & (1 << 5)) != 0 {
                    CellType::End
                } else if (value & (1 << 6)) != 0 {
                    CellType::Start
                } else {
                    CellType::Normal
                },
        }
    }
}

struct Model {
    grid: Vec<Vec<Cell>>,

    grid_width: usize,
    grid_height: usize,

    cell_width: f32,
    cell_height: f32,

    show_fps: bool,

    maze_name: String,

    egui: Egui,

    rng: ThreadRng
}

fn save_maze_to_file(grid_width: usize, grid_height: usize, grid: &Vec<Vec<Cell>>, file_path: &Path) -> Result<(), std::io::Error> {
    let mut file = File::create(file_path)?;

    file.write_all(&(grid_width as u64).to_le_bytes())?;
    file.write_all(&(grid_height as u64).to_le_bytes())?;

    let mut packed_data = Vec::new();
    let mut buffer = 0u16;
    let mut bits_in_buffer = 0;

    for row in grid {
        for cell in row {
            let as_byte: u8 = cell.into(); 
            buffer |= (as_byte as u16 & 0b111111) << bits_in_buffer;
            bits_in_buffer += 6;

            if bits_in_buffer >= 8 {
                packed_data.push((buffer & 0xFF) as u8);
                buffer >>= 8;
                bits_in_buffer -= 8;
            }
        }
    }

    if bits_in_buffer > 0 {
        packed_data.push(buffer as u8);
    }

    file.write_all(&packed_data)?;

    Ok(())
}

fn load_maze_from_file(file_path: &Path) -> Result<(usize, usize, Vec<Vec<Cell>>), std::io::Error> {
    let mut file = File::open(file_path)?;

    let mut size_buffer = [0u8; 8];
    
    file.read_exact(&mut size_buffer)?;
    let grid_width = u64::from_le_bytes(size_buffer) as usize;

    file.read_exact(&mut size_buffer)?;
    let grid_height = u64::from_le_bytes(size_buffer) as usize;

    let mut packed_data = Vec::new();
    file.read_to_end(&mut packed_data)?;

    let mut grid = Vec::new();

    let mut buffer = 0u16;
    let mut bits_in_buffer = 0;
    let mut byte_index = 0;

    for row in 0..grid_height {
        grid.push(Vec::new());

        for _ in 0..grid_width {
            if bits_in_buffer < 6 {
                if byte_index >= packed_data.len() {
                    return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Not enough data."));
                }

                buffer |= (packed_data[byte_index] as u16) << bits_in_buffer;
                bits_in_buffer += 8;
                byte_index += 1;
            }

            let value = (buffer & 0b111111) as u8;
            buffer >>= 6;
            bits_in_buffer -= 6;

            let cell = Cell::from(value);

            grid[row].push(cell);
        }
    }

    Ok((grid_width, grid_height, grid))
}

fn main() {
    nannou::app(init)
        .loop_mode(LoopMode::rate_fps(30.))
        .update(update)
        .run();
}

fn init(app: &App) -> Model {
    let window_id = app
        .new_window()
        .title("Maze Generator")
        .key_pressed(key_pressed)
        .raw_event(raw_window_event)
        .view(draw)
        .build()
        .unwrap();
    let window = app.window(window_id).unwrap();

    let grid_width = 32;
    let grid_height = 32;

    let grid = (0..grid_height).map(|_| {
        (0..grid_width).map(|_| {
            let mut cell = Cell::default();
            cell.top_wall = true;
            cell.bottom_wall = true;
            cell.right_wall = true;
            cell.left_wall = true;

            cell
        }).collect()
    }).collect();

    let cell_width = 24.0;
    let cell_height = 24.0;

    let egui = Egui::from_window(&window);

    Model {
        grid,

        grid_width,
        grid_height,

        cell_height,
        cell_width,

        show_fps: false,

        maze_name: String::from("Maze"),

        egui,

        rng: rand::thread_rng()
    }
}

fn reset_grid(grid: &mut Vec<Vec<Cell>>) {
    for row in grid {
        for col in row {
            col.top_wall = true;
            col.bottom_wall = true;
            col.right_wall = true;
            col.left_wall = true;
            col.visited = false;
        }
    }
}

fn update(app: &App, model: &mut Model, update: Update) {
    let egui = &mut model.egui;

    egui.set_elapsed_time(update.since_start);

    let ctx = egui.begin_frame();

    egui::Window::new("Maze Config").show(&ctx, |ui| {
        // UI for Maze Width
        ui.horizontal(|ui| {
            ui.label("Maze Width:");
            ui.add(Slider::new(&mut model.grid_width, 1..=50));
        });

        // UI for Maze Height
        ui.horizontal(|ui| {
            ui.label("Maze Height:");
            ui.add(Slider::new(&mut model.grid_height, 1..=50));
        });

        // UI for Load Maze
        ui.horizontal(|ui| {
            ui.label("Maze name:");
            ui.add(TextEdit::singleline(&mut model.maze_name).desired_width(200.0));

        });

        if ui.add(Button::new("Load Maze")).clicked() {
            let result = load_maze_from_file(&env::current_dir().unwrap().join(&model.maze_name));

            if let Ok((width, height, grid)) = result {
                model.grid_width = width;
                model.grid_height = height;
                model.grid = grid;
            } else {
                println!("Error loading maze: {:?}", result);
            }
        }

        if ui.add(Button::new("Save Maze")).clicked() {
            let result = save_maze_to_file(model.grid_width, model.grid_height, &model.grid, 
                &env::current_dir().unwrap().join(&model.maze_name)
            );

            if result.is_err() {
                println!("Error saving maze: {:?}", result);
            }
        }

        if ui.add(Button::new("Generate Maze")).clicked() {
            reset_grid(&mut model.grid);
            generate_maze((0, 0), &mut model.grid, &mut model.rng);
        }

        ui.label(format!("{:.2} FPS", app.fps()));
    });
}

fn raw_window_event(_app: &App, model: &mut Model, event: &nannou::winit::event::WindowEvent) {
    model.egui.handle_raw_event(event);
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    // match key {
    //     Key::F => model.show_fps = !model.show_fps,
    //     Key::Space => {
    //         reset_grid(&mut model.grid);
    //         generate_maze((0, 0), &mut model.grid, &mut model.rng)
    //     },
    //     Key::S => {
    //         let result = save_maze_to_file(model.grid_width, model.grid_height, &model.grid, &Path::new("./maze.mz"));

    //         if result.is_err() {
    //             println!("Error saving maze: {:?}", result);
    //         }
    //     },
    //     Key::L => {
    //         let result = load_maze_from_file(&Path::new("./maze.mz"));

    //         if let Ok((width, height, grid)) = result {
    //             println!("Loaded maze!");
    //             println!("Width: {}, Height: {}", width, height);
    //             model.grid = grid;
    //         } else {
    //             println!("Error loading maze: {:?}", result);
    //         }
    //     }
    //     _ => {},
    // }
}

fn draw(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    draw.background().color(color::hsv(0.0, 0.0, 0.01));

    let grid_top_left_x = -(model.grid_width as f32 * model.cell_width - model.cell_width) * 0.5;
    let grid_top_left_y = (model.grid_height as f32 * model.cell_height - model.cell_height) * 0.5;

    let half_width = model.cell_width * 0.5;
    let half_height = model.cell_height * 0.5;

    for row in 0..model.grid_height {
        for col in 0..model.grid_width {
            let cell = &model.grid[row][col];

            let x = grid_top_left_x + col as f32 * model.cell_width;
            let y = grid_top_left_y - row as f32 * model.cell_height;

            let color = if cell.visited {
                match cell.cell_type {
                    CellType::Normal => color::hsv(0.0, 0.0, 0.85),
                    CellType::End => color::hsv(0.0, 0.75, 1.0),
                    CellType::Start => color::hsv(0.32, 0.75, 0.85)
                }
            } else {
                color::hsv(0.0, 0.0, 0.05)
            };

            draw.rect()
                .x_y(x, y)
                .w_h(model.cell_width, model.cell_height)
                .color(color);

            let top_left =     vec2(x-half_width, y+half_height);
            let bottom_left =  vec2(x-half_width, y-half_height);
            let top_right =    vec2(x+half_width, y+half_height);
            let bottom_right = vec2(x+half_width, y-half_height);

            if cell.top_wall {
                draw.line()
                    .start(top_left)
                    .end(top_right)
                    .stroke_weight(4.0)
                    .color(color::BLACK);
            }

            if cell.bottom_wall {
                draw.line()
                    .start(bottom_left)
                    .end(bottom_right)
                    .stroke_weight(4.0)
                    .color(color::BLACK);
            }

            if cell.left_wall {
                draw.line()
                    .start(top_left)
                    .end(bottom_left)
                    .stroke_weight(4.0)
                    .color(color::BLACK);
            }

            if cell.right_wall {
                draw.line()
                    .start(top_right)
                    .end(bottom_right)
                    .stroke_weight(4.0)
                    .color(color::BLACK);
            }
        }
    }

    if model.show_fps {
        let window_rect = app.window_rect();
        let text_position = pt2(window_rect.left() - 25.0, window_rect.top() - 10.0);

        draw.ellipse()
            .radius(4.0)
            .xy(text_position);

        draw.text(&format!("{:.1} FPS", app.fps()))
            .color(color::WHITE)
            .font_size(16)
            .right_justify()
            .xy(text_position);
    }

    draw.to_frame(app, &frame).unwrap();
    model.egui.draw_to_frame(&frame).unwrap();
}
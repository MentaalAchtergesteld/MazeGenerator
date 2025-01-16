use maze_gen::generate_maze;
use nannou::{color, event::{Key, Update}, geom::pt2, glam::vec2, App, Frame, LoopMode};
use rand::rngs::ThreadRng;

mod maze_gen;

#[derive(Default, Clone)]
enum CellType {
    Start,
    End,
    #[default]
    Normal
}

#[derive(Default, Clone)]
struct Cell {
    top_wall: bool,
    bottom_wall: bool,
    left_wall: bool,
    right_wall: bool,
    visited: bool,
    cell_type: CellType
}

#[derive(Default)]
struct Model {
    grid: Vec<Vec<Cell>>,

    grid_width: usize,
    grid_height: usize,

    cell_width: f32,
    cell_height: f32,

    show_fps: bool,

    rng: ThreadRng
}

fn main() {
    nannou::app(init)
        .loop_mode(LoopMode::rate_fps(30.))
        .update(update)
        .run();
}

fn init(app: &App) -> Model {
    app
        .new_window()
        .title("Maze Generator")
        .key_pressed(key_pressed)
        .view(draw)
        .build()
        .unwrap();

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

    Model {
        grid,

        grid_width,
        grid_height,

        cell_height,
        cell_width,

        ..Default::default()
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

fn update(_app: &App, _model: &mut Model, _update: Update) {
    // let delta = update.since_last.as_secs_f32();
}

fn key_pressed(_app: &App, model: &mut Model, key: Key) {
    match key {
        Key::F => model.show_fps = !model.show_fps,
        Key::Space => {
            reset_grid(&mut model.grid);
            generate_maze((0, 0), &mut model.grid, &mut model.rng)
        },
        _ => {},
    }
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
}
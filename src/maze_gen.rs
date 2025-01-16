use rand::{seq::SliceRandom, Rng};
use crate::{Cell, CellType};

fn maze_gen_step(
    stack: &mut Vec<(usize, usize)>,
    grid: &mut Vec<Vec<Cell>>,
    rng: &mut impl Rng
) -> Option<(usize, usize)> {
    if let Some(current) = stack.pop() {
        grid[current.0][current.1].visited = true;

        let neighbour_directions = [
            (0, 1),
            (0, -1),
            (1, 0),
            (-1, 0)
        ];
    
        let mut neighbours = Vec::new();
    
        for direction in neighbour_directions {
            let neighbour_row = current.0 as isize + direction.0;
            if neighbour_row < 0 || neighbour_row >= grid.len() as isize {
                continue;
            }
    
            let neighbour_col = current.1 as isize + direction.1;
            if neighbour_col < 0 || neighbour_col >= grid[neighbour_row as usize].len() as isize {
                continue;
            }
    
            if grid[neighbour_row as usize][neighbour_col as usize].visited {
                continue;
            }
    
            neighbours.push(((
                neighbour_row as usize,
                neighbour_col as usize
            ), direction));
        }
    
        if !neighbours.is_empty() {
            stack.push(current);
    
            neighbours.shuffle(rng);
    
            let next = neighbours[0];
            let next_coords = next.0;
            let next_direction = next.1;
    
    
            match next_direction {
                (0, 1) => {
                    grid[current.0][current.1].right_wall = false;
                    grid[next_coords.0][next_coords.1].left_wall = false;
                }
                (0, -1) => {
                    grid[current.0][current.1].left_wall = false;
                    grid[next_coords.0][next_coords.1].right_wall = false;
                }
                (1, 0) => {
                    grid[current.0][current.1].bottom_wall = false;
                    grid[next_coords.0][next_coords.1].top_wall = false;
                }
                (-1, 0) => {
                    grid[current.0][current.1].top_wall = false;
                    grid[next_coords.0][next_coords.1].bottom_wall = false;
                },
                _ => {}
            }
    
            Some(next_coords)
        } else if let Some(next) = stack.pop() {
            Some(next)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn generate_maze(start: (usize, usize), grid: &mut Vec<Vec<Cell>>, rng: &mut impl Rng) {
    let mut stack = vec![start];

    grid[start.0][start.1].cell_type = CellType::Start;

    while let Some(next) = maze_gen_step(&mut stack, grid, rng) {
        stack.push(next);
    }
}
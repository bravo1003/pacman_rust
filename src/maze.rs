use crate::geometry::{BOARD_HEIGHT, BOARD_WIDTH, Point, TILE};

/// `#` wall, `=` ghost-house door, `.` pellet, `o` energizer. Digits mark
/// spawn points: 0 Pac-Man, 1 Blinky, 2 Inky, 3 Pinky, 4 Clyde.
const LAYOUT: [&str; BOARD_HEIGHT] = [
    "                            ",
    "                            ",
    "                            ",
    "############################",
    "#............##............#",
    "#.####.#####.##.#####.####.#",
    "#o####.#####.##.#####.####o#",
    "#.####.#####.##.#####.####.#",
    "#..........................#",
    "#.####.##.########.##.####.#",
    "#.####.##.########.##.####.#",
    "#......##....##....##......#",
    "######.##### ## #####.######",
    "     #.##### ## #####.#     ",
    "     #.##    1     ##.#     ",
    "     #.## ###==### ##.#     ",
    "######.## #      # ##.######",
    "      .   #2 3 4 #   .      ",
    "######.## #      # ##.######",
    "     #.## ######## ##.#     ",
    "     #.##          ##.#     ",
    "     #.## ######## ##.#     ",
    "######.## ######## ##.######",
    "#............##............#",
    "#.####.#####.##.#####.####.#",
    "#.####.#####.##.#####.####.#",
    "#o..##.......0 .......##..o#",
    "###.##.##.########.##.##.###",
    "###.##.##.########.##.##.###",
    "#......##....##....##......#",
    "#.##########.##.##########.#",
    "#.##########.##.##########.#",
    "#..........................#",
    "############################",
    "                            ",
    "                            ",
];

const _: () = {
    let mut row = 0;
    while row < BOARD_HEIGHT {
        assert!(LAYOUT[row].len() == BOARD_WIDTH);
        row += 1;
    }
};

const WIDTH: i32 = BOARD_WIDTH as i32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tile {
    Wall,
    Door,
    Pellet,
    Energizer,
    Empty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Food {
    Pellet,
    Energizer,
}

impl Tile {
    fn food(self) -> Option<Food> {
        match self {
            Self::Pellet => Some(Food::Pellet),
            Self::Energizer => Some(Food::Energizer),
            _ => None,
        }
    }
}

/// The maze as currently eaten.
#[derive(Clone)]
pub struct Maze {
    tiles: [Tile; BOARD_WIDTH * BOARD_HEIGHT],
}

impl Maze {
    pub fn new() -> Self {
        let mut tiles = [Tile::Empty; BOARD_WIDTH * BOARD_HEIGHT];
        for (row, line) in LAYOUT.iter().enumerate() {
            for (col, symbol) in line.bytes().enumerate() {
                tiles[row * BOARD_WIDTH + col] = match symbol {
                    b'#' => Tile::Wall,
                    b'=' => Tile::Door,
                    b'.' => Tile::Pellet,
                    b'o' => Tile::Energizer,
                    _ => Tile::Empty,
                };
            }
        }
        Self { tiles }
    }

    pub fn has_food(&self) -> bool {
        self.tiles.iter().any(|tile| tile.food().is_some())
    }

    /// Remaining food with the pixel position of its tile.
    pub fn food(&self) -> impl Iterator<Item = (Point, Food)> + '_ {
        self.tiles.iter().enumerate().filter_map(|(index, tile)| {
            let col = (index % BOARD_WIDTH) as i32;
            let row = (index / BOARD_WIDTH) as i32;
            tile.food()
                .map(|food| (Point::new(col * TILE, row * TILE), food))
        })
    }

    /// Whether an actor at `pos` runs into a wall, or into the door unless
    /// `can_use_door`. The first wall or door corner found decides.
    pub fn blocks(&self, pos: Point, can_use_door: bool) -> bool {
        for (col, row) in corner_tiles(pos) {
            // Columns wrap so the tunnel stays open past the screen edges.
            match self.tile(row * WIDTH + (col % WIDTH).abs()) {
                Tile::Wall => return true,
                Tile::Door => return !can_use_door,
                _ => {}
            }
        }
        false
    }

    /// Eats the first pellet or energizer under an actor at `pos`.
    pub fn eat(&mut self, pos: Point) -> Option<Food> {
        corner_tiles(pos).into_iter().find_map(|(col, row)| {
            let index = row * WIDTH + col;
            let food = self.tile(index).food()?;
            self.tiles[index as usize] = Tile::Empty;
            Some(food)
        })
    }

    fn tile(&self, index: i32) -> Tile {
        usize::try_from(index)
            .ok()
            .and_then(|index| self.tiles.get(index).copied())
            .unwrap_or(Tile::Empty)
    }
}

/// Spawn position of the actor with the given layout marker.
pub fn spawn_point(marker: u8) -> Point {
    LAYOUT
        .iter()
        .enumerate()
        .find_map(|(row, line)| {
            let col = line.bytes().position(|symbol| symbol == marker)?;
            Some(Point::new(col as i32 * TILE + TILE / 2, row as i32 * TILE))
        })
        .expect("spawn marker is missing from the layout")
}

/// Tiles under the corners of an actor box, in the order they are probed.
fn corner_tiles(pos: Point) -> [(i32, i32); 4] {
    let (left, right) = covered_tiles(pos.x);
    let (top, bottom) = covered_tiles(pos.y);
    [(left, top), (right, top), (left, bottom), (right, bottom)]
}

/// First and last tile covered by a box starting at pixel `coord`.
fn covered_tiles(coord: i32) -> (i32, i32) {
    let first = coord.div_euclid(TILE);
    let last = if coord.rem_euclid(TILE) == 0 {
        first
    } else {
        first + 1
    };
    (first, last)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_points() {
        assert_eq!(
            spawn_point(b'0'),
            Point::new(13 * TILE + TILE / 2, 26 * TILE)
        );
        assert_eq!(
            spawn_point(b'1'),
            Point::new(13 * TILE + TILE / 2, 14 * TILE)
        );
    }

    #[test]
    fn walls_and_door() {
        let maze = Maze::new();
        // Tile-aligned in the top-left corridor, with walls above and left.
        let corridor = Point::new(TILE, 4 * TILE);
        assert!(!maze.blocks(corridor, false));
        assert!(maze.blocks(Point::new(TILE, 4 * TILE - 1), false));
        assert!(maze.blocks(Point::new(TILE - 1, 4 * TILE), false));
        // The door only lets ghosts through when allowed.
        let door = Point::new(13 * TILE, 15 * TILE);
        assert!(maze.blocks(door, false));
        assert!(!maze.blocks(door, true));
    }

    #[test]
    fn tunnel_is_open_off_screen() {
        let maze = Maze::new();
        assert!(!maze.blocks(Point::new(-TILE / 2, 17 * TILE), false));
        assert!(!maze.blocks(Point::new(28 * TILE + TILE / 2, 17 * TILE), false));
    }

    #[test]
    fn eating_clears_food_once() {
        let mut maze = Maze::new();
        let pellet = Point::new(TILE, 4 * TILE);
        assert_eq!(maze.eat(pellet), Some(Food::Pellet));
        assert_eq!(maze.eat(pellet), None);
        assert_eq!(maze.eat(Point::new(TILE, 6 * TILE)), Some(Food::Energizer));
        assert_eq!(maze.food().count(), 242);
    }
}

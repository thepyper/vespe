use rand::Rng;
use std::collections::LinkedList;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Snake {
    pub body: LinkedList<(u16, u16)>,
    pub direction: Direction,
    pub just_ate: bool,
}

impl Snake {
    pub fn new(start: (u16, u16), direction: Direction) -> Self {
        let mut body = LinkedList::new();
        body.push_back(start);
        Self {
            body,
            direction,
            just_ate: false,
        }
    }

    pub fn head(&self) -> (u16, u16) {
        *self.body.front().unwrap()
    }

    pub fn move_forward(&mut self, area: (u16, u16)) {
        let (head_x, head_y) = self.head();
        let (new_head_x, new_head_y) = match self.direction {
            Direction::Up => (head_x, head_y.saturating_sub(1)),
            Direction::Down => (head_x, (head_y + 1)),
            Direction::Left => (head_x.saturating_sub(1), head_y),
            Direction::Right => ((head_x + 1), head_y),
        };

        self.body.push_front((new_head_x, new_head_y));

        if !self.just_ate {
            self.body.pop_back();
        } else {
            self.just_ate = false;
        }
    }

    pub fn check_collision(&self) -> bool {
        let head = self.head();
        self.body.iter().skip(1).any(|&segment| segment == head)
    }
}

#[derive(Debug)]
pub struct GameState {
    pub width: u16,
    pub height: u16,
    pub snake: Snake,
    pub food: (u16, u16),
    pub score: u32,
    pub game_over: bool,
}

impl GameState {
    pub fn new(width: u16, height: u16) -> Self {
        let snake_start = (width / 2, height / 2);
        let snake = Snake::new(snake_start, Direction::Right);
        let mut game = Self {
            width,
            height,
            snake,
            food: (0, 0),
            score: 0,
            game_over: false,
        };
        game.spawn_food();
        game
    }

    pub fn spawn_food(&mut self) {
        let mut rng = rand::thread_rng();
        loop {
            let x = rng.gen_range(1..(self.width - 1));
            let y = rng.gen_range(1..(self.height - 1));
            if !self.snake.body.contains(&(x, y)) {
                self.food = (x, y);
                break;
            }
        }
    }

    pub fn tick(&mut self) {
        if self.game_over {
            return;
        }
        
        let head = self.snake.head();
        if (self.snake.direction == Direction::Up && head.1 == 1)
        || (self.snake.direction == Direction::Down && head.1 == self.height - 2)
        || (self.snake.direction == Direction::Left && head.0 == 1)
        || (self.snake.direction == Direction::Right && head.0 == self.width - 2)
        {
            self.game_over = true;
            return;
        }

        self.snake.move_forward((self.width, self.height));

        if self.snake.check_collision() {
            self.game_over = true;
            return;
        }

        if self.snake.head() == self.food {
            self.snake.just_ate = true;
            self.score += 1;
            self.spawn_food();
        }
    }

    pub fn change_direction(&mut self, direction: Direction) {
        if self.snake.direction.opposite() != direction {
            self.snake.direction = direction;
        }
    }
}

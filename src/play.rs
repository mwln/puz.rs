use std::time::Duration;

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    widgets::Paragraph,
    Frame,
};

const GAME_WIDTH: i32 = 5;
const GAME_HEIGHT: i32 = 5;

const LETTER_GRID: [[char; 5]; 5] = [
    ['A', 'B', 'C', 'D', 'E'],
    ['F', 'G', 'H', 'I', 'J'],
    ['K', 'L', 'M', 'N', 'O'],
    ['P', 'Q', 'R', 'S', 'T'],
    ['U', 'V', 'W', 'X', 'Y'],
];

enum Axis {
    X,
    Y,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Coordinate {
    x: usize,
    y: usize,
}

impl Coordinate {
    fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
    fn increment(&mut self, axis: Axis) {
        match axis {
            Axis::X => self.x = self.x + 1,
            Axis::Y => self.y = self.y + 1,
        }
    }
    fn decrement(&mut self, axis: Axis) {
        match axis {
            Axis::X => self.x = self.x - 1,
            Axis::Y => self.y = self.y - 1,
        }
    }
    fn set_x(&mut self, x: usize) {
        self.x = x;
    }
    fn set_y(&mut self, y: usize) {
        self.y = y;
    }
}

#[derive(Debug, Clone)]
struct PuzzleGrid(Vec<Vec<char>>);

impl PuzzleGrid {
    fn new(rows: usize, cols: usize, default_char: char) -> Self {
        Self(vec![vec![default_char; cols]; rows])
    }
    fn width(&self) -> usize {
        self.0.len()
    }
    fn height(&self) -> usize {
        self.0.get(0).map_or(0, |row| row.len())
    }
    fn get(&self, row: usize, col: usize) -> Option<&char> {
        self.0.get(row).and_then(|r| r.get(col))
    }

    fn set(&mut self, row: usize, col: usize, value: char) -> Result<(), &'static str> {
        match self.0.get_mut(row).and_then(|r| r.get_mut(col)) {
            Some(cell) => {
                *cell = value;
                Ok(())
            }
            None => Err("Index out of bounds"),
        }
    }
}

#[derive(Debug)]
struct Model {
    selected_cell: Coordinate,
    running_state: RunningState,
    grid: PuzzleGrid,
}

impl Model {
    fn new() -> Self {
        Self {
            selected_cell: Coordinate::new(0, 0),
            running_state: RunningState::default(),
            grid: PuzzleGrid::new(5, 5, 'A'),
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
enum RunningState {
    #[default]
    Running,
    Done,
}

#[derive(PartialEq)]
enum Message {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    Quit,
}

pub fn start() -> color_eyre::Result<()> {
    tui::install_panic_hook();
    let mut terminal = tui::init_terminal()?;
    let mut model = Model::new();

    while model.running_state != RunningState::Done {
        terminal.draw(|f| view(&mut model, f))?;

        let mut current_msg = handle_event(&model)?;

        while current_msg.is_some() {
            current_msg = update(&mut model, current_msg.unwrap());
        }
    }

    tui::restore_terminal()?;
    Ok(())
}

fn view(model: &mut Model, frame: &mut Frame) {
    frame.render_widget(
        Paragraph::new(format!(
            "Selected Coordinate: {} {}",
            model.selected_cell.x, model.selected_cell.y
        )),
        frame.area(),
    );
}

/// Convert Event to Message
///
/// We don't need to pass in a `model` to this function in this example
/// but you might need it as your project evolves
fn handle_event(_: &Model) -> color_eyre::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key));
            }
        }
    }
    Ok(None)
}

fn handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Char('j') => Some(Message::MoveDown),
        KeyCode::Char('k') => Some(Message::MoveUp),
        KeyCode::Char('h') => Some(Message::MoveLeft),
        KeyCode::Char('l') => Some(Message::MoveRight),
        KeyCode::Char('q') => Some(Message::Quit),
        _ => None,
    }
}

fn update(model: &mut Model, msg: Message) -> Option<Message> {
    let indexed_height = model.grid.height() - 1;
    let indexed_width = model.grid.width() - 1;
    match msg {
        Message::MoveLeft => {
            if model.selected_cell.x == 0 {
                model.selected_cell.set_x(indexed_width)
            } else {
                model.selected_cell.decrement(Axis::X)
            }
        }
        Message::MoveRight => {
            if model.selected_cell.x == indexed_width {
                model.selected_cell.set_x(0)
            } else {
                model.selected_cell.increment(Axis::X)
            }
        }
        Message::MoveUp => {
            if model.selected_cell.y == 0 {
                model.selected_cell.set_y(indexed_height)
            } else {
                model.selected_cell.decrement(Axis::Y)
            }
        }
        Message::MoveDown => {
            if model.selected_cell.y == indexed_height {
                model.selected_cell.set_y(0)
            } else {
                model.selected_cell.increment(Axis::Y)
            }
        }
        Message::Quit => {
            // You can handle cleanup and exit here
            model.running_state = RunningState::Done;
        }
    };
    None
}

mod tui {
    use ratatui::{
        backend::{Backend, CrosstermBackend},
        crossterm::{
            terminal::{
                disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
            },
            ExecutableCommand,
        },
        Terminal,
    };
    use std::{io::stdout, panic};

    pub fn init_terminal() -> color_eyre::Result<Terminal<impl Backend>> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Ok(terminal)
    }

    pub fn restore_terminal() -> color_eyre::Result<()> {
        stdout().execute(LeaveAlternateScreen)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn install_panic_hook() {
        let original_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic_info| {
            stdout().execute(LeaveAlternateScreen).unwrap();
            disable_raw_mode().unwrap();
            original_hook(panic_info);
        }));
    }
}

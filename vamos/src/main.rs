use crossterm::{
    event::{Event, poll, read},
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, size},
    *,
};
#[warn(unused)]
use std::io::{Stdout, Write, stdout};
use std::{
    alloc::Layout,
    collections, env,
    fs::File,
    io::{BufRead, BufReader},
    time::Duration,
};
const STATUS_BAR_HEIGHT: u16 = 2;
const LNUM_BAR_WIDTH: u16 = 4;

struct TextCursor {
    x: u64,
    y: u64,
}

impl TextCursor {
    fn new() -> Self {
        Self { x: (0), y: (0) }
    }
    fn from(_x: u64, _y: u64) -> Self {
        Self { x: (_x), y: (_y) }
    }
}

struct TextBuffer {
    data: Vec<String>,
    //offset from where file is currently rendered
    file: Option<File>,
    x: u64,
    y: u64,
    has_changed: bool,
}

impl TextBuffer {
    fn new() -> Self {
        Self {
            data: vec![String::new(); 1],
            file: None,
            x: 0,
            y: 0,
            has_changed: false,
        }
    }

    fn from_file(file: File) -> std::io::Result<Self> {
        //copy file to data structure
        let reader = BufReader::new(&file);

        let mut lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

        if lines.is_empty() {
            lines.push(String::new());
        }

        Ok(Self {
            data: lines,
            file: Some(file),
            x: 0,
            y: 0,
            has_changed: false,
        })
    }

    fn from_path(path: &str) -> std::io::Result<Self> {
        let file = match File::open(path) {
            Ok(f) => f,
            Err(_) => File::create(path)?,
        };
        Ok(TextBuffer::from_file(file)?)
    }
}

fn init_app(stdout: &mut Stdout) -> std::io::Result<(TextBuffer)> {
    let buffer = match env::args().nth(1) {
        None => TextBuffer::new(),
        Some(path) => TextBuffer::from_path(path.as_str())?,
    };

    enable_raw_mode()?;
    queue!(
        stdout,
        cursor::MoveTo(0, 0),
        terminal::Clear(terminal::ClearType::All)
    )?;
    stdout.flush()?;
    Ok(buffer)
}

fn render_line_num(
    buffer: &TextBuffer,
    tcursor: &TextCursor,
    stdout: &mut Stdout,
) -> std::io::Result<()> {
    let (mut cols, mut rows) = size()?;
    rows -= STATUS_BAR_HEIGHT;
    for screen_row in 0..rows {
        queue!(
            stdout,
            cursor::MoveTo(0, screen_row),
            Print(format!("{}", 1+buffer.y + screen_row as u64))
        )?;
    }
    Ok(())
}

fn render_status_bar(
    buffer: &TextBuffer,
    tcursor: &TextCursor,
    stdout: &mut Stdout,
) -> std::io::Result<()> {
    let (mut cols, mut rows) = size()?;

    //print cursor position
    queue!(
        stdout,
        cursor::MoveTo(LNUM_BAR_WIDTH, rows - STATUS_BAR_HEIGHT)
    )?;
    queue!(stdout, Print(format!("Cursor {}:{}", tcursor.y, tcursor.x)))?;
    queue!(
        stdout,
        cursor::MoveTo(LNUM_BAR_WIDTH, rows - STATUS_BAR_HEIGHT + 1)
    )?;
    let file_name = "nazwa pliku";
    // let file_name = match buffer.file {
    //     None => "",
    //     Some(&f) => "nazwa pliku",
    // };
    queue!(stdout, Print(format!("\"{}\"", file_name)))?;
    Ok(())
}

fn render_text_buffer(buffer: &TextBuffer, stdout: &mut Stdout) -> std::io::Result<()> {
    let (mut cols, mut rows) = size()?;
    cols -= LNUM_BAR_WIDTH;
    rows -= STATUS_BAR_HEIGHT;
    for screen_row in 0..rows {
        let file_row_idx = buffer.y as usize + screen_row as usize;

        queue!(stdout, cursor::MoveTo(LNUM_BAR_WIDTH, screen_row))?;

        if let Some(line) = buffer.data.get(file_row_idx) {
            let visible_slice: String = line
                .chars()
                .skip(buffer.x as usize)
                .take(cols as usize)
                .collect();

            queue!(stdout, Print(&visible_slice))?;
        }

        queue!(stdout, terminal::Clear(terminal::ClearType::UntilNewLine))?;
    }

    Ok(())
}
fn render(buffer: &TextBuffer, tcursor: &TextCursor, stdout: &mut Stdout) -> std::io::Result<()> {
    render_text_buffer(buffer, stdout);
    render_status_bar(buffer, tcursor, stdout);
    render_line_num(buffer, tcursor, stdout);
    stdout.flush()?;
    Ok(())
}

fn handle_event() -> std::io::Result<bool> {
    if poll(Duration::from_millis(500))? {
        match read()? {
            Event::FocusGained => println!("FocusGained"),
            Event::FocusLost => println!("FocusLost"),
            Event::Key(event) => return Ok(true),
            Event::Paste(data) => println!("Pasted {:?}", data),
            Event::Mouse(event) => println!("{:?}", event),
            Event::Resize(width, height) => println!("New size {}x{}", width, height),
        }
    } else {
    }
    Ok(false)
}

fn main() -> std::io::Result<()> {
    let mut stdout = stdout();
    let mut buffer = init_app(&mut stdout)?;
    let mut tcursor = TextCursor::new();
    loop {
        render(&buffer, &tcursor, &mut stdout)?;
        if handle_event()? {
            break;
        }
    }
    disable_raw_mode()?;
    Ok(())
}

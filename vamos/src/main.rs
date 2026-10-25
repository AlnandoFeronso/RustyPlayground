use crossterm::{
    event::{Event, poll, read},
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, size},
    *,
};
#[warn(unused)]
use std::io::{Stdout, Write, stdout};
use std::{
    alloc::Layout, collections, env, fs::File, io::{BufRead, BufReader}, time::Duration,
};

struct EditorLayout {
    lnum_w: u16,
    lnum_h: u16,
    status_w: u16,
    status_h: u16,
}

impl EditorLayout {
    fn default(term_rows: u16, term_cols: u16) -> Self {
        //TODO: apply config file
        Self {
            lnum_w: 2,
            lnum_h: term_rows - 2,
            status_w: term_cols,
            status_h: 2,
        }
    }

    fn resize(&mut self, term_rows: u16, term_cols: u16) {
        self.lnum_h = term_rows - self.status_h;
        self.status_w = term_cols;
    }
}

struct TextBuffer {
    data: Vec<String>,
    //offset from where file is currently rendered
    file: Option<File>,
    x: u64,
    y: u64,
    layout: EditorLayout, 
    has_changed: bool,
}

impl TextBuffer {
    fn new() -> Self {
        Self {
            data: vec![String::new(); 1],
            file: None,
            x: 0,
            y: 0,
            layout: EditorLayout::default(term_rows, term_cols)
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

fn render(buffer: &TextBuffer, stdout: &mut Stdout) -> std::io::Result<()> {
    let (cols, rows) = size()?;

    for screen_row in 0..rows {
        let file_row_idx = buffer.y as usize + screen_row as usize;

        queue!(stdout, cursor::MoveTo(0, screen_row))?;

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
    loop {
        render(&buffer, &mut stdout)?;
        if handle_event()? {
            break;
        }
    }
    disable_raw_mode()?;
    Ok(())
}

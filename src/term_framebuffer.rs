#![allow(dead_code)]

use crate::iterators::par_iter_mut;
use anyhow::Result;
use crossterm::cursor::{DisableBlinking, EnableBlinking, MoveTo};
use crossterm::event::{poll, read, EnableMouseCapture, Event, KeyEvent};
use crossterm::{execute, queue};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, EnableLineWrap, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use std::io::{stdout, Write};
use std::time::Duration;

pub struct TerminalFramebuffer<D: Render + Default + Copy + 'static> {
    data: Vec<D>,
    size: (usize, usize),
}
impl<D: Render + Default + Copy + 'static> TerminalFramebuffer<D> {
    pub fn new() -> Result<Self> {
        let (cols, rows) = size()?;
        let data = vec![D::default(); cols as usize * rows as usize];

        Ok(Self {
            data,
            size: (cols as usize, rows as usize),
        })
    }

    pub fn render_wrapping(&self) -> Result<()> {
        let mut s = stdout();
        queue!(s, MoveTo(0, 0), EnableLineWrap)?;

        let mut carry = D::CarryOver::default();
        for v in self.data.iter() {
            v.render(&mut s, &mut carry);
        }

        Ok(())
    }

    pub fn check_size(&mut self) -> Result<()> {
        let cs = size()?;
        let cs = (cs.0 as usize, cs.1 as usize);
        if self.size != cs {
            self.size = cs;
            self.data = vec![D::default(); cs.0 * cs.1];
        }

        Ok(())
    }

    pub fn raw_data(&mut self) -> &mut Vec<D> {
        &mut self.data
    }

    pub fn iterate_uv_par<F: Fn(f32, f32) -> D::Input + Send + Sync + 'static>(&mut self, func: F) {
        let size = self.size;
        par_iter_mut(&mut self.data, move |v, i| {
            let (xc, yc) = Self::index_to_cords(i, size);
            let x = xc as f32 / size.0 as f32;
            let y = yc as f32 / size.1 as f32;
            let new = func(x, y);
            v.edit(new);
        })
    }

    pub fn index_to_cords(index: usize, size: (usize, usize)) -> (usize, usize) {
        let y = index / size.0;
        let x = index % size.0;
        (x, y)
    }
}

pub fn terminal_setup_alternate_screen() -> Result<()> {
    enable_raw_mode()?;
    execute!(
        stdout(),
        EnterAlternateScreen,
        EnableLineWrap,
        DisableBlinking,
        crossterm::cursor::Hide
    )?;
    Ok(())
}

pub fn terminal_cleanup_alternate_screen() -> Result<()> {
    disable_raw_mode()?;
    execute!(
        stdout(),
        LeaveAlternateScreen,
        EnableLineWrap,
        EnableBlinking,
        crossterm::cursor::Show
    )?;
    Ok(())
}

pub trait Render {
    type CarryOver: Default;
    type Input;

    /// renders a single char to the current position on the screen,
    /// allows printing multiple characters for escape code formatting
    fn render<W: Write>(&self, writer: &mut W, c: &mut Self::CarryOver);

    /// modifies the internal value of the function, used in Self::render
    fn edit(&mut self, change: Self::Input);
}

pub struct TerminalState {
    pub mouse_position: (u16, u16),
    pub focused: bool,
    pub keys: Vec<KeyEvent>
}
impl TerminalState {
    pub fn new() -> Self {
        Self {
            mouse_position: (0, 0),
            focused: true,
            keys: vec![],
        }
    }

    pub fn enable_mouse() -> Result<()> {
        execute!(stdout(), EnableMouseCapture)?;
        Ok(())
    }

    fn poll(arr: &mut Vec<Event>) -> Result<()> {
        if poll(Duration::ZERO)? {
            arr.push(read()?);
            Self::poll(arr)?;
        }
        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        self.keys.clear();
        let mut events = Vec::new();
        Self::poll(&mut events)?;
        for e in events {
            match e {
                Event::FocusGained => {
                    self.focused = true;
                }
                Event::FocusLost => {
                    self.focused = false;
                }
                Event::Key(k) => {
                    self.keys.push(k);
                }
                Event::Mouse(me) => {
                    self.mouse_position = (me.column, me.row);
                }
                Event::Paste(_) => {}
                Event::Resize(_, _) => {}
            }
        }

        Ok(())
    }
}


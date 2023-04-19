use std::io::Write;

use terminal::error;
use terminal::Action;
use terminal::Clear;
use terminal::Event;
use terminal::KeyCode;
use terminal::MouseButton;
use terminal::MouseEvent;
use terminal::Retrieved;
use terminal::Value;

enum Mode {
    Insert,
    Erase,
}

fn main() -> error::Result<()> {
    let mut term = terminal::stdout();
    term.act(Action::EnableRawMode)?;
    term.act(Action::HideCursor)?;
    term.act(Action::EnableMouseCapture)?;
    term.act(Action::EnterAlternateScreen)?;
    term.act(Action::ClearTerminal(Clear::All))?;
    let mut mode = Mode::Insert;
    let mut index = 0;
    let size = 32;
    let mut layers: Vec<Vec<Option<(u8, u8, u8)>>> = vec![vec![None; size * size]];
    let mut hide = false;
    let active = 0;
    let padding = 1;
    let colors = &[
        (255, 255, 255),
        (255, 0, 0),
        (0, 255, 0),
        (0, 0, 255),
        (255, 255, 0),
        (255, 0, 255),
        (0, 255, 255),
    ];
    for y in 0..size {
        for x in 0..size {
            let (r, g, b) = if (y + x) % 2 == 0 {
                (30, 30, 30)
            } else {
                (20, 20, 20)
            };
            term.act(Action::MoveCursorTo(
                padding * 2 + x as u16 * 2,
                padding + y as u16,
            ))?;
            term.write(format!("\x1b[48;2;{r};{g};{b}m  \x1b[0m").as_bytes())?;
        }
    }
    loop {
        let mode_symbol = match mode {
            Mode::Insert => "INS",
            Mode::Erase => "ERS",
        };
        term.act(Action::MoveCursorTo(padding * 2, padding + size as u16 + 1))?;
        term.write(
            format!(
                "{: <offset$}[{size}x{size}]",
                format!("[{mode_symbol}]"),
                offset = (size as u16 * 2 - ((size.to_string().len() * 2 + 3) as u16)) as usize
            )
            .as_bytes(),
        )?;
        for (i, color) in colors.iter().enumerate() {
            let (r, g, b) = color;
            let symbol = if i == index { "[]" } else { "  " };
            term.act(Action::MoveCursorTo(
                padding * 2 + i as u16 * 4,
                padding + size as u16 + 3,
            ))?;
            term.write(
                format!("\x1b[48;2;{r};{g};{b}m\x1b[38;2;0;0;0m{symbol}\x1b[0m").as_bytes(),
            )?;
        }
        // for y in 0..10 {
        //     for x in 0..10 {
        //         let (r, g, b): (u32, u32, u32) = colors[1];
        //         let r = r.saturating_sub(x * 20);
        //         let g = g.saturating_add(y * 20).min(255);
        //         let b = b.saturating_add(y * 20).min(255);
        //         term.act(Action::MoveCursorTo(
        //             padding * 2 + x as u16 * 2,
        //             padding + size + 5 + y as u16,
        //         ))?;
        //         term.write(format!("\x1b[48;2;{r};{g};{b}m\x1b[38;2;0;0;0m  \x1b[0m").as_bytes())?;
        //     }
        // }
        // for (i, color) in colors.iter().enumerate() {
        //     for n in 0..32 {
        //         let (r, g, b): &(u32, u32, u32) = color;
        //         let r = r.saturating_sub(n * 10);
        //         let g = g.saturating_sub(n * 10);
        //         let b = b.saturating_sub(n * 10);
        //         term.act(Action::MoveCursorTo(
        //             padding * 2 + n as u16 * 2,
        //             padding + size + 5 + i as u16,
        //         ))?;
        //         term.write(format!("\x1b[48;2;{r};{g};{b}m\x1b[38;2;0;0;0m  \x1b[0m").as_bytes())?;
        //     }
        // }
        term.flush()?;
        if let Retrieved::Event(Some(event)) = term.get(Value::Event(None))? {
            match event {
                Event::Mouse(mouse) => match mouse {
                    MouseEvent::Up(button, col, row, _) | MouseEvent::Drag(button, col, row, _) => {
                        match button {
                            MouseButton::Left => {
                                if col >= padding * 2
                                    && col < (padding + size as u16) * 2
                                    && row >= padding
                                    && row < padding + size as u16
                                {
                                    match mode {
                                        Mode::Insert => {
                                            let y = row - padding;
                                            let x = (col - padding * 2) / 2;
                                            term.act(Action::MoveCursorTo(col / 2 * 2, row))?;
                                            let (r, g, b) = colors[index];
                                            term.write(
                                                format!("\x1b[48;2;{r};{g};{b}m  \x1b[0m")
                                                    .as_bytes(),
                                            )?;
                                            term.flush()?;
                                            layers[0][y as usize * size + x as usize] =
                                                Some((r, g, b));
                                        }
                                        Mode::Erase => {
                                            let y = row - padding;
                                            let x = (col - padding * 2) / 2;
                                            term.act(Action::MoveCursorTo(
                                                padding * 2 + x * 2,
                                                padding + y,
                                            ))?;
                                            if !hide {
                                                let (r, g, b) = if (y + x) % 2 == 0 {
                                                    (30, 30, 30)
                                                } else {
                                                    (20, 20, 20)
                                                };
                                                term.write(
                                                    format!("\x1b[48;2;{r};{g};{b}m  \x1b[0m")
                                                        .as_bytes(),
                                                )?;
                                            } else {
                                                term.write("  ".as_bytes())?;
                                            }
                                            term.flush()?;
                                            layers[0][y as usize * size + x as usize] = None;
                                        }
                                    }
                                }
                                if row == padding + size as u16 + 3 {
                                    for i in 0..colors.len() {
                                        if col == padding * 2 + i as u16 * 4
                                            || col == (padding * 2 + i as u16 * 4) + 1
                                        {
                                            index = i;
                                            break;
                                        }
                                    }
                                }
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                },
                Event::Key(key) => match key.code {
                    KeyCode::Char(char) => match char {
                        'q' => break,
                        '1' => mode = Mode::Insert,
                        '2' => mode = Mode::Erase,
                        'y' => {
                            hide = !hide;
                            term.act(Action::ClearTerminal(Clear::All))?;
                            if !hide {
                                for y in 0..size {
                                    for x in 0..size {
                                        let (r, g, b) = if (y + x) % 2 == 0 {
                                            (30, 30, 30)
                                        } else {
                                            (20, 20, 20)
                                        };
                                        term.act(Action::MoveCursorTo(
                                            padding * 2 + x as u16 * 2,
                                            padding + y as u16,
                                        ))?;
                                        term.write(
                                            format!("\x1b[48;2;{r};{g};{b}m  \x1b[0m").as_bytes(),
                                        )?;
                                    }
                                }
                            }
                            for y in 0..size {
                                for x in 0..size {
                                    if let Some((r, g, b)) = layers[active][y * size + x] {
                                        term.act(Action::MoveCursorTo(
                                            padding * 2 + x as u16 * 2,
                                            padding + y as u16,
                                        ))?;
                                        term.write(
                                            format!("\x1b[48;2;{r};{g};{b}m  \x1b[0m").as_bytes(),
                                        )?;
                                    }
                                }
                            }
                        }
                        _ => (),
                    },
                    KeyCode::Backspace => {
                        layers[active] = vec![None; size * size];
                        term.act(Action::ClearTerminal(Clear::All))?;
                        if !hide {
                            for y in 0..size {
                                for x in 0..size {
                                    let (r, g, b) = if (y + x) % 2 == 0 {
                                        (30, 30, 30)
                                    } else {
                                        (20, 20, 20)
                                    };
                                    term.act(Action::MoveCursorTo(
                                        padding * 2 + x as u16 * 2,
                                        padding + y as u16,
                                    ))?;
                                    term.write(
                                        format!("\x1b[48;2;{r};{g};{b}m  \x1b[0m").as_bytes(),
                                    )?;
                                }
                            }
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }
    term.act(Action::ClearTerminal(Clear::All))?;
    term.act(Action::DisableMouseCapture)?;
    term.act(Action::ShowCursor)?;
    term.act(Action::LeaveAlternateScreen)?;
    term.act(Action::DisableRawMode)?;
    Ok(())
}

// fn get_term_size(term: &Terminal<Stdout>) -> error::Result<Option<(usize, usize)>> {
//     if let Retrieved::TerminalSize(col, row) = term.get(Value::TerminalSize)? {
//         return Ok(Some((col as usize, row as usize)));
//     }
//     Ok(None)
// }

// fn get_cursor_pos(term: &Terminal<Stdout>) -> error::Result<Option<(usize, usize)>> {
//     if let Retrieved::CursorPosition(col, row) = term.get(Value::CursorPosition)? {
//         return Ok(Some((col as usize, row as usize)));
//     }
//     Ok(None)
// }


use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use std::io::{Write, stdout};
use termion::color::{Rgb, Fg};

use auto_args::AutoArgs;

mod beep;

#[derive(AutoArgs)]
struct Flags {
    /// Enable sound effects
    sound: bool,
}

struct Explosion {
    x: i32,
    y: i32,
    fuse: i32,
    color: Rgb,
}

struct Rocket {
    x: i32,
    y: i32,
    vx: i32,
    vy: i32,
    who: char,
    color: Rgb,
}

fn bright_color() -> Rgb {
    let mut r = rand::random::<u8>();
    let mut g = rand::random::<u8>();
    let mut b = rand::random::<u8>();
    if r >= b && r >= g {
        r = 255;
    } else if g >= b {
        g = 255;
    } else {
        b = 255;
    }
    Rgb(r,g,b)
}

fn main() -> Result<(), std::io::Error> {
    let flags = Flags::from_args();
    let mut _raw_stdout = termion::cursor::HideCursor::from(stdout().into_raw_mode().unwrap());
    let mut stdin = termion::async_stdin().keys();
    let device = rodio::default_output_device().unwrap();
    let up_chirp = || {
        if flags.sound {
            rodio::play_raw(&device, beep::Chirp::up(std::time::Duration::from_secs(2)));
        }
    };
    let down_chirp = || {
        if flags.sound {
            rodio::play_raw(&device, beep::Chirp::down(std::time::Duration::from_secs(2)));
        }
    };
    let (a,b) = termion::terminal_size()?;
    let mut x = (rand::random::<u16>() % a) + 1;
    let mut y = (rand::random::<u16>() % b) + 1;
    let alphabet = (b'A'..=b'z')           // Start as u8
        .map(|c| c as char)            // Convert all to chars
        .filter(|c| c.is_alphabetic()) // Filter only alphabetic chars
        .collect::<Vec<_>>();          // Collect as Vec<char>
    let mut the_char = alphabet[rand::random::<usize>() % alphabet.len()];
    let mut the_color = bright_color();
    let mut explosions: Vec<Explosion> = Vec::new();
    let mut rockets: Vec<Rocket> = Vec::new();
    let fragments = vec![
        vec!['*'],
        vec!['*','+','/','\\','-','|'],
        vec!['*','+','/','\\','-','|'],
        vec!['+','/','\\','-','|'],
        vec!['.','+','`','\'',',',' '],
        vec!['.','+','`','\'',',',' '],
        vec!['.','+','`','\'',',',' ',' '],
        vec!['.','`','\'',',',' ',' ',' '],
        vec!['.',' '],
        vec!['.',' ',' '],
    ];
    loop {
        let (a,b) = termion::terminal_size()?;
        {
            let mut screen = AlternateScreen::from(stdout());
            if x > 0 {
                x = x - 1 + (rand::random::<u16>() % 3);
            } else {
                x = x + (rand::random::<u16>() % 2);
            }
            if x >= a {
                x = a;
            }
            if y > 0 {
                y = y - 1 + (rand::random::<u16>() % 3);
            } else {
                y = y + (rand::random::<u16>() % 2);
            }
            if y >= b {
                y = b;
            }
            while let Some(k) = stdin.next() {
                match k {
                    Ok(Key::Esc) => return Ok(()),
                    Ok(Key::Char(c)) => {
                        if c.to_lowercase().next() == the_char.to_lowercase().next() {
                            down_chirp();
                            explosions.push(Explosion {
                                x: x as i32,
                                y: y as i32,
                                fuse: 1,
                                color: the_color,
                            });
                            the_char = alphabet[rand::random::<usize>() % alphabet.len()];
                            the_color = bright_color();
                            x = (rand::random::<u16>() % a) + 1;
                            y = (rand::random::<u16>() % b) + 1;
                        } else {
                            rockets.push(Rocket {
                                x: ((rand::random::<u16>() % a) + 1) as i32,
                                y: b as i32,
                                vy: -1,
                                vx: 0, // rand::random::<i32>() % 3 - 1,
                                color: bright_color(),
                                who: c,
                            });
                        }
                    }
                    _ => (),
                }
            }
            for e in explosions.iter_mut() {
                let fr = &fragments[e.fuse as usize];
                const DIMINISH: u8 = 20;
                if e.color.0 > DIMINISH/2 {
                    e.color.0 -= DIMINISH/2;
                }
                if e.color.1 > DIMINISH {
                    e.color.1 -= DIMINISH;
                }
                if e.color.2 > DIMINISH {
                    e.color.2 -= DIMINISH;
                }
                for dx in -e.fuse ..= e.fuse {
                    for dy in -e.fuse ..= e.fuse {
                        let f = fr[rand::random::<usize>() % fr.len()];
                        if e.x + dx > 0 && e.y + dy > 0 && dx*dx + 2*dy*dy < e.fuse*e.fuse && f != ' ' {
                            write!(screen, "{}{}{}",
                                   termion::cursor::Goto((e.x+dx) as u16,
                                                         (e.y+dy) as u16),
                                   Fg(e.color), f,
                            ).unwrap();
                        }
                    }
                }
                e.fuse += 1;
            }
            for r in rockets.iter_mut() {
                if rand::random::<u16>() % (b/2) == 0 {
                    up_chirp();
                    explosions.push(Explosion {
                        x: r.x,
                        y: r.y,
                        color: r.color,
                        fuse: 0,
                    });
                    write!(screen, "{}{}{}",
                           termion::cursor::Goto(r.x as u16,
                                                 r.y as u16),
                           Fg(r.color),
                           '*',
                    ).unwrap();
                    r.x = 0;
                } else {
                    write!(screen, "{}{}{}",
                           termion::cursor::Goto(r.x as u16,
                                                 r.y as u16),
                           Fg(Rgb(255,0,0)),
                           '^',
                    ).unwrap();
                    r.x += r.vx;
                    r.y += r.vy;
                    write!(screen, "{}{}{}",
                           termion::cursor::Goto(r.x as u16,
                                                 r.y as u16),
                           Fg(r.color),
                           r.who,
                    ).unwrap();
                }
            }
            rockets.retain(|r| r.x > 0 && r.x < a as i32 && r.y > 0);
            explosions.retain(|e| e.fuse < fragments.len() as i32);
            write!(screen, "{}{}{}",
                   termion::cursor::Goto(x, y),
                   Fg(the_color),
                   the_char,
            ).unwrap();
            screen.flush().unwrap();
        }
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
}

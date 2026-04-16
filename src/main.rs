#![feature(ascii_char)]
use std::{
    borrow::Cow,
    io::Read,
    pin::Pin,
    time::{Duration, Instant},
};

use async_io::Timer;
use chrono::{DurationRound, Local, TimeDelta};
use macro_rules_attribute::apply;
use smol_macros::{Executor, main};

mod bar;
mod colour;

use crate::{
    bar::{Align, Bar, Block, Width},
    colour::Rgb,
};

const SUCCESS: Rgb = Rgb::new([166, 227, 161]);
const WARN: Rgb = Rgb::new([249, 226, 175]);
const ERR: Rgb = Rgb::new([243, 139, 168]);
const TEXT: Rgb = Rgb::new([30, 30, 46]);
const TEAL: Rgb = Rgb::new([148, 226, 213]);

fn time(mut block: Block) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    Box::pin(async move {
        let mut string = String::new();
        let mut now = Local::now();
        let mut next_second = Instant::now()
            + (now.duration_round_up(TimeDelta::seconds(1)).unwrap() - now)
                .to_std()
                .unwrap();

        loop {
            Timer::at(next_second).await;
            next_second += Duration::from_secs(1);

            now = Local::now();
            string.clear();
            now.format("%Y-%m-%d %H:%M:%S")
                .write_to(&mut string)
                .expect("Unable to write to string");

            block.set_full_text(&string);
            block.flush();
        }
    })
}

fn battery(mut block: Block) -> Pin<Box<dyn Future<Output = ()> + Send>> {
    let capacity_path = "/sys/class/power_supply/BAT1/capacity";
    let status_path = "/sys/class/power_supply/BAT1/status";
    let mut status_buf = [b'U'];
    block.use_pango(true);
    block.set_width(Some(Width::String(Cow::Borrowed("    "))));
    block.set_align(Some(Align::Centre));

    Box::pin(async move {
        let mut text;
        block.set_text_colour(Some(TEXT));

        loop {
            text = [b' '; 4];
            let n = std::fs::File::open(capacity_path)
                .unwrap()
                .read(text.as_mut_slice())
                .expect("could not read capacity");

            let _ = std::fs::File::open(status_path)
                .unwrap()
                .read_exact(&mut status_buf);

            let status_index = n - 1;

            match status_buf[0] {
                b'U' => text[status_index] = b'?',
                b'C' => {
                    text[status_index] = b'%';
                    block.set_background(Some(WARN));
                }
                b'D' => {
                    text[status_index] = b'%';
                    block.set_background(Some(TEAL));
                }
                b'N' => {
                    text[status_index] = b'%';
                    block.set_background(Some(ERR));
                }
                b'F' => {
                    text[status_index] = b'%';
                    block.set_background(Some(SUCCESS));
                }
                _ => {}
            }

            block.set_full_text(unsafe { text.as_ascii_unchecked() }[..n].as_str());
            block.flush();
            Timer::after(Duration::from_secs(5)).await;
        }
    })
}

// fn cpu(id: u8, tx: Sender<(u8, Block)>) -> Pin<Box<dyn Future<Output = ()> + Send>> {
// Box::pin(async move {
//     let bar_item = Block::new("battery");
// })
// }

#[apply(main!)]
async fn main(ex: &Executor<'static>) {
    Bar::new()
        .add_block("time", Box::new(time))
        .add_block("battery", Box::new(battery))
        .run(ex)
        .await;
}

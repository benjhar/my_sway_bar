use std::{pin::Pin, sync::mpsc::Sender};

use smol_macros::Executor;

use crate::colour::Rgb;

#[derive(serde::Serialize, Default)]
struct BarHeader {
    version: u8,
    click_events: Option<bool>,
    cont_signal: Option<u8>,
    stop_signal: Option<u8>,
}

#[derive(serde::Serialize, Clone)]
pub struct Block {
    /// The text that will be displayed
    full_text: String,
    name: &'static str,

    // Optional parameters
    /// If given and the text needs to be shortened due to space, this will be displayed instead of `full_text`
    #[serde(skip_serializing_if = "Option::is_none")]
    short_text: Option<String>,
    /// The text color to use in #RRGGBBAA or #RRGGBB notation
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<Rgb>,
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<Rgb>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border_top: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border_bottom: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border_left: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    border_right: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    min_wdith: Option<Width>,
    #[serde(skip_serializing_if = "Option::is_none")]
    align: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    urgent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    separator: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    separator_block_width: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    markup: Option<String>,

    // Syncronisation
    #[serde(skip_serializing)]
    id: u8,
    #[serde(skip_serializing)]
    tx: Sender<(u8, String)>,
}

impl Block {
    pub fn new(name: &'static str, id: u8, tx: Sender<(u8, String)>) -> Block {
        Block {
            full_text: String::new(),
            short_text: None,
            color: None,
            background: None,
            border: None,
            border_top: None,
            border_bottom: None,
            border_left: None,
            border_right: None,
            min_wdith: None,
            align: None,
            name,
            instance: None,
            urgent: None,
            separator: None,
            separator_block_width: None,
            markup: None,
            id,
            tx,
        }
    }

    pub fn set_full_text(&mut self, full_text: &str) {
        full_text.clone_into(&mut self.full_text);
    }

    pub fn set_background(&mut self, colour: Option<Rgb>) {
        self.background = colour;
    }

    pub fn set_text_colour(&mut self, colour: Option<Rgb>) {
        self.color = colour;
    }

    /// Update this [`Block`] on the bar.
    ///
    /// # Panics
    ///
    /// This function will panic if the main task panics.
    pub fn flush(&self) {
        self.tx
            .send((self.id, serde_json::to_string(self).unwrap()))
            .unwrap();
    }
}

#[derive(serde::Serialize, Clone)]
#[serde(untagged)]
pub enum Width {
    Int(u8),
    String(String),
}

pub type BlockFn = Box<dyn FnOnce(Block) -> Pin<Box<dyn Future<Output = ()> + Send>>>;

pub struct Bar {
    names: Vec<&'static str>,
    blocks: Vec<BlockFn>,
}

impl Bar {
    pub fn new() -> Bar {
        Bar {
            names: Vec::new(),
            blocks: Vec::new(),
        }
    }

    pub fn add_block(mut self, name: &'static str, block: BlockFn) -> Self {
        self.names.push(name);
        self.blocks.push(block);
        self
    }

    pub async fn run(mut self, ex: &Executor<'_>) {
        assert_eq!(self.names.len(), self.blocks.len());

        let n_blocks = self.blocks.len();

        let (tx0, rx) = std::sync::mpsc::channel();
        let mut futures = Vec::with_capacity(n_blocks);
        let mut i = 0u8;
        while let Some(block) = self.blocks.pop() {
            let name = self.names[(n_blocks - 1) - i as usize];
            let bar_item = Block::new(name, i, tx0.clone());
            futures.push(block(bar_item));
            i += 1;
        }
        drop(self.blocks);

        let mut tasks = Vec::with_capacity(n_blocks);
        ex.spawn_many(futures, &mut tasks);
        while let Some(t) = tasks.pop() {
            t.detach();
        }

        // Drop tx0 so that rx.recv panics if all block tasks exit - otherwise tx0 would still be in
        // scope and it would still be able to block.
        drop(tx0);

        let names = self.names.clone();
        ex.spawn(async move {
            let header = BarHeader {
                version: 1,
                ..Default::default()
            };
            let mut body: Vec<String> = names
                .iter()
                .map(|n| format!(r#"{{"full_text": "","name": "{n}",}},"#))
                .collect();

            println!("{}\n", serde_json::to_string(&header).unwrap());
            println!("[");
            for (i, out) in &rx {
                body[i as usize] = out;

                print!("[");
                let mut j = 0;
                loop {
                    print!("{}", body[j]);
                    j += 1;
                    if j < body.len() {
                        print!(",");
                    } else {
                        break;
                    }
                }
                println!("],");
            }
        })
        .await;
    }
}

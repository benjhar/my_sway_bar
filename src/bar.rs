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
    /// A name for the block. This is only used to identify the block for click events.
    name: &'static str,

    // Optional parameters
    /// If given and the text needs to be shortened due to space, this will be displayed instead of `full_text`
    #[serde(skip_serializing_if = "Option::is_none")]
    short_text: Option<String>,

    /// The text color to use
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<Rgb>,

    /// The background color for the block
    #[serde(skip_serializing_if = "Option::is_none")]
    background: Option<Rgb>,

    // The border color for the block
    #[serde(skip_serializing_if = "Option::is_none")]
    border: Option<Rgb>,

    /// The height in pixels of the top border. The default is 1
    #[serde(skip_serializing_if = "Option::is_none")]
    border_top: Option<u8>,

    /// The height in pixels of the bottom border. The default is 1
    #[serde(skip_serializing_if = "Option::is_none")]
    border_bottom: Option<u8>,

    /// The height in pixels of the left border. The default is 1
    #[serde(skip_serializing_if = "Option::is_none")]
    border_left: Option<u8>,

    /// The height in pixels of the left border. The default is 1
    #[serde(skip_serializing_if = "Option::is_none")]
    border_right: Option<u8>,

    /// The minimum width to use for the block. If a [`Width::String`] is used, with will be
    /// calculated based on the length of the string. If a [`Width::Int`] is used, the mimunum width
    /// is that number of pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    min_wdith: Option<Width>,

    /// If the text does not span the full width of the block, this specifies how the text should be aligned inside of the block. This can be left (default), right, or center.
    // TODO: Change to enum
    #[serde(skip_serializing_if = "Option::is_none")]
    align: Option<String>,

    /// The instance of the name for the block. This is only used to identify the block for click events. If set, each block should have a unique name and instance pair.
    #[serde(skip_serializing_if = "Option::is_none")]
    instance: Option<String>,

    /// Whether the block should be displayed as urgent. Currently swaybar utilizes the colors set in the sway config for urgent workspace buttons. See sway-bar(5) for more information on bar color configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    urgent: Option<bool>,

    /// Whether the bar separator should be drawn after the block. See sway-bar(5) for more information on how to set the separator text.
    #[serde(skip_serializing_if = "Option::is_none")]
    separator: Option<bool>,

    /// The amount of pixels to leave blank after the block. The separator text will be displayed centered in this gap. The default is 9 pixels.
    #[serde(skip_serializing_if = "Option::is_none")]
    separator_block_width: Option<u8>,

    /// The type of markup to use when parsing the text for the block. This can either be pango or none (default).
    // TODO: Change to use_pango `bool` and serialize as markup `Option`
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

    pub fn set_short_text(&mut self, short_text: Option<&str>) {
        if let Some(text) = short_text {
            self.short_text = Some(String::from(text));
        } else {
            self.short_text = None;
        }
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

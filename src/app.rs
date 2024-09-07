use crate::PrintErr;

use super::file_service as cfs;
use cfs::{MAX_CHUNK_SIZE, MIN_CHUNK_SIZE};

use fltk;

use super::Criterion;
use fltk::button::Button;
use fltk::enums::Color;
use fltk::enums::Event;
use fltk::group::Group;
use fltk::input::Input;
use fltk::prelude::*;
use fltk_theme::widget_themes;
use rfd::FileDialog;

// A kilobyte is 1024 bytes (technically that's a kibibyte, but no-one really cares)
const KILOBYTE: u32 = 1024;

// Because I am mutating static variables, any code reading or writing them will need to be marked as unsafe.
// The code is safe though, as it's single threaded.
// Keep this in mind whenever you see the unsafe block.
static mut CHUNK_SIZE: u32 = 25 * KILOBYTE;

pub fn show_file_select() -> Option<std::path::PathBuf> {
    let file = FileDialog::new()
        .set_title("Select file to split")
        .pick_file();
    let filtered = file.meets_criteria(|file| file.is_file());
    filtered.to_owned()
}

pub fn show_folder_select() -> Option<std::path::PathBuf> {
    let dir = FileDialog::new()
        .set_title("Select folder containing reassembly files")
        .pick_folder();
    let filtered = dir.meets_criteria(|file| file.is_dir());
    filtered.to_owned()
}

pub fn start() {
    // Basic FLTK boilerplate.
    // As a side note, in order to compile this on your machine, you WILL need to have cmake installed.
    // Cmake is the bane of my existence, so I hope you have gotten used to it.
    let app = fltk::app::App::default();
    let mut win = fltk::window::Window::new(0, 0, 500, 700, "File Seperator").center_screen();

    // This group contains the preset tabs
    let presets = Group::new(0, 100, 500, 100, None);

    // Preset data
    let sizes = [
        // SMS Messaging
        (3584, "3.5 Mb"),
        // Discord File Limit
        (10 * KILOBYTE, "10 Mb"),
        // iMessage limit
        (100 * KILOBYTE, "100 Mb"),
    ];
    let default_color = Color::Dark1;

    // Preset creation
    for (index, size) in sizes.into_iter().enumerate() {
        let mut preset = Button::new(55 + index as i32 * 100, 100, 90, 40, size.1);

        // Preset two is selected by default when the program opens, so it is automatically a lighter color.
        if index == 1 {
            preset.set_color(default_color);
        } else {
            // Presets which are not selected are darkened/
            preset.set_color(default_color.darker());
        }
        preset.set_callback(move |btn| {
            let parent = btn.parent().unwrap();

            let mut i = 0;
            while let Some(mut child) = parent.child(i) {
                if i == 4 {
                    // Hides the custom input menu, which is child #5
                    child.hide();
                } else {
                    // Darkens all presets
                    child.set_color(default_color.darker());
                }
                // Redraw for changes to take effect
                child.redraw();
                i += 1;
            }
            // Sets own color to be lighter
            btn.set_color(default_color);

            unsafe {
                CHUNK_SIZE = size.0;
            }
        });
    }

    // The custom tab has enough changes for it to not be contained in the main loop.
    let mut custom: Button = Button::new(355, 100, 90, 40, "Custom");
    custom.set_color(default_color.darker());
    custom.set_callback(move |btn| {
        let parent = btn.parent().unwrap();

        let mut i = 0;
        while let Some(mut child) = parent.child(i) {
            if i == 4 {
                // Shows the custom input menu instead of hiding it.
                child.show();
            } else {
                child.set_color(default_color.darker());
            }
            child.redraw();
            i += 1;
        }
        btn.set_color(default_color);
    });

    // Custom input menu
    let mut input = Input::new(75, 150, 350, 40, None);
    input.hide();
    input.set_value("8192 KB");
    input.set_callback(|input| {
        let mut val = &input.value().to_lowercase()[..];
        // Autocorrect to kb if mb are entered
        if val.trim().ends_with("mb") {
            val = val.trim_end_matches("mb");
            if let Ok(new_size) = val.trim().parse::<f32>() {
                let new_size: u32 = (new_size * 1024.0).round() as u32;
                unsafe {
                    CHUNK_SIZE =
                        new_size.clamp(MIN_CHUNK_SIZE / KILOBYTE, MAX_CHUNK_SIZE / KILOBYTE);
                    input.set_value(&format!("{} KB", CHUNK_SIZE));
                }
            } else {
                unsafe {
                    input.set_value(&format!("{} KB", CHUNK_SIZE));
                }
            }
            return;
        } else if val.trim().ends_with("kb") {
            // Ignore if kb is entered
            val = val.trim_end_matches("kb");
        }

        // Parse input as number
        if let Ok(new_size) = val.trim().parse::<u32>() {
            unsafe {
                CHUNK_SIZE = new_size.clamp(MIN_CHUNK_SIZE / KILOBYTE, MAX_CHUNK_SIZE / KILOBYTE);
                input.set_value(&format!("{} KB", CHUNK_SIZE));
            }
        } else {
            // If input is not a number, revert to last valid input.
            unsafe {
                input.set_value(&format!("{} KB", CHUNK_SIZE));
            }
        }
    });
    input.set_text_size(16);

    presets.end();

    // Button to split file
    let mut split = Button::new(50, 400, 400, 75, "Split");
    split.set_color(default_color);
    split.set_frame(widget_themes::OS_BUTTON_UP_BOX);

    split.handle(move |btn: &mut Button, ev: Event| match ev {
        Event::Enter => {
            btn.set_frame(widget_themes::OS_BUTTON_UP_FRAME);
            btn.set_color(default_color.lighter());
            btn.redraw();
            true
        }
        Event::Push => {
            btn.set_frame(widget_themes::OS_CHECK_DOWN_BOX);
            btn.set_color(default_color.darker());
            btn.set_label("Splitting...");
            let path = show_file_select();
            if let Some(path) = path {
                unsafe {
                    cfs::split_file(path, CHUNK_SIZE * KILOBYTE);
                }
            }
            btn.set_label("Split");
            true
        }
        Event::Released => {
            btn.set_color(default_color);
            btn.set_frame(widget_themes::OS_BUTTON_UP_BOX);
            true
        }
        Event::Leave => {
            btn.set_frame(widget_themes::OS_BUTTON_UP_BOX);
            btn.set_color(default_color);
            btn.redraw();
            true
        }
        _ => false,
    });

    // Button to reassemble file
    let mut reassemble = Button::new(50, 500, 400, 75, "Reassemble");
    reassemble.set_color(default_color);
    reassemble.set_frame(widget_themes::OS_BUTTON_UP_BOX);

    reassemble.handle(move |btn: &mut Button, ev: Event| match ev {
        Event::Enter => {
            btn.set_frame(widget_themes::OS_BUTTON_UP_FRAME);
            btn.set_color(default_color.lighter());
            btn.redraw();
            true
        }
        Event::Push => {
            btn.set_frame(widget_themes::OS_CHECK_DOWN_BOX);
            btn.set_color(default_color.darker());
            btn.set_label("Reassembling...");
            let path = show_folder_select();
            if let Some(path) = path {
                cfs::combine_files(path);
            }
            btn.set_label("Reassemble");
            true
        }
        Event::Leave => {
            btn.set_frame(widget_themes::OS_BUTTON_UP_BOX);
            btn.set_color(default_color);
            btn.redraw();
            true
        }
        _ => false,
    });

    win.end();

    win.show();

    app.run().print_and_expect("Could not start application");
}

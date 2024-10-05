pub mod app;
pub mod file_service;
use std::{
    error::Error,
    fmt::{Debug, Display},
};

pub trait Criterion<T> {
    fn meets_criteria(&self, filter: impl Fn(&T) -> bool) -> &Self;
}

impl<T> Criterion<T> for Option<T> {
    fn meets_criteria(&self, filter: impl Fn(&T) -> bool) -> &Self {
        if let Some(item) = self {
            let passes = filter(item);
            if passes {
                self
            } else {
                &None
            }
        } else {
            &None
        }
    }
}

pub trait PrintErr<T> {
    fn print_err(self) -> Self;
    fn print_and_unwrap(self) -> T;
    fn print_and_expect<S: Display + Debug>(self, msg: S) -> T;
}

impl<T, U> PrintErr<T> for Result<T, U>
where
    U: Error,
{
    fn print_err(self) -> Self {
        if let Err(e) = &self {
            let error_msg = format!(
                "Unexpected error: {}\nPlease inform the developer about the error.",
                e
            );
            rfd::MessageDialog::new()
                .set_title("Error")
                .set_description(error_msg.as_str())
                .show();
        };
        self
    }
    fn print_and_unwrap(self) -> T {
        match self {
            Ok(val) => val,
            Err(e) => {
                let error_msg = format!(
                    "Unexpected error: {}\nPlease inform the developer about the error.",
                    e
                );
                rfd::MessageDialog::new()
                    .set_title("Error")
                    .set_description(error_msg.as_str())
                    .show();
                panic!("Unexpected error: {}", e)
            }
        }
    }
    fn print_and_expect<S: Display + Debug>(self, msg: S) -> T {
        match self {
            Ok(val) => val,
            Err(_) => {
                let error_msg = format!("Error: {}", msg);
                rfd::MessageDialog::new()
                    .set_title("Error")
                    .set_description(error_msg.as_str())
                    .show();
                panic!("Unexpected error: {:?}", msg)
            }
        }
    }
}

impl<T> PrintErr<T> for Option<T> {
    fn print_err(self) -> Self {
        if self.is_none() {
            let error_msg = "Unexpected null value.\nPlease inform the developer about the error.";
            rfd::MessageDialog::new()
                .set_title("Error")
                .set_description(error_msg)
                .show();
        }
        self
    }
    fn print_and_unwrap(self) -> T {
        match self {
            Some(val) => val,
            None => {
                let error_msg = "Unexpected null value in Option<T>.\nPlease inform the developer about the error.";
                rfd::MessageDialog::new()
                    .set_title("Error")
                    .set_description(error_msg)
                    .show();
                panic!("Unexpected null value.")
            }
        }
    }
    fn print_and_expect<S: Display + Debug>(self, msg: S) -> T {
        match self {
            Some(val) => val,
            None => {
                let error_msg = format!("Error: {}", msg);
                rfd::MessageDialog::new()
                    .set_title("Error")
                    .set_description(error_msg.as_str())
                    .show();
                panic!("Unexpected null value.")
            }
        }
    }
}

#[derive(Debug)]
pub enum ExpectedErrors {
    UnreadableFile,
    BadFiles,
    NoFiles,
    NoFolder,
    NoParent,
    NullPath,
    NoName,
    CouldNotMakeDir(String),
    CouldNotCreateFile(String),
    CouldNotWriteFile(String),
    CouldNotReadDir(String),
    FileTooBig(u64, u64),
    FileTooSmall(u64, u64),
    CouldNotReadFile(String),
    FileLargerThanChunkSize,
    InvalidString,
    GiantFile,
}

impl ExpectedErrors {
    pub fn print(&self) {
        let err_title = format!("Error: class {:?}", self);
        let mut window = rfd::MessageDialog::new().set_title(err_title.as_str());

        match self {
            ExpectedErrors::UnreadableFile => {
                window = window.set_description(
                    "File is unreadable. Please check the file's permissions and try again.",
                );
            }
            ExpectedErrors::BadFiles => {
                window = window.set_description(
                    "One or more files are invalid. Please check the files and try again.",
                );
            }
            ExpectedErrors::NoFiles => {
                window = window.set_description("No .reassembly files were selected. Please select at least one file and try again.");
            }
            ExpectedErrors::NoFolder => {
                window = window.set_description(
                    "No folder was selected. Please select a folder and try again.",
                );
            }
            ExpectedErrors::NoParent => {
                window = window.set_description("Selected file or folder has no parent path. Please do not reassemble your actual ROOT DIRECTORY. That's really stupid to even TRY, I won't lie. But to be fair, I did literally write this specific error message for this EXACT thing, so whatever.");
            }
            ExpectedErrors::NullPath => {
                window = window
                    .set_description("No path was selected. Please select a path and try again.");
            }
            ExpectedErrors::NoName => {
                window = window.set_description("The file you selected has no name.");
            }
            ExpectedErrors::CouldNotCreateFile(path) => {
                window =
                    window.set_description(format!("Could not create file {}.", path).as_str());
            }
            ExpectedErrors::CouldNotMakeDir(path) => {
                window = window
                    .set_description(format!("Could not create directory {}.", path).as_str());
            }
            ExpectedErrors::CouldNotWriteFile(path) => {
                window = window.set_description(
                    format!(
                        "Could not write to file {}. Check permissions and try again.",
                        path
                    )
                    .as_str(),
                );
            }
            ExpectedErrors::FileTooBig(file_size, max_size) => {
                let size_in_mb: f32 = *file_size as f32 / 1024.0 / 1024.0;
                let max_size_in_mb: f32 = *max_size as f32 / 1024.0 / 1024.0;
                let text = format!(
                    "Chunk size is too big. Max size: {} MB. Actual size: {} MB.",
                    max_size_in_mb, size_in_mb
                );
                window = window.set_description(text);
            }
            ExpectedErrors::FileTooSmall(file_size, min_size) => {
                let size_in_mb: f32 = *file_size as f32 / 1024.0 / 1024.0;
                let min_size_in_mb: f32 = *min_size as f32 / 1024.0;
                let text = format!(
                    "Chunk size is too small. Min size: {} KB. Actual size: {} MB.",
                    min_size_in_mb, size_in_mb
                );
                window = window.set_description(text);
            }
            ExpectedErrors::CouldNotReadFile(path) => {
                window = window.set_description(
                    format!(
                        "Could not read file {}. Check permissions and try again.",
                        path
                    )
                    .as_str(),
                );
            }
            ExpectedErrors::FileLargerThanChunkSize => {
                window = window.set_description("The file you selected is larger than the chunk size. Please select a smaller file or chunk size and try again.");
            }
            ExpectedErrors::CouldNotReadDir(path) => {
                window = window.set_description(
                    format!(
                        "Could not read dir {}. Check permissions and try again.",
                        path
                    )
                    .as_str(),
                );
            }
            ExpectedErrors::InvalidString => {
                window = window.set_description(
                    "Invalid string. The utf-8 bytes may have been manipulated or changed.",
                );
            }
            ExpectedErrors::GiantFile => {
                window = window.set_description("The file you chose is waaaaaayy too big. I'm suprised you have that much storage space.");
            }
        }
        window.show();
        #[cfg(test)]
        {
            panic!()
        }
    }
}

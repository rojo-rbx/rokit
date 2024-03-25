use std::{
    env::{args, consts::EXE_EXTENSION},
    path::PathBuf,
};

pub fn arg0_file_name() -> String {
    let arg0 = args().next().unwrap();
    let exe_path = PathBuf::from(arg0);
    let exe_name = exe_path
        .file_name()
        .expect("Invalid file name passed as arg0")
        .to_str()
        .expect("Non-UTF8 file name passed as arg0")
        .trim_end_matches(EXE_EXTENSION);
    exe_name.to_string()
}

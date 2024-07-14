use anyhow::Error;
use console::style;

use rokit::{
    descriptor::{Arch, Descriptor, OS},
    tool::ToolAlias,
};

pub fn inform_user_about_potential_fixes(alias: &ToolAlias, e: &Error) {
    if is_likely_rosetta2_error(e) {
        suggest_installing_rosetta(alias);
    }
}

fn is_likely_rosetta2_error(e: &Error) -> bool {
    let is_bad_cpu_type = e
        .to_string()
        .to_ascii_lowercase()
        .contains("bad cpu type in executable");

    let is_running_macos_aarch64 = {
        let current = Descriptor::current_system();
        matches!(current.os(), OS::MacOS) && matches!(current.arch(), Some(Arch::Arm64))
    };

    is_bad_cpu_type && is_running_macos_aarch64
}

fn suggest_installing_rosetta(alias: &ToolAlias) {
    tracing::error!(
        "Rokit failed to run tool {} because of a 'bad CPU type in executable' error.\
        \nThis is likely because it was compiled for an Intel Mac and you are running an Apple Silicon Mac.\
        \n\nRosetta 2 is a compatibility layer that enables running x86_64 apps on \
        Apple Silicon Macs, and can be installed by running the following command:\
        \n\n{}\n",
        style(alias.to_string()).bold().cyan(),
        style("softwareupdate --install-rosetta").bold()
    );
}
